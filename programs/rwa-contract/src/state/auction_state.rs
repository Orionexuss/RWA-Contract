use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AuctionState {
    pub auction_creator: Pubkey,
    pub is_active: bool,
    pub highest_bid: u64,
    pub highest_bidder: Pubkey,
    pub auction_end_time: i64,
}
