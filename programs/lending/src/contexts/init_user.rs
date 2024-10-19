use anchor_lang::prelude::*;

use crate::{User, ANCHOR_DISCRIMINATOR};

#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        init,
        payer = user,
        seeds = [b"user", signer.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + User::INIT_SPACE
    )]
    user: Account<'info, User>,

    system_program: Program<'info, System>,
}

impl<'info> InitUser<'info> {
    pub fn init_user(&mut self, mint_usdc: Pubkey) -> Result<()> {
        self.user.set_inner(User {
            owner: self.signer.key(),
            deposited_sol: 0,
            deposited_sol_shares: 0,
            borrowed_sol: 0,
            borrowed_sol_shares: 0,
            deposited_usdc: 0,
            deposited_usdc_shares: 0,
            borrowed_usdc: 0,
            borrowed_usdc_shares: 0,
            mint_usdc: mint_usdc,
            last_updated: 0,
            last_updated_borrow: 0,
        });

        Ok(())
    }
}
