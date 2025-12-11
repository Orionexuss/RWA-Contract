use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::{error::ErrorCode, state::{AssetState, VoteRecord, VoteState}, SEED_STATE_ACCOUNT, SEED_VOTE_RECORD_ACCOUNT, SEED_VOTE_STATE_ACCOUNT};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Yes,
    No,
}

#[derive(Accounts)]
pub struct Vote<'info> {

    #[account(mut)]
    pub voter: Signer<'info>,

    /// CHECK:
    pub asset: AccountInfo<'info>,

    pub ft_mint: InterfaceAccount<'info, Mint>,


    #[account(
        has_one = asset,
        has_one = ft_mint,
        seeds = [SEED_STATE_ACCOUNT, asset.key().as_ref()],
        bump = asset_state.bump,
    )]
    pub asset_state: Account<'info, AssetState>,

    #[account(
        mut,
        seeds = [SEED_VOTE_STATE_ACCOUNT, asset.key().as_ref()],
        bump = vote_state.bump,
        has_one = ft_mint, 
        has_one = asset
        )]
    pub vote_state: Account<'info, VoteState>,

    #[account(
        init,
        payer = voter,
        space = 8 + VoteRecord::INIT_SPACE,
        seeds = [
            SEED_VOTE_RECORD_ACCOUNT,
            vote_state.key().as_ref(),
            voter.key().as_ref(),
        ],
        bump,
    )]
    pub vote_record: Account<'info, VoteRecord>,


    #[account(
        token::mint = ft_mint.key(),
        token::authority = voter.key()
    )]
    pub voter_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
}

pub fn handle_vote(ctx: Context<Vote>, choice: u8) -> Result<()> {
    require!(choice == 0 || choice == 1, ErrorCode::InvalidChoice);

    let weight = ctx.accounts.voter_token_account.amount;
    require!(weight > 0, ErrorCode::NoShares);

    let vote_state = &mut ctx.accounts.vote_state;
    let vote_record = &mut ctx.accounts.vote_record;
    vote_record.vote_state = vote_state.key();
    vote_record.voter = ctx.accounts.voter.key();
    vote_record.choice = choice;
    vote_record.weight = weight;
    vote_record.bump = ctx.bumps.vote_record;

    if choice == 1 {
        vote_state.yes_weight = vote_state
            .yes_weight
            .checked_add(weight)
            .ok_or(ErrorCode::Overflow)?;
    } else {
        vote_state.no_weight = vote_state
            .no_weight
            .checked_add(weight)
            .ok_or(ErrorCode::Overflow)?;
    }

    Ok(())
}
