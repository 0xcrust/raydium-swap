use std::collections::HashMap;

use log::{debug, error, info};
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
    #[serde(with = "pubkey")]
    pub id: Pubkey,
    #[serde(with = "pubkey")]
    pub base_mint: Pubkey,
    #[serde(with = "pubkey")]
    pub quote_mint: Pubkey,
    #[serde(with = "pubkey")]
    pub lp_mint: Pubkey,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub lp_decimals: u8,
    pub version: u8,
    #[serde(with = "pubkey")]
    pub program_id: Pubkey,
    #[serde(with = "pubkey")]
    pub authority: Pubkey,
    #[serde(with = "pubkey")]
    pub open_orders: Pubkey,
    #[serde(with = "pubkey")]
    pub target_orders: Pubkey,
    #[serde(with = "pubkey")]
    pub base_vault: Pubkey,
    #[serde(with = "pubkey")]
    pub quote_vault: Pubkey,
    #[serde(with = "pubkey")]
    pub withdraw_queue: Pubkey,
    #[serde(with = "pubkey")]
    pub lp_vault: Pubkey,
    pub market_version: u8,
    #[serde(with = "pubkey")]
    pub market_program_id: Pubkey,
    #[serde(with = "pubkey")]
    pub market_id: Pubkey,
    #[serde(with = "pubkey")]
    pub market_authority: Pubkey,
    #[serde(with = "pubkey")]
    pub market_base_vault: Pubkey,
    #[serde(with = "pubkey")]
    pub market_quote_vault: Pubkey,
    #[serde(with = "pubkey")]
    pub market_bids: Pubkey,
    #[serde(with = "pubkey")]
    pub market_asks: Pubkey,
    #[serde(with = "pubkey")]
    pub market_event_queue: Pubkey,
}

/// Make a call to the raydium api endpoint to retrieve all liquidity pools.
pub async fn fetch_all_liquidity_pools() -> anyhow::Result<LiqPoolInformation> {
    debug!("fn: fetch_all_liquidity_pools");
    info!(
        "Fetching LP infos from raydium api endpoint={}",
        RAYDIUM_POOL_INFO_ENDPOINT
    );
    Ok(reqwest::get(RAYDIUM_POOL_INFO_ENDPOINT)
        .await?
        .json()
        .await?)
}

pub async fn fetch_all_prices() -> anyhow::Result<HashMap<String, f64>> {
    debug!("fn: fetch_all_prices");
    info!(
        "Fetching price infos from raydium api endpoint={}",
        RAYDIUM_PRICE_INFO_ENDPOINT
    );
    let price_info_result: serde_json::Value = reqwest::get(RAYDIUM_PRICE_INFO_ENDPOINT)
        .await?
        .json()
        .await?;
    deserialize_price_info(price_info_result)
}
fn deserialize_price_info(value: serde_json::Value) -> anyhow::Result<HashMap<String, f64>> {
    debug!("fn: deserialize_price_info(value={})", value);
    Ok(value
        .as_object()
        .ok_or(anyhow::format_err!("malformed content. expected object."))?
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.as_f64().expect("value is f64")))
        .collect())
}

/// Retrieve pool information for a particular token pair. Cache path is an optional path to the
/// already dumped json data from the raydium API.
///
/// Although `token_a` is the base mint and `token_b` is the quote mint, this method will return
/// a `token_b/token_a` pool if it can't find a `token_a/token_b` one.
pub async fn get_pool_info(
    token_a: &Pubkey,
    token_b: &Pubkey,
    cache_path: Option<String>,
    allow_unofficial: bool,
) -> anyhow::Result<Option<LiquidityPool>> {
    debug!(
        "fn: get_pool_info(token_a={},token_b={},cache_path={:?})",
        token_a, token_b, cache_path
    );
    let pools = if let Some(path) = cache_path {
        info!("Fetching liq-pool-infos from pool-cache. path={}", path);
        serde_json::from_str(&std::fs::read_to_string(path)?)?
    } else {
        fetch_all_liquidity_pools().await?
    };

    let mut pools: Box<dyn Iterator<Item = _>> = if allow_unofficial {
        Box::new(
            pools
                .official
                .into_iter()
                .chain(pools.unofficial.into_iter()),
        )
    } else {
        Box::new(pools.official.into_iter())
    };

    match pools.find(|pool| pool.base_mint == *token_a && pool.quote_mint == *token_b) {
        Some(pool) => Ok(Some(pool)),
        None => Ok(pools.find(|pool| pool.base_mint == *token_b && pool.quote_mint == *token_a)),
    }
}

/*pub async fn get_price(token: &Pubkey, cache_path: &Option<String>) -> anyhow::Result<Option<f64>> {
    debug!(
        "fn: get_price(token = {},cache_path = {:?})",
        token, cache_path
    );
    let pools = if let Some(path) = cache_path {
        deserialize_price_info(serde_json::from_str(&std::fs::read_to_string(path)?)?)?
    } else {
        fetch_all_prices().await?
    };

    Ok(pools
        .into_iter()
        .find_map(|(tok, price)| {
            if token.to_string() == *tok {
                return Some(price);
            }
            None
        }))
}*/

pub async fn get_price(token: &Pubkey, cache_path: &Option<String>) -> anyhow::Result<f64> {
    debug!(
        "fn: get_price(token = {},cache_path = {:?})",
        token, cache_path
    );
    let pools = if let Some(path) = cache_path {
        info!("Fetching price-information from price-cache. path={}", path);
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
        .ok_or_else(|| {
            error!("Failed to find price for token {}", token);
            anyhow::anyhow!("Failed to find price for token {}", token)
        })
}

pub mod pubkey {
    use serde::{self, Deserialize, Deserializer, Serializer};
    pub use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    pub fn serialize<S>(pubkey: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", pubkey);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Pubkey::from_str(&s).map_err(serde::de::Error::custom)
    }
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
