use crate::state::UserAccount;
use anchor_lang::prelude::*;
use solana_gpt_oracle::Counter;

const AGENT_DESC: &str =
    "You are an AI agent that analyzes crypto market trends. Keep responses brief.";

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 1, // Discriminator + context pubkey + bump
        seeds = [b"user", payer.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: Oracle counter account
    #[account(mut)]
    pub counter: Account<'info, Counter>,

    /// CHECK: Will be initialized by oracle program
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,

    /// CHECK: Verified by address constraint
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    // Store context key and bump
    ctx.accounts.user_account.context = ctx.accounts.llm_context.key();
    ctx.accounts.user_account.bump = ctx.bumps.user_account;

    // CPI to oracle to create the LLM context with our system prompt
    let cpi_program = ctx.accounts.oracle_program.to_account_info();
    let cpi_accounts = solana_gpt_oracle::cpi::accounts::CreateLlmContext {
        payer: ctx.accounts.payer.to_account_info(),
        context_account: ctx.accounts.llm_context.to_account_info(),
        counter: ctx.accounts.counter.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    solana_gpt_oracle::cpi::create_llm_context(cpi_ctx, AGENT_DESC.to_string())?;

    msg!(
        "Agent initialized with LLM context: {}",
        ctx.accounts.llm_context.key()
    );
    Ok(())
}
