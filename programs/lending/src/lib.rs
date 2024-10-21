use anchor_lang::prelude::*;

pub mod constants;
pub mod contexts;
pub mod error;
pub mod states;

pub use constants::*;
pub use contexts::*;
pub use states::*;

declare_id!("8iZGbJw7yWA4znvCcnz4VGKhdnzRwGPiU5BjLpV539Kc");

#[program]
pub mod lending {
    use super::*;

    pub fn init_bank(
        ctx: Context<InitBank>,
        liquidation_threshold: u64,
        max_ltv: u64,
    ) -> Result<()> {
        ctx.accounts
            .init_bank(liquidation_threshold, max_ltv, &ctx.bumps)
    }

    pub fn init_user(ctx: Context<InitUser>, mint_usdc: Pubkey) -> Result<()> {
        ctx.accounts.init_user(mint_usdc, &ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        ctx.accounts.borrow(amount)
    }

    pub fn replay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        ctx.accounts.repay(amount)
    }
}
