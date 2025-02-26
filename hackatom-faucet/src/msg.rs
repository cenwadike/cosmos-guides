use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw20::Balance;

use crate::state::{TokenConfig, UserInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub tokens: Vec<TokenConfig>,
    pub rate_limit_seconds: Option<u64>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Claim {},
    UpdateTokenConfig {
        token_index: u32,
        new_config: TokenConfig,
    },
    UpdateRateLimit {
        seconds: u64,
    },
    SetAdmin {
        admin: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TokenConfigsResponse)]
    GetTokenConfigs {},
    #[returns(RateLimitResponse)]
    GetRateLimit {},
    #[returns(UserInfoResponse)]
    GetUserInfo { address: String },
    #[returns(AdminResponse)]
    GetAdmin {},
    #[returns(BalanceResponse)]
    GetBalance {},
    #[returns(ClaimStatusResponse)]
    CanUserClaim { address: String },
}

#[cw_serde]
pub struct TokenConfigsResponse {
    pub tokens: Vec<TokenConfig>,
}

#[cw_serde]
pub struct RateLimitResponse {
    pub rate_limit_seconds: u64,
}

#[cw_serde]
pub struct UserInfoResponse {
    pub user_info: Option<UserInfo>,
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: String,
}

#[cw_serde]
pub struct BalanceResponse {
    pub balances: Vec<Balance>,
}

#[cw_serde]
pub struct ClaimStatusResponse {
    pub can_claim: bool,
    pub seconds_until_next_claim: u64,
}
