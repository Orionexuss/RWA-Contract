use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The token account does not have the required token balance.")]
    NotTokenBalance,

    #[msg("The choice provided is invalid.")]
    InvalidChoice,

    #[msg("The voter has no voting power.")]
    NoVotingPower,

    #[msg("The voter does not hold any shares.")]
    NoShares,

    #[msg("Arithmetic operation resulted in an overflow.")]
    Overflow,

    #[msg("The token account has insufficient balance for the operation.")]
    InsuficientTokenBalance,
}
