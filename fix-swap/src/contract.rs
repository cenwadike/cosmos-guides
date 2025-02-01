use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Market, EXCHANGE_RATES};

#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, DepsMut, Env, MessageInfo, Response,
};
use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:fix-swap";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let base_token = msg.base_token;
    let quote_token = msg.quote_token;

    let market = Market {
        base_token: Addr::unchecked("ATOM"),
        quote_token: quote_token.clone(),
        exchange_rate: msg.rate,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    EXCHANGE_RATES.save(
        deps.storage,
        (base_token.clone(), quote_token.clone()),
        &market.exchange_rate,
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("rate", msg.rate.to_string())
        .add_attribute("quote_token", quote_token.to_string()))
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
            base_token,
            quote_token,
            token_denom,
            recipient,
            amount_in,
        } => execute::execute_swap(
            deps,
            env,
            info,
            base_token,
            quote_token,
            token_denom,
            recipient,
            amount_in,
        ),
    }
}

pub mod execute {
    use cosmwasm_std::Uint128;
    use cosmwasm_std::{Coin, CosmosMsg, WasmMsg};
    use cw20::Cw20ExecuteMsg;

    use super::*;

    pub fn execute_swap(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        base_token: Addr,
        quote_token: Addr,
        token_denom: String,
        recipient: Addr,
        amount_in: u128,
    ) -> Result<Response, ContractError> {
        // Get exchange rate
        let exchange_rate =
            EXCHANGE_RATES.load(deps.storage, (base_token.clone(), quote_token.clone()))?;
        let attached_coin = info
            .funds
            .iter()
            .find(|x| x.denom == token_denom)
            .ok_or(ContractError::TokenNotFound {})?;

        // Ensure correct tokens were attached
        assert!(attached_coin
            .amount
            .eq(&<u128 as Into<Uint128>>::into(amount_in)));

        // get amount out
        let amount_out = amount_in * exchange_rate;

        // construct transfer msgs
        let mut msgs = vec![];

        // Transfer ATOM from user wallet
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: env.contract.address.to_string(),
            amount: vec![Coin {
                denom: "uatom".to_string(),
                amount: amount_out.into(),
            }],
        }));

        // Transfer tokens to recipient wallet
        let cw20_contract_addr = quote_token;
        let transfer_msg = Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount: amount_out.into(),
        };
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw20_contract_addr.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: vec![],
        }));

        Ok(Response::new()
            .add_messages(msgs) // notice message
            .add_attribute("action", "swap"))
    }
}
