use crate::constants::*;
use crate::error::ErrorCode;
use crate::state::{AssetState, AuctionState};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct SettleAuctionWithoutBids<'info> {
    #[account(mut)]
    pub settler: Signer<'info>,

    /// CHECK: validated against auction_state
    #[account(mut)]
    pub auction_creator: UncheckedAccount<'info>,

    /// CHECK: validated against auction_state
    pub asset: UncheckedAccount<'info>,

    pub ft_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [
            SEED_AUCTION_STATE_ACCOUNT,
            auction_creator.key().as_ref(),
            asset.key().as_ref()
        ],
        bump,
        close = auction_creator
    )]
    pub auction_state: Account<'info, AuctionState>,

    #[account(
        seeds = [SEED_STATE_ACCOUNT, asset.key().as_ref()],
        bump
    )]
    pub asset_state: Account<'info, AssetState>,

    /// CHECK: PDA authority for auction vault
    #[account(
        seeds = [
            SEED_AUCTION_VAULT_ACCOUNT,
            auction_creator.key().as_ref(),
            asset.key().as_ref()
        ],
        bump
    )]
    pub auction_vault_pda: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            SEED_AUCTION_VAULT_ACCOUNT,
            auction_creator.key().as_ref(),
            asset.key().as_ref()
        ],
        bump,
    )]
    pub auction_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = settler,
        associated_token::mint = ft_mint,
        associated_token::authority = auction_creator,
        associated_token::token_program = token_program,
    )]
    pub creator_asset_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_settle_auction_without_bids(ctx: Context<SettleAuctionWithoutBids>) -> Result<()> {
    let clock = Clock::get()?;
    let auction_state = &mut ctx.accounts.auction_state;

    require_keys_eq!(
        auction_state.auction_creator,
        ctx.accounts.auction_creator.key(),
        ErrorCode::InvalidAuctionCreator
    );
    require_keys_eq!(
        auction_state.asset,
        ctx.accounts.asset.key(),
        ErrorCode::InvalidAsset
    );
    require_keys_eq!(
        auction_state.ft_mint,
        ctx.accounts.ft_mint.key(),
        ErrorCode::InvalidMint
    );
    require_keys_eq!(
        ctx.accounts.asset_state.asset,
        ctx.accounts.asset.key(),
        ErrorCode::InvalidAsset
    );
    require_keys_eq!(
        ctx.accounts.asset_state.ft_mint,
        ctx.accounts.ft_mint.key(),
        ErrorCode::InvalidMint
    );
    require_keys_eq!(
        ctx.accounts.auction_vault.mint,
        ctx.accounts.ft_mint.key(),
        ErrorCode::InvalidMint
    );

    require!(
        clock.unix_timestamp >= auction_state.auction_end_time,
        ErrorCode::AuctionStillActive
    );
    require!(auction_state.is_active, ErrorCode::AuctionAlreadySettled);
    require!(auction_state.highest_bid == 0, ErrorCode::BidsAlreadyPlaced);

    let auction_creator_key = ctx.accounts.auction_creator.key();
    let asset_key = ctx.accounts.asset.key();
    let asset_decimals = ctx.accounts.ft_mint.decimals;
    let auction_vault_amount = ctx.accounts.auction_vault.amount;

    let vault_seeds = &[
        SEED_AUCTION_VAULT_ACCOUNT,
        auction_creator_key.as_ref(),
        asset_key.as_ref(),
        &[ctx.bumps.auction_vault_pda],
    ];
    let vault_signer_seeds = &[&vault_seeds[..]];

    if auction_vault_amount > 0 {
        let refund_accounts = TransferChecked {
            from: ctx.accounts.auction_vault.to_account_info(),
            to: ctx.accounts.creator_asset_account.to_account_info(),
            authority: ctx.accounts.auction_vault_pda.to_account_info(),
            mint: ctx.accounts.ft_mint.to_account_info(),
        };

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                refund_accounts,
                vault_signer_seeds,
            ),
            auction_vault_amount,
            asset_decimals,
        )?;
    }

    auction_state.is_active = false;

    msg!("Auction settled without bids");
    msg!(
        "Refunded {} fractional tokens to {}",
        auction_vault_amount,
        auction_creator_key
    );

    Ok(())
}
