use crate::state::UserAccount;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        close = payer,
        seeds = [b"user", payer.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, UserAccount>,
}

pub fn close_user(_ctx: Context<CloseUser>) -> Result<()> {
    msg!("UserAccount closed.");
    Ok(())
}
