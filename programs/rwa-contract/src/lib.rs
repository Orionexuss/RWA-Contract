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
}
