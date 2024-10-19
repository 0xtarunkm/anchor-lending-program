use std::f64::consts::E;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::error::ErrorCode;
use crate::{Bank, User};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let deposited_value: u64;

        if self.mint.to_account_info().key() == self.user.mint_usdc {
            deposited_value = self.user.deposited_usdc;
        } else {
            deposited_value = self.user.deposited_sol;
        }

        let time_diff = self.user.last_updated - Clock::get()?.unix_timestamp;
        self.bank.total_deposit = (self.bank.total_deposit as f64 * E.powf(self.bank.interest_rate as f64 * time_diff as f64)) as u64;

        let value_per_share = self.bank.total_deposit as f64 / self.bank.total_deposit_shares as f64;

        let user_value = (deposited_value as f64 / value_per_share) as u64;

        if user_value < amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        let cpi_accounts = TransferChecked {
            from: self.treasury.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.user_ata.to_account_info(),
            authority: self.treasury.to_account_info(),
        };

        let seeds = &[
            &b"treasury"[..],
            &self.bank.mint.as_ref(),
            &[self.bank.treasury_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        let shares_to_remove = (amount as f64 / self.bank.total_deposit as f64)
            * self.bank.total_deposit_shares as f64;

        match self.mint.to_account_info().key() {
            key if key == self.user.mint_usdc => {
                self.user.deposited_usdc -= amount;
                self.user.deposited_usdc_shares -= shares_to_remove as u64;
            }
            _ => {
                self.user.deposited_sol -= amount;
                self.user.deposited_sol_shares -= shares_to_remove as u64;
            }
        }

        self.bank.total_deposit -= amount;
        self.bank.total_deposit_shares -= shares_to_remove as u64;

        Ok(())
    }
}
