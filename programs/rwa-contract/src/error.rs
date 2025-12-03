use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The token account does not have the required token balance.")]
    NotTokenBalance,
}
