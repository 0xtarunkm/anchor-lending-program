use std::f64::consts::E;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::error::ErrorCode;

use crate::{Bank, User};

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"bank".as_ref(), mint.key().as_ref()],
        bump,
    )]
    bank: Account<'info, Bank>,
    #[account(
        mut,
        seeds = [b"treasury".as_ref(), mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = treasury,
        token::token_program = token_program
    )]
    treasury: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump,
    )]
    user: Account<'info, User>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    user_ata: InterfaceAccount<'info, TokenAccount>,

    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

impl<'info> Repay<'info> {
    pub fn repay(&mut self, amount: u64) -> Result<()> {
        let borrowed_value: u64;

        match self.mint.to_account_info().key() {
            key if key == self.user.mint_usdc => {
                borrowed_value = self.user.borrowed_usdc;
            }
            _ => {
                borrowed_value = self.user.borrowed_sol;
            }
        }

        let time_diff = self.user.last_updated_borrow - Clock::get()?.unix_timestamp;

        self.bank.total_borrowed -= (self.bank.total_borrowed as f64
            * E.powf(self.bank.interest_rate as f64 * time_diff as f64))
            as u64;

        let value_per_share =
            self.bank.total_borrowed as f64 / self.bank.total_borrowed_shares as f64;

        let user_value = borrowed_value / value_per_share as u64;

        if amount > user_value {
            return Err(ErrorCode::OverRepay.into());
        }

        let cpi_accounts = TransferChecked {
            from: self.user_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.treasury.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        let borrowed_ratio = amount.checked_div(self.bank.total_borrowed).unwrap();
        let user_shares = self
            .bank
            .total_borrowed
            .checked_mul(borrowed_ratio)
            .unwrap();

        match self.mint.to_account_info().key() {
            key if key == self.user.mint_usdc => {
                self.user.borrowed_usdc -= amount;
                self.user.borrowed_usdc_shares -= user_shares;
            }
            _ => {
                self.user.borrowed_sol -= amount;
                self.user.borrowed_sol_shares -= user_shares;
            }
        }

        self.bank.total_borrowed -= amount;
        self.bank.total_borrowed_shares -= user_shares;

        Ok(())
    }
}
