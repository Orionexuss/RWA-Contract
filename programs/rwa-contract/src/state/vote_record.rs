use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VoteRecord {
    /// The voting round this vote belongs to
    pub vote_state: Pubkey,

    pub voter: Pubkey,

    pub choice: u8,

    pub weight: u64,

    pub bump: u8,
}
