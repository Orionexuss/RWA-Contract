use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AuctionState {
    pub asset: Pubkey,
    pub auction_creator: Pubkey,
    pub ft_mint: Pubkey,        // Mint of the tokenized asset being auctioned
    pub bid_token_mint: Pubkey, // USDC mint address for bids
    pub is_active: bool,
    pub highest_bid: u64,
    pub highest_bidder: Pubkey,
    pub auction_end_time: i64,
    pub bump: u8,
}
