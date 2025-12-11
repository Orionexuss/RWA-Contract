use crate::constants::USDC_MINT_DEVNET;
use crate::state::AssetState;
use crate::{error::ErrorCode, state::AuctionState};
use crate::{SEED_AUCTION_STATE_ACCOUNT, SEED_AUCTION_VAULT_ACCOUNT, SEED_STATE_ACCOUNT};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub ft_mint: InterfaceAccount<'info, Mint>,

    /// USDC mint account for bids
    #[account(
        constraint = usdc_mint.key().to_string() == USDC_MINT_DEVNET @ ErrorCode::InvalidBidToken
    )]
    pub usdc_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Asset account is validated through asset_state PDA
    pub asset: AccountInfo<'info>,

    #[account(
        has_one = ft_mint,
        has_one = asset,
        seeds = [SEED_STATE_ACCOUNT, asset.key().as_ref()],
        bump = asset_state.bump,
    )]
    pub asset_state: Account<'info, AssetState>,

    #[account(
        token::mint = ft_mint.key(),
        token::authority = payer.key(),
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        token::mint = ft_mint,
        token::authority = token_account,
        token::token_program = token_program,
        seeds = [SEED_AUCTION_VAULT_ACCOUNT, payer.key().as_ref()],
        bump
    )]
    pub auction_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        space = 8 + AuctionState::INIT_SPACE,
        seeds = [SEED_AUCTION_STATE_ACCOUNT, payer.key().as_ref()],
        bump
    )]
    pub auction_state: Account<'info, AuctionState>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_create_auction(
    ctx: Context<CreateAuction>,
    amount: u64,
    auction_end_time: i64,
) -> Result<()> {
    let token_account_data = &ctx.accounts.token_account;

    if token_account_data.amount < amount {
        return Err(ErrorCode::InsuficientTokenBalance.into());
    }

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.token_account.to_account_info(),
        to: ctx.accounts.auction_vault.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
        mint: ctx.accounts.ft_mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer_checked(cpi_ctx, amount, ctx.accounts.ft_mint.decimals)?;

    let auction_state = &mut ctx.accounts.auction_state;
    auction_state.asset = ctx.accounts.asset.key();
    auction_state.auction_creator = ctx.accounts.payer.key();
    auction_state.ft_mint = ctx.accounts.ft_mint.key();
    auction_state.bid_token_mint = ctx.accounts.usdc_mint.key();
    auction_state.is_active = true;
    auction_state.highest_bid = 0;
    auction_state.highest_bidder = Pubkey::default();
    auction_state.auction_end_time = auction_end_time;
    auction_state.bump = ctx.bumps.auction_state;

    Ok(())
}
