mod api;
mod utils;

use std::str::FromStr;

use anyhow::{format_err, Result};
use clap::Parser;
use raydium_contract_instructions::amm_instruction as amm;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{EncodableKey as _, Keypair, Signer};
use solana_sdk::transaction::Transaction;

const FEE_PERCENTAGE: f64 = 0.9;
const WRAPPED_SOL: &str = "So11111111111111111111111111111111111111112";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Cluster {
    Mainnet,
    Devnet,
    #[default]
    Localnet,
    Testnet,
    Custom(String),
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Cluster::Mainnet => "mainnet",
            Cluster::Devnet => "devnet",
            Cluster::Localnet => "localnet",
            Cluster::Testnet => "testnet",
            Cluster::Custom(url) => url,
        };
        write!(f, "{}", display)
    }
}

impl std::str::FromStr for Cluster {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Cluster> {
        match s.to_lowercase().as_str() {
            custom_url if custom_url.contains("http") => Ok(Cluster::Custom(s.to_owned())),
            "mainnet" => Ok(Cluster::Mainnet),
            "devnet" => Ok(Cluster::Devnet),
            "localnet" => Ok(Cluster::Localnet),
            "testnet" => Ok(Cluster::Testnet),
            _ => Err(format_err!(
                "Cluster argument must be either of [mainnet], [devnet], [localnet], [testnet], or custom http/https url"
            ))
        }
    }
}

#[derive(Clone, Debug)]
pub enum Trade {
    Buy,
    Sell,
}
impl std::fmt::Display for Trade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trade::Buy => f.write_str("buy"),
            Trade::Sell => f.write_str("sell"),
        }
    }
}
impl std::str::FromStr for Trade {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "b" | "buy" => Ok(Trade::Buy),
            "s" | "sell" => Ok(Trade::Sell),
            _ => Err(anyhow::format_err!(
                "Trade argument must be either [b | buy] or [s | sell]"
            )),
        }
    }
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(default_value = "mainnet")]
    pub cluster: Cluster,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Perform a WSOL swap.
    Swap {
        keypair: String,
        token: Pubkey,
        trade: Trade,
        amount: u64,
        fee_vault: Pubkey,
        pool_cache: Option<String>,
        price_cache: Option<String>,
    },
    /// Simulate a WSOL swap.
    SimulateSwap {
        keypair: String,
        token: Pubkey,
        trade: Trade,
        amount: u64,
        pool_cache: Option<String>,
        price_cache: Option<String>,
    },
    /// Dump pool details from https://api.raydium.io/v2/sdk/liquidity/mainnet.json to `output-file`.
    FetchPools {
        output_file: String,
    },
    FetchPrices {
        output_file: String,
    },
    /// Gets the price of a token in USD.
    GetPriceUSD {
        token: Pubkey,
        price_cache: Option<String>,
    },
    GetPriceSOL {
        token: Pubkey,
        price_cache: Option<String>,
    },
}

