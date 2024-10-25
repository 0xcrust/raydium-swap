use crate::api_v3::response::token::ApiV3Token;
use crate::api_v3::serde_helpers::field_as_string;
use crate::api_v3::PoolType;

use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct _ApiV3ClmmPool {
    /// Concentrated
    #[serde(rename = "type")]
    pub pool_type: PoolType,
    pub config: ApiV3ClmmConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiV3ClmmConfig {
    #[serde(with = "field_as_string")]
    pub id: Pubkey,
    pub index: u16,
    pub protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
    pub tick_spacing: u16,
    pub fund_fee_rate: u32,
    //description: Option<String>,
    pub default_range: f64,
    pub default_range_point: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct _ApiV3ClmmPoolKeys {
    pub config: ApiV3ClmmConfig,
    pub reward_infos: Vec<ClmmRewardType>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClmmRewardType {
    pub mint: ApiV3Token,
    #[serde(with = "field_as_string")]
    pub vault: Pubkey,
}
