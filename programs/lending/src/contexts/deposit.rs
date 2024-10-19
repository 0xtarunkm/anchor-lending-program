use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{Bank, User};

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    user_ata: InterfaceAccount<'info, TokenAccount>,

    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.user_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.treasury.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        if self.bank.total_deposit == 0 {
            self.bank.total_deposit = amount;
            self.bank.total_deposit_shares = amount;
        }

        let deposit_ratio = amount.checked_div(self.bank.total_deposit).unwrap();
        let user_shares = self
            .bank
            .total_deposit_shares
            .checked_mul(deposit_ratio)
            .unwrap();

        match self.mint.to_account_info().key() {
            key if key == self.user.mint_usdc => {
                self.user.deposited_usdc += amount;
                self.user.deposited_usdc_shares += user_shares;
            }
            _ => {
                self.user.deposited_sol += amount;
                self.user.deposited_sol_shares += user_shares;
            }
        }

        self.bank.total_deposit.checked_add(amount).unwrap();
        self.bank
            .total_deposit_shares
            .checked_add(user_shares)
            .unwrap();

        Ok(())
    }
}