fn rpc_client(cluster: Cluster) -> RpcClient {
    RpcClient::new(cluster.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Swap {
            token,
            trade,
            amount,
            keypair,
            fee_vault,
            pool_cache,
            price_cache,
        } => {
            let wsol = Pubkey::new_from_array(WRAPPED_SOL.as_bytes().try_into()?);

            // Get the pool information for the wSOL/other-token pool if it exists.
            let pool_info = api::get_pool_info(&wsol, &token, pool_cache).await?;
            let keypair = Keypair::read_from_file(&keypair)
                .map_err(|_| anyhow::anyhow!("failed reading keypair from path {}", keypair))?;

            let (user_wsol_token_account, create_user_wsol_token_account) =
                utils::create_associated_token_account(&keypair.pubkey(), &keypair.pubkey(), &wsol);
            let (user_other_token_account, create_user_other_token_account) =
                utils::create_associated_token_account(
                    &keypair.pubkey(),
                    &keypair.pubkey(),
                    &token,
                );

            let mut instructions = vec![];
            let client = rpc_client(cli.cluster);

            let (
                //fee_token,
                user_source_account,
                user_destination_account,
                create_destination_account,
            ) = match trade {
                Trade::Buy => (
                    //wsol,
                    user_wsol_token_account,
                    user_other_token_account,
                    create_user_other_token_account,
                ),
                Trade::Sell => (
                    //token,
                    user_other_token_account,
                    user_wsol_token_account,
                    create_user_wsol_token_account,
                ),
            };

            // Create the user's destination account if it doesn't exist.
            if client
                .get_token_account(&user_destination_account)
                .await?
                .is_none()
            {
                instructions.push(create_destination_account);
            }

            // Create the required token account for the fee vault if it doesn't yet exist.
            let (fee_vault_token_account, create_fee_vault_token_account) =
                utils::create_associated_token_account(&keypair.pubkey(), &fee_vault, &wsol);
            if client
                .get_token_account(&fee_vault_token_account)
                .await?
                .is_none()
            {
                instructions.push(create_fee_vault_token_account);
            }

            let wsol_price = api::get_price(&wsol, &price_cache).await?;
            let token_price = api::get_price(&token, &price_cache).await?;

            match trade {
                Trade::Buy => {
                    // Append an instruction to transfer fee tokens to the specified vault.
                    let fee = ((FEE_PERCENTAGE / 100.0) * amount as f64).trunc() as u64;
                    let rest = amount - fee;

                    let wsol_fee_transfer_instruction = spl_token::instruction::transfer(
                        &spl_token::ID,
                        &user_source_account,
                        &fee_vault_token_account,
                        &keypair.pubkey(),
                        &[&keypair.pubkey()],
                        fee,
                    )?;

                    // Pool is wSOL/other. Base token is wSOL. We're providing the base token(wSOL) to get `other-token`.
                    let expected_amount_out = (token_price / wsol_price) * rest as f64;
                    // TODO: We arbitrarily tolerate slippage of < 10%
                    let min_expected_amount_out = (expected_amount_out * 0.9).trunc() as u64;

                    let swap_instruction = amm::swap_base_in(
                        &amm::ID,
                        &Pubkey::from_str(&pool_info.id)?,
                        &Pubkey::from_str(&pool_info.authority)?,
                        &Pubkey::from_str(&pool_info.open_orders)?,
                        &Pubkey::from_str(&pool_info.target_orders)?,
                        &Pubkey::from_str(&pool_info.base_vault)?,
                        &Pubkey::from_str(&pool_info.quote_vault)?,
                        &Pubkey::from_str(&pool_info.market_program_id)?,
                        &Pubkey::from_str(&pool_info.market_id)?,
                        &Pubkey::from_str(&pool_info.market_bids)?,
                        &Pubkey::from_str(&pool_info.market_asks)?,
                        &Pubkey::from_str(&pool_info.market_event_queue)?,
                        &Pubkey::from_str(&pool_info.market_base_vault)?,
                        &Pubkey::from_str(&pool_info.market_quote_vault)?,
                        &Pubkey::from_str(&pool_info.market_authority)?,
                        &user_source_account,
                        &user_destination_account,
                        &keypair.pubkey(),
                        rest,
                        min_expected_amount_out,
                    )?;

                    instructions.push(wsol_fee_transfer_instruction);
                    instructions.push(swap_instruction);
                }
                Trade::Sell => {
                    // Append an instruction to transfer fee tokens to the specified vault.
                    let fee = ((FEE_PERCENTAGE / 100.0) * amount as f64).trunc() as u64;
                    let rest = amount - fee;

                    let wsol_fee_transfer_instruction = spl_token::instruction::transfer(
                        &spl_token::ID,
                        &user_source_account,
                        &fee_vault_token_account,
                        &keypair.pubkey(),
                        &[&keypair.pubkey()],
                        fee,
                    )?;

                    // Pool is wSOL/other. Base token is wSOL. We're providing the base token(wSOL) to get `other-token`.
                    let expected_amount_in = (token_price / wsol_price) * amount as f64;
                    // TODO: We arbitrarily tolerate slippage of < 10%
                    let max_expected_amount_in = (expected_amount_in * 1.1).trunc() as u64;

                    // Pool is wSOL/other. Base token is wSOL. We're getting the base token(wSOL) in return for `other-token`.
                    let swap_instruction = amm::swap_base_out(
                        &amm::ID,
                        &Pubkey::from_str(&pool_info.id)?,
                        &Pubkey::from_str(&pool_info.authority)?,
                        &Pubkey::from_str(&pool_info.open_orders)?,
                        &Pubkey::from_str(&pool_info.target_orders)?,
                        &Pubkey::from_str(&pool_info.base_vault)?,
                        &Pubkey::from_str(&pool_info.quote_vault)?,
                        &Pubkey::from_str(&pool_info.market_program_id)?,
                        &Pubkey::from_str(&pool_info.market_id)?,
                        &Pubkey::from_str(&pool_info.market_bids)?,
                        &Pubkey::from_str(&pool_info.market_asks)?,
                        &Pubkey::from_str(&pool_info.market_event_queue)?,
                        &Pubkey::from_str(&pool_info.market_base_vault)?,
                        &Pubkey::from_str(&pool_info.market_quote_vault)?,
                        &Pubkey::from_str(&pool_info.market_authority)?,
                        &user_source_account,
                        &user_destination_account,
                        &keypair.pubkey(),
                        max_expected_amount_in,
                        rest,
                    )?;

                    instructions.push(swap_instruction);
                    instructions.push(wsol_fee_transfer_instruction);
                }
            }

            let recent_blockhash = client.get_latest_blockhash().await?;
            let transaction = Transaction::new_signed_with_payer(
                &instructions,
                Some(&keypair.pubkey()),
                &vec![&keypair],
                recent_blockhash,
            );

            client
                .send_and_confirm_transaction_with_spinner_and_config(
                    &transaction,
                    CommitmentConfig::confirmed(),
                    RpcSendTransactionConfig {
                        skip_preflight: true,
                        ..RpcSendTransactionConfig::default()
                    },
                )
                .await?;
        }
        Command::FetchPools { output_file } => {
            let output = api::fetch_all_liquidity_pools().await?;
            std::fs::write(output_file, serde_json::to_string_pretty(&output)?)?;
        }
        Command::FetchPrices { output_file } => {
            let output = api::fetch_all_prices().await?;
            std::fs::write(output_file, serde_json::to_string_pretty(&output)?)?;
        }
        Command::GetPriceSOL { token, price_cache } => {
            let sol_price = api::get_price(&Pubkey::from_str(WRAPPED_SOL)?, &price_cache).await?;
            let token_price = api::get_price(&token, &price_cache).await?;
            // TODO: Use log crate.
            println!(
                "The price of token {} is {} SOL.",
                token,
                sol_price / token_price
            );
        }
        Command::GetPriceUSD { token, price_cache } => {
            let price = api::get_price(&token, &price_cache).await?;
            // TODO: Use log crate.
            println!("The price of token {} is {} USD.", token, price);
        }
        _ => unimplemented!(),
    };

    Ok(())
}
