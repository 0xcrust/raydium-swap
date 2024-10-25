use futures_util::stream::FuturesOrdered;
use futures_util::StreamExt;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;

pub async fn get_multiple_account_data(
    rpc_client: &RpcClient,
    keys: &[Pubkey],
) -> anyhow::Result<Vec<Option<Account>>> {
    let mut tasks = FuturesOrdered::new();
    let mut accounts_vec = Vec::with_capacity(keys.len());
    for chunk in keys.chunks(100) {
        tasks.push_back(async {
            let response = rpc_client
                .get_multiple_accounts_with_config(
                    chunk,
                    RpcAccountInfoConfig {
                        encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                        data_slice: None,
                        commitment: Some(CommitmentConfig::confirmed()),
                        min_context_slot: None,
                    },
                )
                .await?;
            Ok::<_, anyhow::Error>(response.value)
        });
    }

    while let Some(result) = tasks.next().await {
        accounts_vec.extend(result?);
    }
    Ok(accounts_vec)
}
