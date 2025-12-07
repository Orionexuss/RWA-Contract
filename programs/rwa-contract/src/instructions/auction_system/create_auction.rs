use crate::{error::ErrorCode, state::AuctionState};
use crate::{SEED_AUCTION_STATE_ACCOUNT, SEED_AUCTION_VAULT_ACCOUNT};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        token::mint = mint.key(),
        token::authority = payer.key()
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = payer,
        token::mint = mint,
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
    // Validate that the signer has a balance greater than 0
    let token_account_data = &ctx.accounts.token_account;

    // Check if the token account has enough balance for the auction
    if token_account_data.amount < amount {
        return Err(ErrorCode::InsuficientTokenBalance.into());
    }

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.token_account.to_account_info(),
        to: ctx.accounts.auction_vault.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer_checked(cpi_ctx, amount, ctx.accounts.mint.decimals)?;

    let auction_state = &mut ctx.accounts.auction_state;
    auction_state.auction_creator = ctx.accounts.payer.key();
    auction_state.is_active = true;
    auction_state.highest_bid = 0;
    auction_state.auction_end_time = auction_end_time;

    Ok(())
}
