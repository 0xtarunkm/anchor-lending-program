use std::f64::consts::E;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{
    error::ErrorCode, Bank, User, MAX_AGE, SEED_BANK_ACCOUNT, SEED_TREASURY_ACCOUNT,
    SEED_USER_ACCOUNT, SOL_USD_FEED_ID, USDC_USD_FEED_ID,
};

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [SEED_BANK_ACCOUNT, mint.key().as_ref()],
        bump = bank.bump,
    )]
    bank: Account<'info, Bank>,
    #[account(
        mut,
        seeds = [SEED_TREASURY_ACCOUNT, mint.key().as_ref()],
        bump = bank.treasury_bump,
        token::mint = mint,
        token::authority = treasury,
        token::token_program = token_program
    )]
    treasury: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [SEED_USER_ACCOUNT, signer.key().as_ref()],
        bump = user.bump,
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
    price_update: Account<'info, PriceUpdateV2>,

    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

impl<'info> Borrow<'info> {
    pub fn borrow(&mut self, amount: u64) -> Result<()> {
        let total_collateral: u64;

        match self.mint.to_account_info().key() {
            key if key == self.user.mint_usdc => {
                let sol_feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;
                let sol_price = self.price_update.get_price_no_older_than(
                    &Clock::get()?,
                    MAX_AGE,
                    &sol_feed_id,
                )?;
                let new_value = self.calculate_accrued_interest(
                    self.user.deposited_sol,
                    self.bank.interest_rate,
                    self.user.last_updated,
                )?;
                total_collateral = sol_price.price as u64 * new_value;
            }
            _ => {
                let usdc_feed_id = get_feed_id_from_hex(USDC_USD_FEED_ID)?;
                let usdc_price = self.price_update.get_price_no_older_than(
                    &Clock::get()?,
                    MAX_AGE,
                    &usdc_feed_id,
                )?;
                let new_value = self.calculate_accrued_interest(
                    self.user.deposited_usdc,
                    self.bank.interest_rate,
                    self.user.last_updated,
                )?;
                total_collateral = usdc_price.price as u64 * new_value;
            }
        }

        let borrowable_amount = total_collateral
            .checked_mul(self.bank.liquidation_threshold)
            .unwrap();

        if borrowable_amount < amount {
            return Err(ErrorCode::OverBorrowableAmount.into());
        }

        let cpi_accounts = TransferChecked {
            from: self.treasury.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.user_ata.to_account_info(),
            authority: self.treasury.to_account_info(),
        };

        let seeds = &[
            &SEED_TREASURY_ACCOUNT[..],
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

        if self.bank.total_borrowed == 0 {
            self.bank.total_borrowed = amount;
            self.bank.total_borrowed_shares = amount;
        }

        let borrowed_ratio = amount.checked_div(self.bank.total_borrowed).unwrap();
        let user_shares = self
            .bank
            .total_borrowed_shares
            .checked_mul(borrowed_ratio)
            .unwrap();

        match self.mint.to_account_info().key() {
            key if key == self.user.mint_usdc => {
                self.user.borrowed_usdc += amount;
                self.user.borrowed_usdc_shares += user_shares;
            }
            _ => {
                self.user.borrowed_sol += amount;
                self.user.borrowed_sol_shares += user_shares;
            }
        }

        self.user.last_updated_borrow = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn calculate_accrued_interest(
        &mut self,
        deposited: u64,
        interest_rate: u64,
        last_updated: i64,
    ) -> Result<u64> {
        let current_time = Clock::get()?.unix_timestamp;
        let time_diff = current_time - last_updated;
        let new_value = (deposited as f64 * E.powf(interest_rate as f64 * time_diff as f64)) as u64;

        Ok(new_value)
    }
}
