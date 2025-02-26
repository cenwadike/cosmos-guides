use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

// Default amount for each token type
pub const DEFAULT_NATIVE_AMOUNT: u128 = 100_000; // 0.1 untrn
pub const DEFAULT_CW20_AMOUNT: u128 = 100_000_000; // 100 token for CW20s

// Default rate limit in seconds (24 hours)
pub const DEFAULT_RATE_LIMIT: u64 = 60 * 60 * 24;

// Native token denom
pub const NATIVE_DENOM: &str = "untrn";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenConfig {
    pub denom: Denom,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfo {
    pub last_claim_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin: Addr,
    pub tokens: Vec<TokenConfig>,
    pub rate_limit_seconds: u64,
}

pub const STATE: Item<State> = Item::new("state");
pub const USER_CLAIMS: Map<&Addr, UserInfo> = Map::new("user_claims");
