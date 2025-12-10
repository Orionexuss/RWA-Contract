use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked};

use crate::error::ErrorCode;
use crate::state::{AssetState, AuctionState};
use crate::constants::{SEED_AUCTION_STATE_ACCOUNT};
use crate::SEED_STATE_ACCOUNT;

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,

    pub auction_creator: AccountInfo<'info>,

    pub asset: AccountInfo<'info>,

    pub ft_mint: InterfaceAccount<'info, Mint>,

    #[account(
        token::mint = ft_mint.key(),
        token::authority = bidder.key()
    )]
    pub bidder_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        has_one = auction_creator,
        has_one = asset,
        seeds = [SEED_AUCTION_STATE_ACCOUNT, auction_creator.key().as_ref()],
        bump = auction_state.bump
        )]
    pub auction_state: Account<'info, AuctionState>,

    #[account(
        has_one = ft_mint,
        has_one = asset,
        seeds = [SEED_STATE_ACCOUNT, asset.key().as_ref()],
        bump = asset_state.bump,
    )]
    pub asset_state: Account<'info, AssetState>,

    #[account(
        init_if_needed,
        payer = bidder,
        associated_token::mint = ft_mint,
        associated_token::authority = auction_state
        
    )]
    pub bids_vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_place_bid(ctx: Context<PlacedBid>, amount: u64) -> Result<()> {

    // Validations
    // Check if auction is still active
    let clock = Clock::get()?;
    let auction_state = &mut ctx.accounts.auction_state;
    require!(clock.unix_timestamp < auction_state.auction_end_time, ErrorCode::AuctionEnded);

    // Check if bidder has enough balance
    let bidder_token_account = &ctx.accounts.bidder_token_account;
    require!(bidder_token_account.amount >= amount, ErrorCode::InsuficientTokenBalance);

    // Check if bid amount is higher than current highest bid
    if amount <= auction_state.highest_bid {
        return Err(ErrorCode::BidTooLow.into());
    }

    // Transfer bid amount from bidder to bids_vault
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.bidder_token_account.to_account_info(),
        to: ctx.accounts.bids_vault.to_account_info(),
        authority: ctx.accounts.bidder.to_account_info(),
        mint: ctx.accounts.ft_mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    let decimals = ctx.accounts.ft_mint.decimals;

    transfer_checked(cpi_ctx, amount, decimals)?;

    // Update auction state with the new highest bid
        auction_state.highest_bid = amount;
        auction_state.highest_bidder  = ctx.accounts.bidder.key();
    
    Ok(())
}
