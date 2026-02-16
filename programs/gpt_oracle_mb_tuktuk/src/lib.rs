#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

mod instructions;
mod state;

use instructions::*;

declare_id!("2RuRYnQYofQLhnCJ3Ywo9atAD7xiiCK3MkaHR8ryYoSC");

#[ephemeral]
#[program]
pub mod gpt_oracle_mb_tuktuk {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn ask_oracle(ctx: Context<AskOracle>) -> Result<()> {
        instructions::ask_oracle(ctx)
    }

    pub fn receive_answer(ctx: Context<ReceiveAnswerContext>, response: String) -> Result<()> {
        instructions::receive_answer(ctx, response)
    }

    pub fn delegate(ctx: Context<DelegateUser>) -> Result<()> {
        instructions::delegate_user(ctx)
    }

    pub fn undelegate(ctx: Context<UndelegateUser>) -> Result<()> {
        instructions::undelegate_user(ctx)
    }

    pub fn schedule<'info>(
        ctx: Context<'_, '_, 'info, 'info, Schedule<'info>>,
        task_id: u16,
        compiled_tx: CompiledTransactionArg,
    ) -> Result<()> {
        ctx.accounts
            .schedule(task_id, compiled_tx, ctx.bumps, ctx.remaining_accounts)
    }

    // Required for Ephemeral Rollups state updates
    pub fn update_commit(_ctx: Context<UpdateCommit>) -> Result<()> {
        Ok(())
    }
}

// Helper context for update_commit (standard pattern for ER)
#[derive(Accounts)]
pub struct UpdateCommit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
}
