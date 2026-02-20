use anchor_lang::prelude::*;

#[account]
pub struct UserAccount {
    pub context: Pubkey, // LLM context created during initialize
    pub bump: u8,
}
