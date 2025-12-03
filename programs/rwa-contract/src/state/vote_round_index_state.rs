use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VoteRoundIndexState {
    pub asset: Pubkey,
    pub vote_round_count: u64,
    pub bump: u8,
}
