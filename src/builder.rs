use crate::types::{ComputeUnitLimits, PriorityFeeConfig};
use anyhow::Context;
use rand::Rng;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_program::message::{Message, VersionedMessage};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::{pubkey, pubkey::Pubkey};

/// Protocol defined: The default compute units set for a transaction
const DEFAULT_INSTRUCTION_COMPUTE_UNIT: u32 = 200_000;
/// Protocol defined: There are 10^6 micro-lamports in one lamport
const MICRO_LAMPORTS_PER_LAMPORT: u64 = 1_000_000;

#[derive(Default, Clone)]
pub struct SwapInstructionsBuilder {
    pub compute_budget_instructions: Vec<Instruction>,
    pub setup_instructions: Vec<Instruction>,
    pub swap_instruction: Option<Instruction>,
    pub cleanup_instruction: Option<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
}

pub struct UserAssociatedTokenAccounts {
    pub input_ata: Pubkey,
    pub output_ata: Pubkey,
}

impl SwapInstructionsBuilder {
    /// Big todo: How to handle spl vs token-22 tokens here?
    #[allow(clippy::too_many_arguments)]
    pub fn handle_token_wrapping_and_accounts_creation(
        &mut self,
        user: Pubkey,
        wrap_and_unwrap_sol: bool,
        input_amount: u64,
        input_mint: Pubkey,
        output_mint: Pubkey,
        input_token_program: Pubkey,
        output_token_program: Pubkey,
        destination_token_account: Option<Pubkey>,
    ) -> anyhow::Result<UserAssociatedTokenAccounts> {
        let user_input_ata =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &user,
                &input_mint,
                &input_token_program,
            );
        let user_output_ata =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &user,
                &output_mint,
                &output_token_program,
            );

        if input_mint == spl_token::native_mint::ID {
            // Only create an input-ata if it's the native mint
            let create_ata_ix =
                spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                    &user,
                    &user,
                    &input_mint,
                    &spl_token::ID, // SOL uses token-22
                );
            self.setup_instructions.push(create_ata_ix);

            // Only wrap SOL if user specifies this behaviour and the input-token is SOL
            if wrap_and_unwrap_sol {
                let transfer_ix =
                    solana_sdk::system_instruction::transfer(&user, &user_input_ata, input_amount);
                let sync_ix = spl_token::instruction::sync_native(&spl_token::ID, &user_input_ata)
                    .expect("spl_token::ID is valid");
                self.setup_instructions.extend([transfer_ix, sync_ix]);

                let close_ix = spl_token::instruction::close_account(
                    &spl_token::ID,
                    &user_input_ata,
                    &user,
                    &user,
                    &[],
                )
                .expect("spl_token::ID is valid");
                self.cleanup_instruction = Some(close_ix);
            }
        }

        if destination_token_account.is_none() {
            // Only create an ATA if no destination-token-account is specified. If specified, we assume it is
            // already initialized.
            let create_ata_ix =
                spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                    &user,
                    &user,
                    &output_mint,
                    &output_token_program,
                );
            self.setup_instructions.push(create_ata_ix);

            if wrap_and_unwrap_sol && output_mint == spl_token::native_mint::ID {
                self.cleanup_instruction = Some(
                    spl_token::instruction::close_account(
                        &spl_token::ID,
                        &user_output_ata,
                        &user,
                        &user,
                        &[],
                    )
                    .expect("spl_token::ID is valid"),
                )
            }
        }

        Ok(UserAssociatedTokenAccounts {
            input_ata: user_input_ata,
            output_ata: user_output_ata,
        })
    }

    pub fn handle_priority_fee_params(
        &mut self,
        priority_fee_config: Option<PriorityFeeConfig>,
        compute_units: Option<u32>,
        funder: Pubkey,
    ) -> anyhow::Result<()> {
        let compute_units = compute_units.unwrap_or(DEFAULT_INSTRUCTION_COMPUTE_UNIT);
        log::debug!("Prioritization fee config: {priority_fee_config:#?}");
        match priority_fee_config {
            Some(PriorityFeeConfig::FixedCuPrice(cu_price)) => {
                log::trace!("setting user defined cu-price: {}", cu_price);
                let compute_ix =
                    solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(
                        cu_price,
                    );
                self.compute_budget_instructions.push(compute_ix);
            }
            Some(PriorityFeeConfig::DynamicMultiplier(multiplier)) => {
                let priofee = multiplier
                    .checked_mul(100_000)
                    .context("Overflow error while calculating priofee auto-multiplier")?;
                let cu_price = calculate_cu_price(priofee, compute_units);
                log::trace!(
                    "prioritization-fee-lamports: cu-price={}, multiplier={}. priofee={}, cu-limit={}",
                    cu_price,
                    multiplier,
                    priofee,
                    compute_units
                );
                let compute_ix =
                    solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(
                        cu_price,
                    );
                self.compute_budget_instructions.push(compute_ix);
            }
            Some(PriorityFeeConfig::JitoTip(jito_tip)) => {
                let tip_ix = build_jito_tip_ix(&funder, jito_tip);
                self.setup_instructions.push(tip_ix);
            }
            None => {}
        }

        Ok(())
    }

    pub async fn handle_compute_units_params(
        &mut self,
        compute_limits: Option<ComputeUnitLimits>,
        rpc_client: &RpcClient,
        payer: Pubkey,
    ) -> anyhow::Result<Option<u32>> {
        let cu_limit = match compute_limits {
            None => None,
            Some(ComputeUnitLimits::Dynamic) => {
                let simulate_txn = self.clone().build_transaction(Some(&payer), None)?;
                let result = rpc_client
                    .simulate_transaction_with_config(
                        &simulate_txn,
                        RpcSimulateTransactionConfig {
                            sig_verify: false,
                            replace_recent_blockhash: true,
                            commitment: Some(CommitmentConfig::confirmed()),
                            ..Default::default()
                        },
                    )
                    .await?;

                result.value.units_consumed.and_then(|compute_units| {
                    u32::try_from(compute_units).ok()?.checked_add(50_000)
                })
            }
            Some(ComputeUnitLimits::Fixed(cu_limits)) => Some(u32::try_from(cu_limits)?),
        };

        if let Some(cu_limit) = cu_limit {
            self.compute_budget_instructions.push(
                solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(
                    cu_limit,
                ),
            );
        }

        Ok(cu_limit)
    }

    pub fn build_instructions(self) -> anyhow::Result<Vec<Instruction>> {
        let mut final_instructions = Vec::new();
        let SwapInstructionsBuilder {
            compute_budget_instructions,
            setup_instructions,
            swap_instruction,
            cleanup_instruction,
            address_lookup_table_addresses: _,
        } = self;
        final_instructions.extend(compute_budget_instructions);
        final_instructions.extend(setup_instructions);
        final_instructions.push(swap_instruction.context("Swap instruction not set")?);
        if let Some(cleanup_instruction) = cleanup_instruction {
            final_instructions.push(cleanup_instruction);
        }
        Ok(final_instructions)
    }

    pub fn build_transaction(
        self,
        payer: Option<&Pubkey>,
        blockhash: Option<Hash>,
    ) -> anyhow::Result<VersionedTransaction> {
        let instructions = self.build_instructions()?;
        let mut message = VersionedMessage::Legacy(Message::new(&instructions, payer));
        if let Some(hash) = blockhash {
            message.set_recent_blockhash(hash);
        }
        Ok(VersionedTransaction {
            signatures: vec![Signature::default()],
            message,
        })
    }
}

