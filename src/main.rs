mod api;

use std::sync::Arc;

use anchor_client::Cluster;
use anyhow::anyhow;
use clap::Parser;
use log::{debug, error, info};
use raydium_contract_instructions::amm_instruction as amm;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{EncodableKey as _, Keypair, Signer};
use solana_sdk::transaction::Transaction;
use spl_token_client::client::{ProgramClient, ProgramRpcClient, ProgramRpcClientSendTransaction};
use spl_token_client::token::{Token, TokenError};

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(
        default_value = "mainnet",
        help = "URL for Solana's JSON RPC or moniker (or their first letter): [mainnet-beta,
    testnet, devnet, localhost]"
    )]
    pub cluster: Cluster,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Perform a swap against wSOL.
    Swap {
        #[arg(long, help = "Path to the provider keypair file")]
        keypair: String,
        #[arg(long, help = "Pubkey of the input token mint")]
        in_token: Pubkey,
        #[arg(long, help = "Pubkey of the output token mint")]
        out_token: Pubkey,
        #[arg(long, help = "Amount of input tokens provided by the user for a swap.")]
        amount_in: u64,
        #[arg(long, help = "The (optional) percentage charged as fee on each trade")]
        fee_percentage: Option<f64>,
        #[arg(long, help = "The (optional) vault fee tokens are sent to")]
        fee_vault: Option<Pubkey>,
        #[arg(
            long,
            help = "The (optional) path to the json file that stores information on raydium pools"
        )]
        pool_cache: Option<String>,
        #[arg(
            long,
            help = "The (optional) Path to the file that stores information on token prices"
        )]
        price_cache: Option<String>,
        #[arg(
            default_value = "0.5",
            help = "The (optional) slippage tolerance percentage. Default is 0.5%",
            long
        )]
        slippage: f32,
        #[arg(
            long,
            help = "(Optional) Allow interactions with non-official Raydium pools. Default is false"
        )]
        allow_unofficial: Option<bool>,
    },
    /// TODO: Simulate a WSOL swap
    SimulateSwap {
        #[arg(long, help = "Path to the provider keypair file")]
        keypair: String,
        #[arg(long, help = "Pubkey of the token to simulate a swap for")]
        token: Pubkey,
        #[arg(long, help = "Amount of tokens to buy/sell")]
        amount: u64,
        #[arg(long, help = "Slippage tolerance percentage")]
        slippage: u8,
        #[arg(
            long,
            help = "Optional path to the json file that stores information on raydium pools"
        )]
        pool_cache: Option<String>,
        #[arg(
            long,
            help = "Optional path to the file that stores information on token prices"
        )]
        price_cache: Option<String>,
    },
    /// Dump pool details from `https://api.raydium.io/v2/sdk/liquidity/mainnet.json`.
    FetchPools {
        #[arg(long, help = "Path to output file", default_value = "pools.json")]
        output_file: String,
    },
    /// Fetch token prices from `https://api.raydium.io/v2/main/price`.
    FetchPrices {
        #[arg(long, help = "Path to output file", default_value = "prices.json")]
        output_file: String,
    },
    /// Gets the price of a token in USD.
    GetPriceUSD {
        #[arg(long, help = "Token address")]
        token: Pubkey,
        #[arg(help = "Optional path to the file storing cached information on token prices")]
        price_cache: Option<String>,
    },
    /// Gets the price of a token in SOL.
    GetPriceSOL {
        #[arg(long, help = "Token address")]
        token: Pubkey,
        #[arg(help = "Optional path to the file storing cached information on token prices")]
        price_cache: Option<String>,
    },
}

