use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::error::ErrorCode;
use crate::state::{AssetState, VoteRoundIndexState, VoteState};

#[derive(Accounts)]
pub struct CreateVoteRound<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub asset: AccountInfo<'info>,

    #[account(
        token::mint = ft_mint.key(),
        token::authority = payer.key()
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub ft_mint: InterfaceAccount<'info, Mint>,

    #[account(
        has_one = asset,
        has_one = ft_mint
    )]
    pub asset_state: Account<'info, AssetState>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + VoteRoundIndexState::INIT_SPACE,
        seeds = [b"vote_round_index", asset.key().as_ref()],
        bump,
    )]
    // Each vote round gets a unique vote_state account, derived using vote_round_count as a seed.
    pub vote_round_index: Account<'info, VoteRoundIndexState>,

    #[account(
        init,
        payer = payer,
        space = 8 +VoteState::INIT_SPACE,
        seeds = [b"vote", asset.key().as_ref(), payer.key().as_ref(), vote_round_index.vote_round_count.to_le_bytes().as_ref()],
        bump
    )]
    pub vote_state: Account<'info, VoteState>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_vote_round(ctx: Context<CreateVoteRound>, description: String) -> Result<()> {
    //validates signer has ft_mint balance > 0
    let token_account_data = &ctx.accounts.token_account;
    if token_account_data.amount == 0 {
        return Err(ErrorCode::NotTokenBalance.into());
    }

    let vote = &mut ctx.accounts.vote_state;
    vote.description = description;
    vote.voter = ctx.accounts.payer.key();
    vote.asset = ctx.accounts.asset.key();
    vote.yes_weight = 0;
    vote.no_weight = 0;

    let vote_round_count = ctx.accounts.vote_round_index.vote_round_count;
    let vote_round_index = &mut ctx.accounts.vote_round_index;
    vote_round_index.asset = ctx.accounts.asset.key();
    vote_round_index.vote_round_count = vote_round_count + 1;
    Ok(())
}
