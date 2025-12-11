use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::spl_token_2022::instruction::AuthorityType, 
    token_interface::{mint_to, set_authority, Mint, MintTo, SetAuthority, TokenAccount, TokenInterface}
};

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct CreateFungibleToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        init, 
        payer = payer,
        mint::decimals = decimals,
        mint::authority = payer,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn create_fungible_token_and_revoke_authority(ctx: Context<CreateFungibleToken>, decimals: u8, supply: u8) -> Result<()> {
    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    mint_to(cpi_ctx, supply as u64 * 10u64.pow(decimals as u32))?;
    
    let cpi_accounts_for_revoke = SetAuthority{
        account_or_mint: ctx.accounts.mint.to_account_info(),
        current_authority: ctx.accounts.payer.to_account_info(),
    };

    let cpi_ctx_revoke = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_for_revoke);
    set_authority(cpi_ctx_revoke,AuthorityType::MintTokens, None)?;

    Ok(())
}