fn rpc(cluster: Cluster) -> Arc<RpcClient> {
    Arc::new(RpcClient::new(cluster.url().to_string()))
}
fn program_rpc(rpc: Arc<RpcClient>) -> Arc<dyn ProgramClient<ProgramRpcClientSendTransaction>> {
    let program_client: Arc<dyn ProgramClient<ProgramRpcClientSendTransaction>> = Arc::new(
        ProgramRpcClient::new(rpc.clone(), ProgramRpcClientSendTransaction),
    );
    program_client
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Command::Swap {
            keypair,
            in_token,
            out_token,
            amount_in,
            slippage,
            fee_percentage,
            fee_vault,
            pool_cache,
            price_cache,
            allow_unofficial,
        } => {
            debug!("Command::Swap");
            let allow_unofficial = allow_unofficial.unwrap_or(false);
            let path = keypair;
            let keypair = Keypair::read_from_file(&path).map_err(|_| {
                error!("Failed to read keypair from path={}", path);
                anyhow::anyhow!("failed reading keypair from path {}", path)
            })?;
            info!(
                "Read keypair from {} successfully. Address: {}",
                path,
                keypair.pubkey().to_string()
            );
            info!(
                "Input token mint={}. Output token mint={}",
                in_token, out_token
            );
            if in_token == out_token {
                error!("Noop. Input token and output token are the same mint");
                return Ok(());
            }
            let keypair = Arc::new(keypair);
            let user = keypair.pubkey();

            let client = rpc(cli.cluster);
            let program_client = program_rpc(Arc::clone(&client));

            let in_token_client = Token::new(
                Arc::clone(&program_client),
                &spl_token::ID,
                &in_token,
                None,
                Arc::new(keypair_clone(&keypair)),
            );
            let out_token_client = Token::new(
                Arc::clone(&program_client),
                &spl_token::ID,
                &in_token,
                None,
                Arc::new(keypair_clone(&keypair)),
            );

            let pool_info =
                match api::get_pool_info(&in_token, &out_token, pool_cache, allow_unofficial)
                    .await?
                {
                    Some(info) => info,
                    None => {
                        error!(
                            "Failed to find pool in any specified direction for {}/{} pair",
                            in_token, out_token
                        );
                        return Err(anyhow!(
                            "Failed to find pool for {}/{} pair",
                            in_token,
                            out_token
                        ));
                    }
                };
            debug!("Retrieved pool_info={:?}", pool_info);

            // Get the user's ATA. We don't try to create it as it is expected to exist.
            let user_in_token_account = in_token_client.get_associated_token_address(&user);
            debug!("User input-tokens ATA={}", user_in_token_account);
            let user_in_acct = in_token_client
                .get_account_info(&user_in_token_account)
                .await?;

            // TODO: If input tokens is the native mint(wSOL) and the balance is inadequate, attempt to
            // convert SOL to wSOL.
            let balance = user_in_acct.base.amount;
            info!("User input-tokens ATA balance={}", balance);
            if in_token_client.is_native() && balance < amount_in {
                let transfer_amt = amount_in - balance;
                let blockhash = client.get_latest_blockhash().await?;
                let transfer_instruction = solana_sdk::system_instruction::transfer(
                    &user,
                    &user_in_token_account,
                    transfer_amt,
                );
                let sync_instruction =
                    spl_token::instruction::sync_native(&spl_token::ID, &user_in_token_account)?;
                let tx = Transaction::new_signed_with_payer(
                    &[transfer_instruction, sync_instruction],
                    Some(&user),
                    &[&keypair],
                    blockhash,
                );
                client.send_and_confirm_transaction(&tx).await.unwrap();
            }

            // Create the user's out-token ATA if it doesn't exist.
            let user_out_token_account = out_token_client.get_associated_token_address(&user);
            debug!("User's output-tokens ATA={}", user_out_token_account);
            match out_token_client
                .get_account_info(&user_out_token_account)
                .await
            {
                Ok(_) => debug!("User's ATA for output tokens exists. Skipping creation.."),
                Err(TokenError::AccountNotFound) | Err(TokenError::AccountInvalidOwner) => {
                    info!("User's output-tokens ATA does not exist. Creating..");
                    out_token_client
                        .create_associated_token_account(&user)
                        .await?;
                }
                Err(error) => error!("Error retrieving user's output-tokens ATA: {}", error),
            }

            // If a fee recipient is specified then setup its token account to receive fee tokens(create if needed).
            // Fee tokens are always paid in the input token.
            let mut fee_vault_token_account = None;
            if let Some(vault_key) = fee_vault {
                let vault_in_token_account =
                    in_token_client.get_associated_token_address(&vault_key);
                debug!("Vault's input-token ATA={}", vault_in_token_account);
                match in_token_client
                    .get_account_info(&vault_in_token_account)
                    .await
                {
                    Ok(_) => debug!("Vault ATA for input tokens exists. Skipping creation.."),
                    Err(TokenError::AccountNotFound) | Err(TokenError::AccountInvalidOwner) => {
                        info!("Vault's input-tokens ATA does not exist. Creating..");
                        in_token_client
                            .create_associated_token_account(&vault_key)
                            .await?;
                    }
                    Err(error) => error!("Error retrieving vault's input-tokens ATA: {}", error),
                }
                fee_vault_token_account = Some(vault_in_token_account)
            }

            /*let in_token_price = match api::get_price(&in_token, &price_cache).await? {
                Some(price) => price,
                None => {
                    error!("Failed to find price for token {}", in_token);
                    return Err(anyhow!("Failed to find price for token {}", in_token));
                }
            };
            let out_token_price = match api::get_price(&out_token, &price_cache).await? {
                Some(price) => price,
                None => {
                    error!("Failed to find price for token {}", out_token);
                    return Err(anyhow!("Failed to find price for token {}", out_token));
                }
            };*/
            let in_token_price = api::get_price(&in_token, &price_cache).await?;
            let out_token_price = api::get_price(&out_token, &price_cache).await?;
            info!("Current price of 1 input token={} USD", in_token_price);
            info!("Current price of 1 output token={} USD", out_token_price);

            let mut instructions = vec![];
            // If both a fee-vault and a fee-percentage are specified then split off and transfer fees.
            // Fees are always paid in `input_tokens`.
            let mut swap_amount_in = amount_in;
            if fee_vault_token_account.is_some() && fee_percentage.is_some() {
                let vault_token_account = fee_vault_token_account.expect("option is some");
                let percent = fee_percentage.expect("option is some");
                let fee_vault = fee_vault.expect("fee-vault was specified");

                if percent >= 100.0 {
                    return Err(anyhow::anyhow!("Invalid percentage"));
                }
                let fee = ((percent / 100.0) * amount_in as f64).trunc() as u64;
                swap_amount_in -= fee;
                // Append instruction to transfer fees to fee vault.
                let fee_transfer_instruction = spl_token::instruction::transfer(
                    &spl_token::ID,
                    &user_in_token_account,
                    &vault_token_account,
                    &user,
                    &[&user],
                    fee,
                )?;
                log::info!("Appending fee-transfer instruction. Fee-percentage={}, Fee-amount={}. Fee-vault-owner={}. Fee-vault-ata={}", percent, fee, fee_vault, vault_token_account);
                instructions.push(fee_transfer_instruction);
            }

            /*
            If 1 wSOL costs X USD &
               1 token costs Y USD

            I want to find how much SOL I'd pay for 1 token.
            1 sol -> x usd
            _     -> y usd(1 token)
            I'd need to pay y/x SOL for 1 token.
            (e.g if wSOL is 100 usd and token is 50 usd, then I'd need to pay 50/100 SOL for 1 token)

            I want to find how much tokens I'd pay for 1 SOL.
            1 token -> y usd
            _       -> x usd(1 sol)
            I'd need to pay x/y tokens for 1 SOL.
            */
            let in_out_rate = out_token_price / in_token_price;
            //let in_out_rate = in_token_price / out_token_price;
            //let expected_output_amt = in_out_rate * swap_amount_in as f64;
            let expected_output_amt = swap_amount_in as f64 / in_out_rate;
            if slippage > 100.0 {
                error!("Invalid slippage percentage. > 100");
                return Err(anyhow!("Invalid slippage percentage. >100"));
            }
            let out_factor = ((100.0 - slippage) / 100.0) as f64;
            //let min_expected_out = (expected_output_amt * out_factor).trunc() as u64;
            let decimals = in_token_client.get_mint_info().await?.base.decimals;
            info!("decimals: {}", decimals);
            let min_expected_out = (expected_output_amt * out_factor * decimals as f64);
            info!("min_expected_out ={}", min_expected_out);
            debug!("out_factor={}", out_factor);
            info!(
                "Initiating swap of {} input tokens for {} output. Rate={} input-tokens/1 output-token",
                swap_amount_in, min_expected_out, in_out_rate
            );
            if pool_info.base_mint == in_token {
                info!("Initializing swap with input tokens as pool base token");
                debug_assert!(pool_info.quote_mint == out_token);
                let swap_instruction = amm::swap_base_in(
                    &amm::ID,
                    &pool_info.id,
                    &pool_info.authority,
                    &pool_info.open_orders,
                    &pool_info.target_orders,
                    &pool_info.base_vault,
                    &pool_info.quote_vault,
                    &pool_info.market_program_id,
                    &pool_info.market_id,
                    &pool_info.market_bids,
                    &pool_info.market_asks,
                    &pool_info.market_event_queue,
                    &pool_info.market_base_vault,
                    &pool_info.market_quote_vault,
                    &pool_info.market_authority,
                    &user_in_token_account,
                    &user_out_token_account,
                    &user,
                    swap_amount_in,
                    min_expected_out as u64,
                )?;
                instructions.push(swap_instruction);
            } else {
                info!("Initializing swap with input tokens as pool quote token");
                debug_assert!(pool_info.quote_mint == in_token && pool_info.base_mint == out_token);
                let swap_instruction = amm::swap_base_out(
                    &amm::ID,
                    &pool_info.id,
                    &pool_info.authority,
                    &pool_info.open_orders,
                    &pool_info.target_orders,
                    &pool_info.base_vault,
                    &pool_info.quote_vault,
                    &pool_info.market_program_id,
                    &pool_info.market_id,
                    &pool_info.market_bids,
                    &pool_info.market_asks,
                    &pool_info.market_event_queue,
                    &pool_info.market_base_vault,
                    &pool_info.market_quote_vault,
                    &pool_info.market_authority,
                    &user_in_token_account,
                    &user_out_token_account,
                    &user,
                    swap_amount_in,
                    min_expected_out as u64,
                )?;
                instructions.push(swap_instruction);
            }

            let recent_blockhash = client.get_latest_blockhash().await?;
            let transaction = Transaction::new_signed_with_payer(
                &instructions,
                Some(&user),
                &vec![&keypair],
                recent_blockhash,
            );

            if let Err(e) = client
                .send_and_confirm_transaction_with_spinner_and_config(
                    &transaction,
                    CommitmentConfig::confirmed(),
                    RpcSendTransactionConfig {
                        skip_preflight: true,
                        ..RpcSendTransactionConfig::default()
                    },
                )
                .await
            {
                info!("{e}");
            };
        }
        Command::FetchPools { output_file } => {
            debug!("Command::FetchPools");
            let output = api::fetch_all_liquidity_pools().await?;
            std::fs::write(output_file, serde_json::to_string_pretty(&output)?)?;
        }
        Command::FetchPrices { output_file } => {
            debug!("Command::FetchPrices");
            let output = api::fetch_all_prices().await?;
            std::fs::write(output_file, serde_json::to_string_pretty(&output)?)?;
        }
        Command::GetPriceSOL { token, price_cache } => {
            debug!("Command::GetPriceSOL");
            let sol_price = api::get_price(&spl_token::native_mint::ID, &price_cache).await?;
            let token_price = api::get_price(&token, &price_cache).await?;
            println!(
                "The price of token {} is {} SOL.",
                token,
                token_price / sol_price
            );
        }
        Command::GetPriceUSD { token, price_cache } => {
            debug!("Command::GetPriceUSD");
            let price = api::get_price(&token, &price_cache).await?;
            println!("The price of token {} is {} USD.", token, price);
        }
        _ => unimplemented!(),
    };

    Ok(())
}

fn keypair_clone(kp: &Keypair) -> Keypair {
    Keypair::from_bytes(&kp.to_bytes()).expect("failed to copy keypair")
}
// Getting a 42 error(0x2A).
