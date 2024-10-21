use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{Bank, User};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    liquidator: Signer<'info>,
    collateral_mint: InterfaceAccount<'info, Mint>,
    borrowed_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"bank".as_ref(), collateral_mint.key().as_ref()],
        bump
    )]
    collateral_bank: Account<'info, Bank>,
    #[account(
        mut,
        seeds = [b"bank".as_ref(), borrowed_mint.key().as_ref()],
        bump
    )]
    borrowed_bank: Account<'info, Bank>,
    #[account(
        mut,
        seeds = [b"treasury".as_ref(), collateral_mint.key().as_ref()],
        bump
    )]
    collateral_treasury: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"treasury".as_ref(), borrowed_mint.key().as_ref()],
        bump
    )]
    borrowed_treasury: Account<'info, TokenAccount>,
    price_update: Account<'info, PriceUpdateV2>,
    #[account(
        mut,
        seeds = [b"user", liquidator.key().as_ref()],
        bump
    )]
    liquidator_account: Account<'info, User>,
    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = collateral_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program
    )]
    collateral_user_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = borrowed_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program
    )]
    borrowed_user_ata: InterfaceAccount<'info, TokenAccount>,

    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>
}