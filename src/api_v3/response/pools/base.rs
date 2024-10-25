use crate::api_v3::response::token::ApiV3Token;
use crate::api_v3::serde_helpers::{field_as_string, option_field_as_string};

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiV3BasePool<T> {
    #[serde(with = "field_as_string")]
    pub program_id: Pubkey,
    #[serde(with = "field_as_string")]
    pub id: Pubkey,
    pub mint_a: ApiV3Token,
    pub mint_b: ApiV3Token,
    pub reward_default_infos: Vec<ApiV3PoolFarmRewardInfo>,
    pub reward_default_pool_infos: Option<String>, // "Ecosystem" | "Fusion" | "Raydium" | "Clmm";
    pub price: f64,
    pub mint_amount_a: f64,
    pub mint_amount_b: f64,
    pub fee_rate: f64,
    pub open_time: String,
    pub pooltype: Vec<String>,
    pub tvl: f64,
    pub day: ApiV3PoolInfoCountItem,
    pub week: ApiV3PoolInfoCountItem,
    pub month: ApiV3PoolInfoCountItem,
    pub farm_upcoming_count: u32,
    pub farm_ongoing_count: u32,
    pub farm_finished_count: u32,
    #[serde(flatten)]
    pub pool: T,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiV3BasePoolKeys<T> {
    #[serde(with = "field_as_string")]
    pub program_id: Pubkey,
    #[serde(with = "field_as_string")]
    pub id: Pubkey,
    pub mint_a: ApiV3Token,
    pub mint_b: ApiV3Token,
    #[serde(default, with = "option_field_as_string")]
    pub lookup_table_account: Option<Pubkey>,
    pub open_time: String,
    pub vault: VaultKeys,
    #[serde(flatten)]
    pub keys: T,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VaultKeys {
    #[serde(rename = "A", with = "field_as_string")]
    pub a: Pubkey,
    #[serde(rename = "B", with = "field_as_string")]
    pub b: Pubkey,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiV3PoolFarmRewardInfo {
    pub mint: ApiV3Token,
    #[serde(with = "field_as_string")]
    pub per_second: i64,
    #[serde(default, with = "option_field_as_string")]
    pub start_time: Option<i64>,
    #[serde(default, with = "option_field_as_string")]
    pub end_time: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiV3PoolInfoCountItem {
    pub volume: f64,
    pub volume_quote: f64,
    pub volume_fee: f64,
    pub apr: f64,
    pub fee_apr: f64,
    pub price_min: f64,
    pub price_max: f64,
    pub reward_apr: Vec<f64>,
}
