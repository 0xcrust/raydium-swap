use crate::api_v3::response::token::ApiV3Token;
use crate::api_v3::serde_helpers::field_as_string;
use crate::api_v3::PoolType;

use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct _ApiV3CpmmPool {
    /// Standard
    #[serde(rename = "type")]
    pub pool_type: PoolType,
    pub lp_mint: ApiV3Token,
    pub lp_price: f64,
    pub lp_amount: u64,
    pub config: ApiV3CpmmConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiV3CpmmConfig {
    #[serde(with = "field_as_string")]
    pub id: Pubkey,
    pub index: u16,
    pub protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
    pub fund_fee_rate: u32,
    pub create_pool_fee: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct _ApiV3CpmmPoolKeys {
    #[serde(with = "field_as_string")]
    pub authority: Pubkey,
    pub mint_lp: ApiV3Token,
    pub config: ApiV3CpmmConfig,
}
