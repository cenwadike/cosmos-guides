use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub base_token: Addr,
    pub quote_token: Addr,
    pub rate: u128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Swap {
        base_token: Addr,
        quote_token: Addr,
        token_denom: String,
        recipient: Addr,
        amount_in: u128,
    },
}

#[cw_serde]
pub enum QueryMsg {}
