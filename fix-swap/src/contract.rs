use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Addr, BankMsg, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:fix-swap";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Swap {
            recipient,
            amount_in,
        } => execute::execute_swap(deps, env, info, recipient, amount_in),
    }
}

pub mod execute {
    use cosmwasm_std::{Coin, CosmosMsg, Uint128};

    use super::*;

    pub fn execute_swap(
        _deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        recipient: Addr,
        amount_in: Uint128,
    ) -> Result<Response, ContractError> {
        // get amount out
        let amount_out = amount_in / Uint128::new(1_000_000u128);

        // construct transfer msgs
        let mut msgs = vec![];

        // Transfer ATOM from user wallet
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: env.contract.address.to_string(),
            amount: vec![Coin {
                denom: "untrn".to_string(),
                amount: amount_in,
            }],
        }));

        // transfer a fraction to user
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: "untrn".to_string(),
                amount: amount_out,
            }],
        }));

        Ok(Response::new()
            .add_messages(msgs) // notice message
            .add_attribute("action", "swap"))
    }
}
