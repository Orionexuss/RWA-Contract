use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::error::ErrorCode;
use crate::state::{AssetState, AuctionState};
use crate::{SEED_AUCTION_STATE_ACCOUNT, SEED_AUCTION_VAULT_ACCOUNT, SEED_STATE_ACCOUNT};

#[derive(Accounts)]
pub struct SettleAuction<'info> {
    #[account(mut)]
    pub settler: Signer<'info>,

    /// CHECK: Auction creator is validated through auction_state has_one constraint
    #[account(mut)]
    pub auction_creator: AccountInfo<'info>,

    /// CHECK: Highest bidder is validated through auction_state has_one constraint
    #[account(mut)]
    pub highest_bidder: AccountInfo<'info>,

    /// CHECK: Asset account is validated through auction_state has_one constraint
    pub asset: AccountInfo<'info>,

    /// Mint of the tokenized asset being auctioned
    pub ft_mint: InterfaceAccount<'info, Mint>,

    /// USDC mint for bids
    pub usdc_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        has_one = auction_creator,
        has_one = ft_mint,
        has_one = asset,
        has_one = highest_bidder,
        constraint = ft_mint.key() == auction_state.ft_mint @ ErrorCode::InvalidChoice,
        constraint = usdc_mint.key() == auction_state.bid_token_mint @ ErrorCode::InvalidBidToken,
        seeds = [SEED_AUCTION_STATE_ACCOUNT, auction_creator.key().as_ref()],
        bump = auction_state.bump
    )]
    pub auction_state: Account<'info, AuctionState>,

    #[account(
        has_one = asset,
        has_one = ft_mint,
        constraint = asset_state.ft_mint == ft_mint.key() @ ErrorCode::InvalidChoice,
        seeds = [SEED_STATE_ACCOUNT, asset.key().as_ref()],
        bump = asset_state.bump,
    )]
    pub asset_state: Account<'info, AssetState>,

    /// CHECK: PDA authority for auction vault
    #[account(
        seeds = [SEED_AUCTION_VAULT_ACCOUNT, auction_creator.key().as_ref()],
        bump
    )]
    pub auction_vault_pda: UncheckedAccount<'info>,

    // Vault holding the asset tokens being auctioned (self-custodied)
    #[account(
        mut,
        token::mint = ft_mint,
        token::authority = auction_vault_pda,
        seeds = [SEED_AUCTION_VAULT_ACCOUNT, auction_creator.key().as_ref()],
        bump
    )]
    pub auction_vault: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: PDA authority for bids vault
    #[account(
        seeds = [SEED_AUCTION_STATE_ACCOUNT, auction_creator.key().as_ref()],
        bump
    )]
    pub auction_state_pda: UncheckedAccount<'info>,

    // Vault holding the USDC bids (owned by auction_state PDA)
    #[account(
        mut,
        token::mint = usdc_mint,
        token::authority = auction_state_pda,
    )]
    pub bids_vault: InterfaceAccount<'info, TokenAccount>,

    // Auction creator's USDC account to receive the winning bid
    #[account(
        init_if_needed,
        payer = settler,
        token::mint = usdc_mint,
        token::authority = auction_creator,
    )]
    pub auction_creator_usdc_account: InterfaceAccount<'info, TokenAccount>,

    // Highest bidder's token account to receive the auctioned asset tokens
    #[account(
        init_if_needed,
        payer = settler,
        token::mint = ft_mint,
        token::authority = highest_bidder,
    )]
    pub highest_bidder_asset_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_settle_auction(ctx: Context<SettleAuction>) -> Result<()> {
    let clock = Clock::get()?;
    let auction_state = &mut ctx.accounts.auction_state;

    // Ensure auction has ended
    require!(
        clock.unix_timestamp >= auction_state.auction_end_time,
        ErrorCode::AuctionStillActive
    );

    // Ensure auction is still active (not already settled)
    require!(auction_state.is_active, ErrorCode::AuctionAlreadySettled);

    // Ensure there was at least one bid
    require!(auction_state.highest_bid > 0, ErrorCode::NoBidsPlaced);

    let usdc_decimals = ctx.accounts.usdc_mint.decimals;
    let asset_decimals = ctx.accounts.ft_mint.decimals;
    let auction_creator_key = ctx.accounts.auction_creator.key();
    let highest_bid_amount = auction_state.highest_bid;
    let auction_bump = auction_state.bump;

    // Generate signer seeds for the auction_state PDA
    let auction_state_seeds = &[
        SEED_AUCTION_STATE_ACCOUNT,
        auction_creator_key.as_ref(),
        &[auction_bump],
    ];
    let signer_seeds = &[&auction_state_seeds[..]];

    // Transfer the winning USDC bid from bids_vault to auction creator
    let transfer_bid_accounts = TransferChecked {
        from: ctx.accounts.bids_vault.to_account_info(),
        to: ctx.accounts.auction_creator_usdc_account.to_account_info(),
        authority: ctx.accounts.auction_state_pda.to_account_info(),
        mint: ctx.accounts.usdc_mint.to_account_info(),
    };

    let transfer_bid_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_bid_accounts,
        signer_seeds,
    );

    transfer_checked(transfer_bid_ctx, highest_bid_amount, usdc_decimals)?;

    // Generate signer seeds for the auction_vault PDA
    let vault_seeds = &[
        SEED_AUCTION_VAULT_ACCOUNT,
        auction_creator_key.as_ref(),
        &[ctx.bumps.auction_vault_pda],
    ];
    let vault_signer_seeds = &[&vault_seeds[..]];

    // Transfer the auctioned asset tokens from auction_vault to highest bidder
    let transfer_tokens_accounts = TransferChecked {
        from: ctx.accounts.auction_vault.to_account_info(),
        to: ctx.accounts.highest_bidder_asset_account.to_account_info(),
        authority: ctx.accounts.auction_vault_pda.to_account_info(),
        mint: ctx.accounts.ft_mint.to_account_info(),
    };

    let transfer_tokens_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_tokens_accounts,
        vault_signer_seeds,
    );

    let auction_vault_amount = ctx.accounts.auction_vault.amount;
    transfer_checked(transfer_tokens_ctx, auction_vault_amount, asset_decimals)?;

    // Mark auction as settled
    ctx.accounts.auction_state.is_active = false;

    msg!("Auction settled successfully!");
    msg!(
        "Winning bid: {} transferred to auction creator",
        highest_bid_amount
    );
    msg!(
        "Tokens: {} transferred to highest bidder",
        auction_vault_amount
    );

    Ok(())
}
