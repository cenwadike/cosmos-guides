use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Market {
    pub base_token: Addr,    // eg. ATOM in ATOM/USDT 
    pub quote_token: Addr,   // eg. USDT in ATOM/USDT
    pub exchange_rate: u128, // eg. ATOM/USDT exchange is 10
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum TokenType {
    Native,
    Cw20,
}

pub const EXCHANGE_RATES: Map<(Addr, Addr), u128> = Map::new("exchange_rates");