fn calculate_cu_price(priority_fee: u64, compute_units: u32) -> u64 {
    // protocol: priority-fee = cu-price * cu-limit / 1_000_000
    // agave: priority-fee = (cu-price * cu-limit + 999_999) / 1_000_000
    let cu_price = (priority_fee as u128)
        .checked_mul(MICRO_LAMPORTS_PER_LAMPORT as u128)
        .expect("u128 multiplication shouldn't overflow")
        .saturating_sub(MICRO_LAMPORTS_PER_LAMPORT as u128 - 1)
        .checked_div(compute_units as u128 + 1)
        .expect("non-zero compute units");
    log::trace!("cu-price u128: {}", cu_price);
    u64::try_from(cu_price).unwrap_or(u64::MAX)
}

const JITO_TIP_ACCOUNTS: [Pubkey; 8] = [
    pubkey!("96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5"),
    pubkey!("HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe"),
    pubkey!("Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY"),
    pubkey!("ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49"),
    pubkey!("DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh"),
    pubkey!("ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt"),
    pubkey!("DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL"),
    pubkey!("3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT"),
];

fn build_jito_tip_ix(from: &Pubkey, tip: u64) -> Instruction {
    let random_recipient =
        &JITO_TIP_ACCOUNTS[rand::thread_rng().gen_range(0..JITO_TIP_ACCOUNTS.len())];
    solana_sdk::system_instruction::transfer(from, random_recipient, tip)
}
