use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("You have Insufficient Funds")]
    InsufficientFunds,
}
