use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{Bank, ANCHOR_DISCRIMINATOR};

#[derive(Accounts)]
pub struct InitBank<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = signer,
        seeds = [b"bank".as_ref(), mint.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Bank::INIT_SPACE
    )]
    bank: Account<'info, Bank>,
    #[account(
        init,
        payer = signer,
        seeds = [b"treasury", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = treasury,
        token::token_program = token_program
    )]
    treasury: InterfaceAccount<'info, TokenAccount>,

    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> InitBank<'info> {
    pub fn init_bank(
        &mut self,
        liquidation_threshold: u64,
        max_ltv: u64,
        bumps: &InitBankBumps,
    ) -> Result<()> {
        self.bank.set_inner(Bank {
            authority: self.signer.key(),
            mint: self.mint.key(),
            total_deposit: 0,
            total_borrowed: 0,
            total_deposit_shares: 0,
            total_borrowed_shares: 0,
            liquidation_threshold,
            liquidation_bonus: 1000,
            liquidation_close_factor: 5000,
            max_ltv,
            last_updated: 0,
            interest_rate: 500,
            treasury_bump: bumps.treasury,
            bump: bumps.bank,
        });
        Ok(())
    }
}
