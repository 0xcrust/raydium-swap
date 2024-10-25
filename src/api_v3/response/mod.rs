pub mod pools;
pub mod token;

use pools::{
    ApiV3BasePool, ApiV3BasePoolKeys, _ApiV3ClmmPool, _ApiV3ClmmPoolKeys, _ApiV3StandardPool,
    _ApiV3StandardPoolKeys,
};
use serde::{Deserialize, Serialize};
pub use token::ApiV3Token;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiV3Response<T> {
    pub id: String,
    pub success: bool,
    pub data: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiV3TokenList {
    #[serde(default)]
    pub mint_list: Vec<ApiV3Token>,
    #[serde(default)]
    pub blacklist: Vec<ApiV3Token>,
    #[serde(default)]
    pub whitelist: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiV3PoolsPage<T> {
    pub count: u64,
    pub has_next_page: bool,
    #[serde(rename = "data")]
    pub pools: Vec<T>, //
}

pub type ApiV3StandardPool = ApiV3BasePool<_ApiV3StandardPool>;
pub type ApiV3StandardPoolKeys = ApiV3BasePoolKeys<_ApiV3StandardPoolKeys>;
pub type ApiV3StandardPoolsPage = ApiV3PoolsPage<ApiV3StandardPool>;

pub type ApiV3ClmmPool = ApiV3BasePool<_ApiV3ClmmPool>;
pub type ApiV3ClmmPoolKeys = ApiV3BasePoolKeys<_ApiV3ClmmPoolKeys>;
pub type ApiV3ClmmPoolsPage = ApiV3PoolsPage<ApiV3ClmmPool>;
