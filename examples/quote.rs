use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::{pubkey, pubkey::Pubkey};
use std::sync::Arc;
use swap::amm::executor::{RaydiumAmm, RaydiumAmmExecutorOpts};
use swap::api_v3::ApiV3Client;
use swap::types::{SwapExecutionMode, SwapInput};

const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const SOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let client = Arc::new(RpcClient::new(std::env::var("RPC_URL")?));
    let executor = RaydiumAmm::new(
        Arc::clone(&client),
        RaydiumAmmExecutorOpts::default(),
        ApiV3Client::new(None),
    );
    let swap_input = SwapInput {
        input_token_mint: SOL,
        output_token_mint: USDC,
        slippage_bps: 1000,    // 10%
        amount: 1_000_000_000, // 1 SOL
        mode: SwapExecutionMode::ExactIn,
        market: None,
    };

    let quote = executor.quote(&swap_input).await?;
    log::info!("Quote: {:#?}", quote);

    let keypair = Keypair::new();
    let mut transaction = executor
        .swap_transaction(keypair.pubkey(), quote, None)
        .await?;
    let blockhash = client.get_latest_blockhash().await?;
    transaction.message.set_recent_blockhash(blockhash);
    let _final_tx = VersionedTransaction::try_new(transaction.message, &[&keypair])?;

    Ok(())
}
