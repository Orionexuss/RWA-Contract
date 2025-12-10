pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
pub use constants::*;
pub use instructions::*;

declare_id!("CBMZzRxxmag85FWsXqgfZMB5xE7fw9hg37WCfrvbQgtU");

#[program]
pub mod rwa_contract {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn create_fungible_token(
        ctx: Context<CreateFungibleToken>,
        decimals: u8,
        supply: u8,
    ) -> Result<()> {
        create_fungible_token_and_revoke_authority(ctx, decimals, supply)
    }

    pub fn create_non_fungible_token(
        ctx: Context<CreateNonFungibleToken>,
        args: CreateAssetArgs,
    ) -> Result<()> {
        handle_create_non_fungible_token(ctx, args)
    }

    pub fn create_vote_round(ctx: Context<CreateVoteRound>, description: String) -> Result<()> {
        handle_create_vote_round(ctx, description)
    }

    pub fn vote(ctx: Context<Vote>, choice: u8) -> Result<()> {
        handle_vote(ctx, choice)
    }

    pub fn create_auction(
        ctx: Context<CreateAuction>,
        amount: u64,
        auction_end_time: i64,
    ) -> Result<()> {
        handle_create_auction(ctx, amount, auction_end_time)
    }

    pub fn place_bid(ctx: Context<PlaceBid>, bid_amount: u64) -> Result<()> {
        handle_place_bid(ctx, bid_amount)
    }
}
