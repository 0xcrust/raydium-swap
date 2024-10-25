mod client;
pub mod response;
mod serde_helpers;

use anyhow::Context;
pub use client::ApiV3Client;
use response::ApiV3Response;
use serde::{Deserialize, Serialize};

pub type ApiV3Result<T> = Result<ApiV3Response<T>, anyhow::Error>;

#[derive(Clone, Debug, Deserialize)]
pub struct ApiV3ErrorResponse {
    pub id: String,
    pub success: bool,
    pub msg: String,
}

impl std::fmt::Display for ApiV3ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Received error response from API: {}",
            self.msg
        ))
    }
}
impl std::error::Error for ApiV3ErrorResponse {}

async fn handle_response_or_error<T>(
    response: reqwest::Response,
) -> Result<ApiV3Response<T>, anyhow::Error>
where
    T: serde::de::DeserializeOwned,
{
    let response = response.error_for_status()?;
    let json = response.json::<serde_json::Value>().await?;
    let success = json
        .get("success")
        .and_then(|v| v.as_bool())
        .context("Invalid api response")?;

    if success {
        Ok(serde_json::from_value::<ApiV3Response<T>>(json)?)
    } else {
        Err(serde_json::from_value::<ApiV3ErrorResponse>(json)?.into())
    }
}

#[derive(Clone, Debug, Default)]
pub struct PoolFetchParams {
    pub pool_type: PoolType,
    pub pool_sort: PoolSort,
    pub sort_type: PoolSortOrder,
    pub page_size: u16,
    pub page: u16,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PoolType {
    #[default]
    All,
    Standard,
    Concentrated,
    AllFarm,
    StandardFarm,
    ConcentratedFarm,
}
impl std::fmt::Display for PoolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolType::All => f.write_str("all"),
            PoolType::Standard => f.write_str("standard"),
            PoolType::Concentrated => f.write_str("concentrated"),
            PoolType::AllFarm => f.write_str("allFarm"),
            PoolType::StandardFarm => f.write_str("standardFarm"),
            PoolType::ConcentratedFarm => f.write_str("concentratedFarm"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum PoolSort {
    #[default]
    Liquidity,
    Volume24h,
    Volume7d,
    Volume30d,
    Fee24h,
    Fee7d,
    Fee30d,
    Apr24h,
    Apr7d,
    Apr30d,
}
impl std::fmt::Display for PoolSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolSort::Liquidity => f.write_str("liquidity"),
            PoolSort::Volume24h => f.write_str("volume24h"),
            PoolSort::Volume7d => f.write_str("volume7d"),
            PoolSort::Volume30d => f.write_str("volume30d"),
            PoolSort::Fee24h => f.write_str("fee24h"),
            PoolSort::Fee7d => f.write_str("fee7d"),
            PoolSort::Fee30d => f.write_str("fee30d"),
            PoolSort::Apr24h => f.write_str("apr24h"),
            PoolSort::Apr7d => f.write_str("apr7d"),
            PoolSort::Apr30d => f.write_str("apr30d"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum PoolSortOrder {
    Ascending,
    #[default]
    Descending,
}
impl std::fmt::Display for PoolSortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolSortOrder::Ascending => f.write_str("asc"),
            PoolSortOrder::Descending => f.write_str("desc"),
        }
    }
}

#[cfg(test)]
pub mod raydium_api_v3 {
    use super::response::{
        ApiV3ClmmPool, ApiV3ClmmPoolKeys, ApiV3StandardPool, ApiV3StandardPoolKeys,
    };
    use super::{ApiV3Client, PoolFetchParams, PoolSort, PoolSortOrder, PoolType};

    #[tokio::test]
    pub async fn get_token_list_and_info() {
        let client = ApiV3Client::default();
        let token_list = client.get_token_list().await.unwrap();
        let keys = token_list
            .mint_list
            .into_iter()
            .take(5)
            .map(|token| token.address.to_string())
            .collect::<Vec<_>>();
        let token_info = client.get_token_info(keys).await.unwrap();
        assert!(token_info.len() == 5);
    }

    #[tokio::test]
    pub async fn get_pool_list() {
        let client = ApiV3Client::default();
        let _pools = client
            .get_pool_list::<serde_json::Value>(&Default::default())
            .await
            .unwrap();
    }

    #[tokio::test]
    pub async fn get_standard_pool_and_keys() {
        let client = ApiV3Client::default();
        let params = PoolFetchParams {
            pool_type: PoolType::Standard,
            pool_sort: PoolSort::Liquidity,
            sort_type: PoolSortOrder::Ascending,
            page_size: 20,
            page: 1,
        };

        let pools = client
            .get_pool_list::<ApiV3StandardPool>(&params)
            .await
            .unwrap();

        let ids = pools
            .pools
            .iter()
            .map(|p| p.id.to_string())
            .collect::<Vec<_>>();
        let pools_by_id = client
            .fetch_pools_by_ids::<ApiV3StandardPool>(ids.clone())
            .await
            .unwrap();
        let pool_keys_by_id = client
            .fetch_pool_keys_by_ids::<ApiV3StandardPoolKeys>(ids)
            .await
            .unwrap();

        for pool in pools.pools.iter().take(3) {
            let pools_by_mint = client
                .fetch_pool_by_mints::<ApiV3StandardPool>(
                    &pool.mint_a.address,
                    Some(&pool.mint_b.address),
                    &params,
                )
                .await
                .unwrap();
            let found = pools_by_mint.pools.iter().find(|p| p.id == pool.id);
            assert!(found.is_some());

            let pool_by_id = pools_by_id.iter().find(|p| p.id == pool.id);
            assert!(pool_by_id.is_some());
            assert!(pool_by_id.unwrap().id == pool.id);

            let pool_keys_by_id = pool_keys_by_id.iter().find(|p| p.id == pool.id);
            assert!(pool_keys_by_id.is_some());
            assert!(pool_keys_by_id.unwrap().id == pool.id);
        }
    }

    #[tokio::test]
    pub async fn get_clmm_pool_and_keys() {
        let client = ApiV3Client::default();
        let params = PoolFetchParams {
            pool_type: PoolType::Concentrated,
            pool_sort: PoolSort::Liquidity,
            sort_type: PoolSortOrder::Ascending,
            page_size: 100,
            page: 1,
        };
        let pools = client
            .get_pool_list::<ApiV3ClmmPool>(&params)
            .await
            .unwrap();

        let ids = pools
            .pools
            .iter()
            .map(|p| p.id.to_string())
            .collect::<Vec<_>>();
        let pools_by_id = client
            .fetch_pools_by_ids::<ApiV3ClmmPool>(ids.clone())
            .await
            .unwrap();
        let pool_keys_by_id = client
            .fetch_pool_keys_by_ids::<ApiV3ClmmPoolKeys>(ids)
            .await
            .unwrap();

        for pool in pools.pools.iter().take(3) {
            let pools_by_mint = client
                .fetch_pool_by_mints::<ApiV3ClmmPool>(
                    &pool.mint_a.address,
                    Some(&pool.mint_b.address),
                    &params,
                )
                .await
                .unwrap();
            let found = pools_by_mint.pools.iter().find(|p| p.id == pool.id);
            assert!(found.is_some());

            let pool_by_id = pools_by_id.iter().find(|p| p.id == pool.id);
            assert!(pool_by_id.is_some());
            assert!(pool_by_id.unwrap().id == pool.id);

            let pool_keys_by_id = pool_keys_by_id.iter().find(|p| p.id == pool.id);
            assert!(pool_keys_by_id.is_some());
            assert!(pool_keys_by_id.unwrap().id == pool.id);
        }
    }
}
