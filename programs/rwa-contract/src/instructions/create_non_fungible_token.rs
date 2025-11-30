use anchor_lang::prelude::*;
use mpl_core::instructions::CreateV2CpiBuilder;
use mpl_core::ID as MPL_CORE_ID;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateAssetArgs {
    name: String,
    uri: String,
}

#[derive(Accounts)]
pub struct CreateNonFungibleToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    // CHECK: this account will be checked by the mpl_core program
    pub asset: Signer<'info>,

    // CHECK: this account will be checked by the mpl_core program
    pub owner: UncheckedAccount<'info>,

    #[account(seeds = [b"authority", asset.key().as_ref()], bump)]
    pub authority_pda: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = MPL_CORE_ID)]
    // CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn create_non_fungible_token(
    ctx: Context<CreateNonFungibleToken>,
    args: CreateAssetArgs,
) -> Result<()> {
    let cpi_program = ctx.accounts.mpl_core_program.to_account_info();

    // Build CPI
    let mut builder = CreateV2CpiBuilder::new(&cpi_program)
        .asset(&ctx.accounts.asset.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .owner(Some(&ctx.accounts.owner.to_account_info()))
        .authority(Some(&ctx.accounts.authority_pda.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .uri(args.uri)
        .name(args.name)
        .invoke()?;

    Ok(())
}
