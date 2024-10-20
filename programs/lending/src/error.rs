use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("You have Insufficient Funds")]
    InsufficientFunds,
    #[msg("Over Borrowable Amount")]
    OverBorrowableAmount,
    #[msg("Over Repay Amount")]
    OverRepay,
    #[msg("User is not undercollateralized.")]
    NotUndercollateralized,
}
