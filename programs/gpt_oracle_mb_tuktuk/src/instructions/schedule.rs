use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke_signed;
use borsh::BorshSerialize;

use crate::state::*;

// TukTuk program ID
const TUKTUK_PROGRAM_ID: Pubkey =
    anchor_lang::pubkey!("tuktukUrfhXT6ZT77QTU8RQtvgL967uRuVagWF57zVA");

// ─── Args types matching TukTuk's on-chain Borsh format ─────────────────────

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CompiledInstructionArg {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CompiledTransactionArg {
    pub num_rw_signers: u8,
    pub num_ro_signers: u8,
    pub num_rw: u8,
    pub accounts: Vec<Pubkey>,
    pub instructions: Vec<CompiledInstructionArg>,
    pub signer_seeds: Vec<Vec<Vec<u8>>>,
}

/// Matches TukTuk TriggerV0 enum
#[derive(BorshSerialize)]
pub enum TriggerArg {
    Now,
    #[allow(dead_code)]
    Timestamp(i64),
}

/// Matches TukTuk TransactionSourceV0 enum
#[derive(BorshSerialize)]
pub enum TransactionSourceArg {
    CompiledV0(CompiledTransactionBorsh),
}

/// Borsh-only version for manual serialization in the CPI data
/// Field order MUST match TukTuk's compiledTransactionV0 IDL
#[derive(BorshSerialize)]
pub struct CompiledTransactionBorsh {
    pub num_rw_signers: u8,
    pub num_ro_signers: u8,
    pub num_rw: u8,
    pub accounts: Vec<Pubkey>,
    pub instructions: Vec<CompiledInstructionBorsh>,
    pub signer_seeds: Vec<Vec<Vec<u8>>>,
}

#[derive(BorshSerialize)]
pub struct CompiledInstructionBorsh {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

/// Full args for queue_task_v0 CPI — field order MUST match TukTuk's QueueTaskArgsV0
#[derive(BorshSerialize)]
pub struct QueueTaskCpiArgs {
    pub id: u16,
    pub trigger: TriggerArg,
    pub transaction: TransactionSourceArg,
    pub crank_reward: Option<u64>,
    pub free_tasks: u8,
    pub description: String,
}

// ─── Accounts ───────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct Schedule<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: TukTuk task queue — validated by tuktuk program CPI
    #[account(mut)]
    pub task_queue: UncheckedAccount<'info>,

    /// CHECK: TukTuk task queue authority — validated by tuktuk program CPI
    pub task_queue_authority: UncheckedAccount<'info>,

    /// CHECK: TukTuk task account — initialized by CPI
    #[account(mut)]
    pub task: UncheckedAccount<'info>,

    /// CHECK: PDA signer for queue authority
    #[account(
        mut,
        seeds = [b"queue_authority"],
        bump
    )]
    pub queue_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: TukTuk program validated by address
    #[account(address = TUKTUK_PROGRAM_ID)]
    pub tuktuk_program: AccountInfo<'info>,
}

// ─── Handler ────────────────────────────────────────────────────────────────

impl<'info> Schedule<'info> {
    /// Schedule a pre-compiled task via CPI into TukTuk's queue_task_v0.
    pub fn schedule(
        &self,
        task_id: u16,
        compiled_tx: CompiledTransactionArg,
        bumps: ScheduleBumps,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<()> {
        // Convert from Anchor-deserialized arg type to Borsh-serializable CPI type
        let compiled_borsh = CompiledTransactionBorsh {
            num_rw_signers: compiled_tx.num_rw_signers,
            num_ro_signers: compiled_tx.num_ro_signers,
            num_rw: compiled_tx.num_rw,
            accounts: compiled_tx.accounts,
            instructions: compiled_tx
                .instructions
                .into_iter()
                .map(|ix| CompiledInstructionBorsh {
                    program_id_index: ix.program_id_index,
                    accounts: ix.accounts,
                    data: ix.data,
                })
                .collect(),
            signer_seeds: compiled_tx.signer_seeds,
        };

        let args = QueueTaskCpiArgs {
            id: task_id,
            trigger: TriggerArg::Now,
            transaction: TransactionSourceArg::CompiledV0(compiled_borsh),
            crank_reward: Some(1_000_001),
            free_tasks: 0,
            description: "gpt_oracle_task".to_string(), // Customized description
        };

        // Discriminator for "global:queue_task_v0"
        // sha256("global:queue_task_v0")[0..8] = 0xb15fc3fcf102b258
        const QUEUE_TASK_DISC: [u8; 8] = [0xb1, 0x5f, 0xc3, 0xfc, 0xf1, 0x02, 0xb2, 0x58];
        let mut data = QUEUE_TASK_DISC.to_vec();
        args.serialize(&mut data)
            .map_err(|_| error!(ErrorCode::ConstraintRaw))?;

        // Account metas matching QueueTaskV0 layout:
        // payer, queue_authority, task_queue_authority, task_queue, task, system_program
        // + remaining_accounts (TukTuk extends compiled_tx.accounts from these)
        let mut account_metas = vec![
            AccountMeta::new(self.user.key(), true),
            AccountMeta::new_readonly(self.queue_authority.key(), true),
            AccountMeta::new_readonly(self.task_queue_authority.key(), false),
            AccountMeta::new(self.task_queue.key(), false),
            AccountMeta::new(self.task.key(), false),
            AccountMeta::new_readonly(self.system_program.key(), false),
        ];
        for acc in remaining_accounts {
            account_metas.push(AccountMeta {
                pubkey: acc.key(),
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            });
        }

        let ix = Instruction {
            program_id: TUKTUK_PROGRAM_ID,
            accounts: account_metas,
            data,
        };

        // Build account infos for CPI (same order as metas + remaining)
        let mut account_infos = vec![
            self.user.to_account_info(),
            self.queue_authority.to_account_info(),
            self.task_queue_authority.to_account_info(),
            self.task_queue.to_account_info(),
            self.task.to_account_info(),
            self.system_program.to_account_info(),
            self.tuktuk_program.to_account_info(),
        ];
        for acc in remaining_accounts {
            account_infos.push(acc.clone());
        }

        // CPI with PDA signer
        invoke_signed(
            &ix,
            &account_infos,
            &[&[b"queue_authority", &[bumps.queue_authority]]],
        )?;

        emit!(TaskScheduledEvent {
            user: self.user.key(),
            task_queue: self.task_queue.key(),
            task_id,
        });

        Ok(())
    }
}

#[event]
pub struct TaskScheduledEvent {
    pub user: Pubkey,
    pub task_queue: Pubkey,
    pub task_id: u16,
}
