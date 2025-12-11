use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use mpl_core::instructions::CreateV2CpiBuilder;
use mpl_core::ID as MPL_CORE_ID;

use crate::state::AssetState;
use crate::{SEED_STATE_ACCOUNT, SEED_VAULT_AUTHORITY_ACCOUNT, SEED_VAULT_OWNER_ACCOUNT};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateAssetArgs {
    pub name: String,
    pub uri: String,
}

#[derive(Accounts)]
pub struct CreateNonFungibleToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 0,
        seeds = [SEED_VAULT_OWNER_ACCOUNT, asset.key().as_ref()],
        bump
    )]
    /// CHECK:
    pub owner: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = 0,
        seeds = [SEED_VAULT_AUTHORITY_ACCOUNT, asset.key().as_ref()],
        bump
    )]
    /// CHECK:
    pub authority_pda: UncheckedAccount<'info>,

    pub ft_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        space = 8 + AssetState::INIT_SPACE,
        seeds = [SEED_STATE_ACCOUNT, asset.key().as_ref()],
        bump
    )]
    pub asset_state: Account<'info, AssetState>,

    pub system_program: Program<'info, System>,

    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn handle_create_non_fungible_token(
    ctx: Context<CreateNonFungibleToken>,
    args: CreateAssetArgs,
) -> Result<()> {
    let cpi_program = ctx.accounts.mpl_core_program.to_account_info();

    let asset_key = ctx.accounts.asset.key();
    let owner_bump = ctx.bumps.owner;
    let authority_bump = ctx.bumps.authority_pda;

    let owner_seeds = &[
        SEED_VAULT_OWNER_ACCOUNT,
        asset_key.as_ref(),
        &[owner_bump],
    ];
    let authority_seeds = &[
        SEED_VAULT_AUTHORITY_ACCOUNT,
        asset_key.as_ref(),
        &[authority_bump],
    ];

    CreateV2CpiBuilder::new(&cpi_program)
        .asset(&ctx.accounts.asset.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .owner(Some(&ctx.accounts.owner.to_account_info()))
        .authority(Some(&ctx.accounts.authority_pda.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .uri(args.uri)
        .name(args.name)
        .invoke_signed(&[owner_seeds, authority_seeds])?;

    let asset_state = &mut ctx.accounts.asset_state;
    asset_state.asset = ctx.accounts.asset.key();
    asset_state.ft_mint = ctx.accounts.ft_mint.key();
    asset_state.total_shares = ctx.accounts.ft_mint.supply;
    asset_state.bump = ctx.bumps.asset_state;

    Ok(())
}
