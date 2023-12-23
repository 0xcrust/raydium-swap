use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

pub const RAYDIUM_POOL_INFO_ENDPOINT: &str = "https://api.raydium.io/v2/sdk/liquidity/mainnet.json";
pub const RAYDIUM_PRICE_INFO_ENDPOINT: &str = "https://api.raydium.io/v2/main/price";

#[derive(Debug, Deserialize, Serialize)]
pub struct LiqPoolInformation {
    pub official: Vec<LiquidityPool>,
    #[serde(rename = "unOfficial")]
    pub unofficial: Vec<LiquidityPool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityPool {
    pub id: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub lp_mint: String,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub lp_decimals: u8,
    pub version: u8,
    pub program_id: String,
    pub authority: String,
    pub open_orders: String,
    pub target_orders: String,
    pub base_vault: String,
    pub quote_vault: String,
    pub withdraw_queue: String,
    pub lp_vault: String,
    pub market_version: u8,
    pub market_program_id: String,
    pub market_id: String,
    pub market_authority: String,
    pub market_base_vault: String,
    pub market_quote_vault: String,
    pub market_bids: String,
    pub market_asks: String,
    pub market_event_queue: String,
}

/// Make a call to the raydium api endpoint to retrieve all liquidity pools.
pub async fn fetch_all_liquidity_pools() -> anyhow::Result<LiqPoolInformation> {
    Ok(reqwest::get(RAYDIUM_POOL_INFO_ENDPOINT)
        .await?
        .json()
        .await?)
}

pub async fn fetch_all_prices() -> anyhow::Result<HashMap<String, f64>> {
    let price_info_result: serde_json::Value = reqwest::get(RAYDIUM_PRICE_INFO_ENDPOINT)
        .await?
        .json()
        .await?;
    deserialize_price_info(price_info_result)
}
fn deserialize_price_info(value: serde_json::Value) -> anyhow::Result<HashMap<String, f64>> {
    Ok(value
        .as_object()
        .ok_or(anyhow::format_err!("malformed content. expected object."))?
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.as_f64().expect("value is f64")))
        .collect())
}

/// Retrieve pool information for a particular token pair. Cache path is an optional path to the already dumped json data
/// from the raydium api.
pub async fn get_pool_info(
    token_a: &Pubkey,
    token_b: &Pubkey,
    cache_path: Option<String>,
) -> anyhow::Result<LiquidityPool> {
    let pools = if let Some(path) = cache_path {
        serde_json::from_str(&std::fs::read_to_string(path)?)?
    } else {
        fetch_all_liquidity_pools().await?
    };

    pools
        .official
        .into_iter()
        .find(|pool| {
            pool.base_mint == token_a.to_string() && pool.quote_mint == token_b.to_string()
        })
        .ok_or(anyhow::format_err!(
            "failed to find
    liquidity pool for pair {}/{}",
            token_a,
            token_b
        ))
}

pub async fn get_price(token: &Pubkey, cache_path: &Option<String>) -> anyhow::Result<f64> {
    let pools = if let Some(path) = cache_path {
        deserialize_price_info(serde_json::from_str(&std::fs::read_to_string(path)?)?)?
    } else {
        fetch_all_prices().await?
    };

    pools
        .into_iter()
        .find_map(|(tok, price)| {
            if token.to_string() == *tok {
                return Some(price);
            }
            None
        })
        .ok_or(anyhow::format_err!(
            "failed to find price for token {}",
            token
        ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_fetch_liquidity_pools() {
        _ = fetch_all_liquidity_pools().await.unwrap();
    }

    #[tokio::test]
    async fn test_fetch_prices() {
        _ = fetch_all_prices().await.unwrap();
    }
}
