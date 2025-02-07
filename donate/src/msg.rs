use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Donate { recipient: Addr, amount_in: Uint128 },
}

#[cw_serde]
pub enum QueryMsg {}
