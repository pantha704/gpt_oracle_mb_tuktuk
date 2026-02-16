use crate::state::UserAccount;
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::delegate, cpi::DelegateConfig};

#[delegate]
#[derive(Accounts)]
pub struct DelegateUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        del,
        seeds = [b"user", payer.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub validator: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn delegate_user(ctx: Context<DelegateUser>) -> Result<()> {
    let pda_seeds: &[&[u8]] = &[b"user", ctx.accounts.payer.key.as_ref()];

    ctx.accounts.delegate_user_account(
        &ctx.accounts.payer,
        pda_seeds,
        DelegateConfig {
            validator: Some(ctx.accounts.validator.key()),
            ..DelegateConfig::default()
        },
    )?;

    Ok(())
}
