use crate::api_v3::response::token::ApiV3Token;
use crate::api_v3::serde_helpers::{field_as_string, option_field_as_string};
use crate::api_v3::PoolType;

use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct _ApiV3StandardPool {
    /// Standard
    #[serde(rename = "type")]
    pub pool_type: PoolType,
    #[serde(default, with = "option_field_as_string")]
    pub market_id: Option<Pubkey>,
    #[serde(default, with = "option_field_as_string")]
    pub config_id: Option<Pubkey>,
    pub lp_price: f64,
    pub lp_amount: f64,
    pub lp_mint: ApiV3Token,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct _ApiV3StandardPoolKeys {
    #[serde(with = "field_as_string")]
    pub authority: Pubkey,
    pub mint_lp: ApiV3Token,
    #[serde(flatten)]
    pub market: Option<MarketKeys>,
    #[serde(default, with = "option_field_as_string")]
    pub open_orders: Option<Pubkey>,
    #[serde(default, with = "option_field_as_string")]
    pub target_orders: Option<Pubkey>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketKeys {
    #[serde(with = "field_as_string")]
    pub market_program_id: Pubkey,
    #[serde(with = "field_as_string")]
    pub market_id: Pubkey,
    #[serde(with = "field_as_string")]
    pub market_authority: Pubkey,
    #[serde(with = "field_as_string")]
    pub market_base_vault: Pubkey,
    #[serde(with = "field_as_string")]
    pub market_quote_vault: Pubkey,
    #[serde(with = "field_as_string")]
    pub market_bids: Pubkey,
    #[serde(with = "field_as_string")]
    pub market_asks: Pubkey,
    #[serde(with = "field_as_string")]
    pub market_event_queue: Pubkey,
}
