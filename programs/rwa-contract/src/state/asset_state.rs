use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AssetState {
    pub nft: Pubkey,
    pub ft_mint: Pubkey,
    pub total_shares: u64,
    pub bump: u8,
}
