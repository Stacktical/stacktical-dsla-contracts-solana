use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::events::StakedUserSideEvent;
use crate::state::sla::Sla;
use crate::utils::*;

#[derive(Accounts)]
pub struct StakeUserSide<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub sla: Account<'info, Sla>,

    /// The token account with the tokens to be staked
    #[account(mut)]
    pub staker: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [USER_VAULT_SEED.as_bytes(), sla.key().as_ref()],
        token::mint = sla.mint_address,
        token::authority = sla,
        bump,
    )]
    pub user_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            UT_MINT_SEED.as_bytes(),
            sla.key().as_ref(),
        ],
        bump,
    )]
    pub ut_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

impl<'info> StakeUserSide<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.staker.to_account_info(),
                to: self.user_vault.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }
    fn mint_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.user.to_account_info(),
                mint: self.ut_mint.to_account_info(),
                authority: self.sla.to_account_info(),
            },
        )
    }
}

pub fn handler(ctx: Context<StakeUserSide>, token_amount: u64) -> Result<()> {
    token::transfer(ctx.accounts.transfer_context(), token_amount)?;

    token::mint_to(ctx.accounts.mint_context(), token_amount)?;

    emit!(StakedUserSideEvent { token_amount });
    Ok(())
}
