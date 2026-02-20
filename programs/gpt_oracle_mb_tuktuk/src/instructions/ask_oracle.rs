use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use solana_gpt_oracle::{ContextAccount, Identity};

use crate::state::UserAccount;

const AGENT_PROMPT: &str = "Analyze the current btc trend in 1 sentence.";

#[derive(Accounts)]
pub struct AskOracle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Created/managed by the oracle program
    #[account(mut)]
    pub interaction: AccountInfo<'info>,

    /// CHECK: The pre-initialized LLM context (stored in user_account)
    #[account(address = user_account.context)]
    pub llm_context: Account<'info, ContextAccount>,

    #[account(
        seeds = [b"user", payer.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: Verified by address constraint
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn ask_oracle(ctx: Context<AskOracle>) -> Result<()> {
    let cpi_program = ctx.accounts.oracle_program.to_account_info();
    let cpi_accounts = solana_gpt_oracle::cpi::accounts::InteractWithLlm {
        payer: ctx.accounts.payer.to_account_info(),
        interaction: ctx.accounts.interaction.to_account_info(),
        context_account: ctx.accounts.llm_context.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Use the discriminator of our callback instruction so the oracle knows where to call back
    let disc: [u8; 8] = crate::instruction::ReceiveAnswer::DISCRIMINATOR
        .try_into()
        .expect("Discriminator must be 8 bytes");

    solana_gpt_oracle::cpi::interact_with_llm(
        cpi_ctx,
        AGENT_PROMPT.to_string(),
        crate::ID,
        disc,
        None,
    )?;

    msg!("Oracle request sent via CPI!");
    Ok(())
}

// Callback handler — called by the oracle's off-chain agent after GPT responds
pub fn receive_answer(ctx: Context<ReceiveAnswerContext>, response: String) -> Result<()> {
    // Verify the oracle identity is a signer (proves callback came from oracle)
    if !ctx.accounts.identity.to_account_info().is_signer {
        return Err(ProgramError::InvalidAccountData.into());
    }

    msg!("Response: {:?}", response);
    Ok(())
}

#[derive(Accounts)]
pub struct ReceiveAnswerContext<'info> {
    pub identity: Account<'info, Identity>,
}
