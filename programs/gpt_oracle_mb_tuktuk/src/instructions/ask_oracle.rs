use anchor_lang::prelude::*;
use solana_gpt_oracle::cpi::accounts::InteractWithLlm;
use solana_gpt_oracle::program::SolanaGptOracle;
use solana_gpt_oracle::{self, cpi::interact_with_llm};

#[derive(Accounts)]
pub struct AskOracle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: The Oracle creates/manages this account
    #[account(mut)]
    pub interaction: UncheckedAccount<'info>,
    /// CHECK: The pre-initialized context
    pub llm_context: UncheckedAccount<'info>,
    pub oracle_program: Program<'info, SolanaGptOracle>,
    pub system_program: Program<'info, System>,
}

pub fn ask_oracle(ctx: Context<AskOracle>) -> Result<()> {
    let prompt = "Analyze the current Solana trend in 1 sentence.";

    let cpi_ctx = CpiContext::new(
        ctx.accounts.oracle_program.to_account_info(),
        InteractWithLlm {
            payer: ctx.accounts.payer.to_account_info(),
            interaction: ctx.accounts.interaction.to_account_info(),
            context_account: ctx.accounts.llm_context.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        },
    );

    // FIX: Manually calculate the discriminator for "receive_answer"
    // Anchor format: "global:function_name" (snake_case)
    let cb_discriminator = get_sighash("global", "receive_answer");

    interact_with_llm(
        cpi_ctx,
        prompt.to_string(),
        crate::ID,
        cb_discriminator, // Pass the calculated bytes
        None,
    )?;

    msg!("Request sent to Oracle!");
    Ok(())
}

pub fn receive_answer(ctx: Context<ReceiveAnswerContext>, response: String) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    user_account.last_response = response.clone();
    msg!("ORACLE RESPONSE STORED: {}", response);
    emit!(AgentEvent { response });
    Ok(())
}

#[derive(Accounts)]
pub struct ReceiveAnswerContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, crate::state::UserAccount>,
    /// CHECK: Verified by the Oracle program logic usually
    pub identity: Signer<'info>,
}

#[event]
pub struct AgentEvent {
    pub response: String,
}

// --- HELPER FUNCTION ---
// Calculates the 8-byte instruction discriminator manually
pub fn get_sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8]);
    sighash
}
