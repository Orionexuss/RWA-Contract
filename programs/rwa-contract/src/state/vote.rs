use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VoteState {
    pub voter: Pubkey,

    #[max_len(50)]
    pub description: String,
    pub ft_mint: Pubkey,
    pub asset: Pubkey,
    pub yes_weight: u64,
    pub no_weight: u64,
    pub bump: u8,
}
