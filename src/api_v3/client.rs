use super::response::{ApiV3PoolsPage, ApiV3Token, ApiV3TokenList};
use super::{handle_response_or_error, PoolFetchParams};
use serde::de::DeserializeOwned;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub struct ApiV3Client {
    base_url: String,
}

impl Default for ApiV3Client {
    fn default() -> Self {
        ApiV3Client {
            base_url: Self::DEFAULT_BASE_URL.to_string(),
        }
    }
}

impl ApiV3Client {
    const DEFAULT_BASE_URL: &'static str = "https://api-v3.raydium.io";

    pub fn new(base_url: Option<String>) -> Self {
        ApiV3Client {
            base_url: base_url.unwrap_or(Self::DEFAULT_BASE_URL.to_string()),
        }
    }

    pub async fn get_token_list(&self) -> Result<ApiV3TokenList, anyhow::Error> {
        let url = format!("{}/mint/list", &self.base_url);
        Ok(handle_response_or_error(reqwest::get(url).await?)
            .await?
            .data)
    }

    pub async fn get_jup_token_list(&self) -> Result<Vec<ApiV3Token>, anyhow::Error> {
        Ok(
            reqwest::get("https://tokens.jup.ag/tokens?tags=lst,community")
                .await?
                .json()
                .await?,
        )
    }

    pub async fn get_token_info(
        &self,
        mints: Vec<String>,
    ) -> Result<Vec<ApiV3Token>, anyhow::Error> {
        let mints = mints.join(",");
        let url = format!("{}/mint/ids?mints={}", &self.base_url, mints);
        Ok(handle_response_or_error(reqwest::get(url).await?)
            .await?
            .data)
    }

    pub async fn get_pool_list<T: DeserializeOwned>(
        &self,
        params: &PoolFetchParams,
    ) -> Result<ApiV3PoolsPage<T>, anyhow::Error> {
        let url = format!(
            "{}/pools/info/list?poolType={}&poolSortField={}&sortType={}&page={}&pageSize={}",
            &self.base_url,
            params.pool_type,
            params.pool_sort,
            params.sort_type,
            params.page,
            params.page_size
        );
        Ok(handle_response_or_error(reqwest::get(url).await?)
            .await?
            .data)
    }

    pub async fn fetch_pools_by_ids<T: DeserializeOwned>(
        &self,
        ids: Vec<String>,
    ) -> Result<Vec<T>, anyhow::Error> {
        let ids = ids.join(",");
        let url = format!("{}/pools/info/ids?ids={}", &self.base_url, ids);
        Ok(handle_response_or_error(reqwest::get(url).await?)
            .await?
            .data)
    }

    pub async fn fetch_pool_keys_by_ids<T: DeserializeOwned>(
        &self,
        ids: Vec<String>,
    ) -> Result<Vec<T>, anyhow::Error> {
        let ids = ids.join(",");
        let url = format!("{}/pools/key/ids?ids={}", &self.base_url, ids);
        Ok(handle_response_or_error(reqwest::get(url).await?)
            .await?
            .data)
    }

    pub async fn fetch_pool_by_mints<T: DeserializeOwned>(
        &self,
        mint1: &Pubkey,
        mint2: Option<&Pubkey>,
        params: &PoolFetchParams,
    ) -> Result<ApiV3PoolsPage<T>, anyhow::Error> {
        let url = format!(
            "{}/pools/info/mint?mint1={}&mint2={}&poolType={}&poolSortField={}&sortType={}&pageSize={}&page={}",
            &self.base_url,
            mint1,
            mint2.map(|x| x.to_string()).unwrap_or_default(),
            params.pool_type,
            params.pool_sort,
            params.sort_type,
            100,
            params.page
        );
        Ok(handle_response_or_error(reqwest::get(url).await?)
            .await?
            .data)
    }
}
