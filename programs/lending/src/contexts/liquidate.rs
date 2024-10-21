use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::error::ErrorCode;
use crate::{
    Bank, User, MAX_AGE, SEED_BANK_ACCOUNT, SEED_TREASURY_ACCOUNT, SEED_USER_ACCOUNT,
    SOL_USD_FEED_ID, USDC_USD_FEED_ID,
};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    liquidator: Signer<'info>,
    collateral_mint: InterfaceAccount<'info, Mint>,
    borrowed_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [SEED_BANK_ACCOUNT, collateral_mint.key().as_ref()],
        bump = collateral_bank.bump,
    )]
    collateral_bank: Account<'info, Bank>,
    #[account(
        mut,
        seeds = [SEED_BANK_ACCOUNT, borrowed_bank.key().as_ref()],
        bump = borrowed_bank.bump
    )]
    borrowed_bank: Account<'info, Bank>,
    #[account(
        mut,
        seeds = [SEED_TREASURY_ACCOUNT, collateral_mint.key().as_ref()],
        bump = collateral_bank.treasury_bump
    )]
    collateral_treasury: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [SEED_TREASURY_ACCOUNT, borrowed_mint.key().as_ref()],
        bump = borrowed_bank.treasury_bump
    )]
    borrowed_treasury: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [SEED_USER_ACCOUNT, liquidator.key().as_ref()],
        bump = user.bump
    )]
    user: Account<'info, User>,
    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = collateral_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program
    )]
    liquidator_collateral_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = borrowed_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program
    )]
    liquidator_borrowed_ata: InterfaceAccount<'info, TokenAccount>,
    price_update: Account<'info, PriceUpdateV2>,

    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

impl<'info> Liquidate<'info> {
    pub fn liquidate(&mut self) -> Result<()> {
        let sol_feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;
        let usdc_feed_id = get_feed_id_from_hex(USDC_USD_FEED_ID)?;

        let sol_price =
            self.price_update
                .get_price_no_older_than(&Clock::get()?, MAX_AGE, &sol_feed_id)?;
        let usdc_price =
            self.price_update
                .get_price_no_older_than(&Clock::get()?, MAX_AGE, &usdc_feed_id)?;

        let total_collateral = (sol_price.price as u64 * self.user.deposited_sol)
            + (usdc_price.price as u64 * self.user.deposited_usdc);
        let total_borrowed = (sol_price.price as u64 * self.user.borrowed_sol)
            + (usdc_price.price as u64 * self.user.borrowed_usdc);

        let health_factor =
            (total_collateral * self.collateral_bank.liquidation_threshold) / total_borrowed;

        if health_factor >= 1 {
            return Err(ErrorCode::NotUndercollateralized.into());
        }

        let liquidation_amount = total_borrowed * self.collateral_bank.liquidation_close_factor;

        let transfer_to_bank = TransferChecked {
            from: self.liquidator_borrowed_ata.to_account_info(),
            mint: self.borrowed_mint.to_account_info(),
            to: self.borrowed_bank.to_account_info(),
            authority: self.liquidator.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_to_bank);

        transfer_checked(cpi_ctx, liquidation_amount, self.borrowed_mint.decimals)?;

        // transfer liquidation value and bonus to liquidator
        let liquidation_bonus = (liquidation_amount * self.collateral_bank.liquidation_bonus) + liquidation_amount;
    
        let transfer_to_liquidator = TransferChecked {
            from: self.collateral_bank.to_account_info(),
            mint: self.collateral_mint.to_account_info(),
            to: self.liquidator_collateral_ata.to_account_info(),
            authority: self.collateral_bank.to_account_info()
        };

        let seeds = &[
            &SEED_TREASURY_ACCOUNT[..],
            &self.collateral_bank.mint.as_ref(),
            &[self.collateral_bank.treasury_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_to_liquidator, signer_seeds);

        transfer_checked(cpi_ctx, liquidation_bonus, self.collateral_mint.decimals)
    }
}
