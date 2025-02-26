#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use execute::{
    execute_claim, execute_set_admin, execute_update_rate_limit, execute_update_token_config,
};
use query::{
    query_admin, query_balance, query_can_user_claim, query_rate_limit, query_token_configs,
    query_user_info,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, DEFAULT_RATE_LIMIT, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:hackatom-faucet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let admin = msg.admin.unwrap_or(info.sender.clone());

    let rate_limit = msg.rate_limit_seconds.unwrap_or(DEFAULT_RATE_LIMIT);

    let state = State {
        admin,
        tokens: msg.tokens,
        rate_limit_seconds: rate_limit,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", state.admin.to_string())
        .add_attribute("rate_limit", rate_limit.to_string())
        .add_attribute("tokens_count", state.tokens.len().to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::UpdateTokenConfig {
            token_index,
            new_config,
        } => execute_update_token_config(deps, info, token_index, new_config),
        ExecuteMsg::UpdateRateLimit { seconds } => execute_update_rate_limit(deps, info, seconds),
        ExecuteMsg::SetAdmin { admin } => execute_set_admin(deps, info, admin),
    }
}

pub mod execute {
    use cosmwasm_std::{coin, Addr, BankMsg, CosmosMsg, WasmMsg};
    use cw20::{Cw20ExecuteMsg, Denom};

    use crate::state::{TokenConfig, UserInfo, NATIVE_DENOM, USER_CLAIMS};

    use super::*;

    pub fn execute_claim(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        // Load state using Item
        let state = STATE.load(deps.storage)?;

        let user_addr = info.sender.clone();
        let current_time = env.block.time.seconds();

        // Check user's last claim time using Map
        let user_info = USER_CLAIMS.may_load(deps.storage, &user_addr)?;

        // Check if user is rate limited
        if let Some(user_data) = &user_info {
            let time_since_last_claim = current_time.saturating_sub(user_data.last_claim_time);
            if time_since_last_claim < state.rate_limit_seconds {
                let seconds_remaining = state.rate_limit_seconds - time_since_last_claim;
                return Err(cosmwasm_std::StdError::generic_err(format!(
                    "Rate limit exceeded. You can claim again in {} seconds",
                    seconds_remaining
                )));
            }
        }

        // Update or create user claim record
        let updated_user_info = UserInfo {
            last_claim_time: current_time,
        };
        USER_CLAIMS.save(deps.storage, &user_addr, &updated_user_info)?;

        // Prepare transfer messages for each token
        let mut messages: Vec<CosmosMsg> = vec![];
        let mut distributed_tokens: Vec<String> = vec![];

        for token_config in state.tokens.iter() {
            let token_msg = match &token_config.denom {
                Denom::Native(denom) if denom == NATIVE_DENOM => {
                    // Create bank send message for native token (untrn)
                    let msg = CosmosMsg::Bank(BankMsg::Send {
                        to_address: user_addr.to_string(),
                        amount: vec![coin(token_config.amount.u128(), denom)],
                    });

                    // Check if contract has enough balance for this native token
                    let balance = deps
                        .querier
                        .query_balance(env.contract.address.clone(), denom)?;
                    if balance.amount >= token_config.amount {
                        distributed_tokens.push(format!("{} {}", token_config.amount, denom));
                        Some(msg)
                    } else {
                        None // Skip if insufficient balance
                    }
                }
                Denom::Cw20(contract_addr) => {
                    // Create CW20 transfer message
                    let cw20_addr = contract_addr;

                    // Query CW20 balance of the contract
                    let balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                        cw20_addr,
                        &cw20::Cw20QueryMsg::Balance {
                            address: env.contract.address.to_string(),
                        },
                    )?;

                    if balance.balance >= token_config.amount {
                        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: cw20_addr.to_string(),
                            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                                recipient: user_addr.to_string(),
                                amount: token_config.amount,
                            })?,
                            funds: vec![],
                        });
                        distributed_tokens
                            .push(format!("{} CW20:{}", token_config.amount, contract_addr));
                        Some(msg)
                    } else {
                        None // Skip if insufficient balance
                    }
                }
                _ => None, // Skip other token types
            };

            if let Some(msg) = token_msg {
                messages.push(msg);
            }
        }

        // Return error if no tokens were distributed
        if messages.is_empty() {
            return Err(cosmwasm_std::StdError::generic_err(
                "Insufficient funds in faucet for all token types",
            ));
        }

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "claim")
            .add_attribute("recipient", user_addr.to_string())
            .add_attribute("distributed_tokens", distributed_tokens.join(", ")))
    }

    pub fn execute_update_token_config(
        deps: DepsMut,
        info: MessageInfo,
        token_index: u32,
        new_config: TokenConfig,
    ) -> StdResult<Response> {
        // Load state using Item
        let mut state = STATE.load(deps.storage)?;

        // Check if sender is admin
        if info.sender != state.admin {
            return Err(cosmwasm_std::StdError::generic_err("Unauthorized"));
        }

        // Check if token index is valid
        if token_index as usize >= state.tokens.len() {
            return Err(cosmwasm_std::StdError::generic_err("Invalid token index"));
        }

        // Update token config
        state.tokens[token_index as usize] = new_config.clone();

        // Save updated state
        STATE.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_attribute("action", "update_token_config")
            .add_attribute("token_index", token_index.to_string())
            .add_attribute("new_denom", format!("{:?}", new_config.denom))
            .add_attribute("new_amount", new_config.amount.to_string()))
    }

    pub fn execute_update_rate_limit(
        deps: DepsMut,
        info: MessageInfo,
        seconds: u64,
    ) -> StdResult<Response> {
        // Load state using Item
        let mut state = STATE.load(deps.storage)?;

        // Check if sender is admin
        if info.sender != state.admin {
            return Err(cosmwasm_std::StdError::generic_err("Unauthorized"));
        }

        // Update rate limit
        state.rate_limit_seconds = seconds;

        // Save updated state
        STATE.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_attribute("action", "update_rate_limit")
            .add_attribute("seconds", seconds.to_string()))
    }

    pub fn execute_set_admin(
        deps: DepsMut,
        info: MessageInfo,
        admin: String,
    ) -> StdResult<Response> {
        // Load state using Item
        let mut state = STATE.load(deps.storage)?;

        // Check if sender is current admin
        if info.sender != state.admin {
            return Err(cosmwasm_std::StdError::generic_err("Unauthorized"));
        }

        // Update admin
        let new_admin = Addr::unchecked(admin);
        state.admin = new_admin.clone();

        // Save updated state
        STATE.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_attribute("action", "set_admin")
            .add_attribute("admin", new_admin.to_string()))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTokenConfigs {} => to_json_binary(&query_token_configs(deps)?),
        QueryMsg::GetRateLimit {} => to_json_binary(&query_rate_limit(deps)?),
        QueryMsg::GetUserInfo { address } => to_json_binary(&query_user_info(deps, address)?),
        QueryMsg::GetAdmin {} => to_json_binary(&query_admin(deps)?),
        QueryMsg::GetBalance {} => to_json_binary(&query_balance(deps, env)?),
        QueryMsg::CanUserClaim { address } => {
            to_json_binary(&query_can_user_claim(deps, env, address)?)
        }
    }
}

pub mod query {
    use cosmwasm_std::Addr;
    use cw20::{Balance, Denom};
    use cw_utils::NativeBalance;

    use crate::{
        msg::{
            AdminResponse, BalanceResponse, ClaimStatusResponse, RateLimitResponse,
            TokenConfigsResponse, UserInfoResponse,
        },
        state::USER_CLAIMS,
    };

    use super::*;

    pub fn query_token_configs(deps: Deps) -> StdResult<TokenConfigsResponse> {
        let state = STATE.load(deps.storage)?;

        Ok(TokenConfigsResponse {
            tokens: state.tokens,
        })
    }

    pub fn query_rate_limit(deps: Deps) -> StdResult<RateLimitResponse> {
        let state = STATE.load(deps.storage)?;

        Ok(RateLimitResponse {
            rate_limit_seconds: state.rate_limit_seconds,
        })
    }

    pub fn query_user_info(deps: Deps, address: String) -> StdResult<UserInfoResponse> {
        let user_addr = Addr::unchecked(address);
        let user_info = USER_CLAIMS.may_load(deps.storage, &user_addr)?;

        Ok(UserInfoResponse { user_info })
    }

    pub fn query_admin(deps: Deps) -> StdResult<AdminResponse> {
        let state = STATE.load(deps.storage)?;

        Ok(AdminResponse {
            admin: state.admin.to_string(),
        })
    }

    pub fn query_balance(deps: Deps, env: Env) -> StdResult<BalanceResponse> {
        let state = STATE.load(deps.storage)?;
        let contract_addr = env.contract.address;

        // Query actual balances for each token
        let mut balances: Vec<Balance> = vec![];

        for token_config in state.tokens.iter() {
            match &token_config.denom {
                Denom::Native(denom) => {
                    // Query native token balance
                    let balance = deps.querier.query_balance(contract_addr.clone(), denom)?;
                    balances.push(Balance::Native(NativeBalance(vec![balance])));
                }
                Denom::Cw20(addr) => {
                    // Query CW20 token balance
                    let cw20_addr = addr;
                    let balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                        cw20_addr,
                        &cw20::Cw20QueryMsg::Balance {
                            address: contract_addr.to_string(),
                        },
                    )?;

                    balances.push(Balance::Cw20(cw20::Cw20CoinVerified {
                        address: addr.clone(),
                        amount: balance.balance,
                    }));
                }
            }
        }

        Ok(BalanceResponse { balances })
    }

    pub fn query_can_user_claim(
        deps: Deps,
        env: Env,
        address: String,
    ) -> StdResult<ClaimStatusResponse> {
        let state = STATE.load(deps.storage)?;
        let user_addr = Addr::unchecked(address);

        let user_info = USER_CLAIMS.may_load(deps.storage, &user_addr)?;
        let current_time = env.block.time.seconds();

        if let Some(info) = user_info {
            let time_since_last_claim = current_time.saturating_sub(info.last_claim_time);
            if time_since_last_claim >= state.rate_limit_seconds {
                // User can claim
                Ok(ClaimStatusResponse {
                    can_claim: true,
                    seconds_until_next_claim: 0,
                })
            } else {
                // User must wait
                let seconds_remaining = state.rate_limit_seconds - time_since_last_claim;
                Ok(ClaimStatusResponse {
                    can_claim: false,
                    seconds_until_next_claim: seconds_remaining,
                })
            }
        } else {
            // First-time user can claim
            Ok(ClaimStatusResponse {
                can_claim: true,
                seconds_until_next_claim: 0,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::contract::instantiate;
    use crate::msg::InstantiateMsg;
    use crate::state::{
        State, TokenConfig, UserInfo, DEFAULT_CW20_AMOUNT, DEFAULT_NATIVE_AMOUNT, NATIVE_DENOM,
        STATE, USER_CLAIMS,
    };
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_dependencies_with_balance, mock_env,
    };
    use cosmwasm_std::{
        coins, from_json, Addr, BankMsg, Coin, ContractResult, CosmosMsg, Response, SystemError,
        SystemResult, Uint128, WasmMsg, WasmQuery,
    };
    use cw20::{Balance, BalanceResponse, Cw20ExecuteMsg, Denom};

    // Define a helper function to create a default instantiation message
    fn default_instantiate_msg() -> InstantiateMsg {
        InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native("untrn".to_string()),
                    amount: Uint128::from(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("cw20_token")),
                    amount: Uint128::from(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        }
    }

    // Test the instantiation process
    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let msg = default_instantiate_msg();

        let response: Response = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // Verify response attributes
        assert_eq!(response.attributes.len(), 4);
        assert_eq!(response.attributes[0].key, "method");
        assert_eq!(response.attributes[0].value, "instantiate");
        assert_eq!(response.attributes[1].key, "admin");
        assert_eq!(response.attributes[1].value, "admin");
        assert_eq!(response.attributes[2].key, "rate_limit");
        assert_eq!(response.attributes[2].value, DEFAULT_RATE_LIMIT.to_string());
        assert_eq!(response.attributes[3].key, "tokens_count");
        assert_eq!(response.attributes[3].value, "2");

        // Verify state stored in contract
        let state: State = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.admin, Addr::unchecked("admin"));
        assert_eq!(state.rate_limit_seconds, DEFAULT_RATE_LIMIT);
        assert_eq!(state.tokens.len(), 2);
        assert_eq!(state.tokens[0].denom, Denom::Native("untrn".to_string()));
        assert_eq!(state.tokens[0].amount, Uint128::from(DEFAULT_NATIVE_AMOUNT));
        assert_eq!(
            state.tokens[1].denom,
            Denom::Cw20(Addr::unchecked("cw20_token"))
        );
        assert_eq!(state.tokens[1].amount, Uint128::from(DEFAULT_CW20_AMOUNT));
    }

    #[test]
    fn test_execute_claim_native() {
        let mut deps = mock_dependencies_with_balance(&[Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000),
        }]);

        // Initialize the contract
        let msg = InstantiateMsg {
            admin: None,
            tokens: vec![TokenConfig {
                denom: Denom::Native(NATIVE_DENOM.to_string()),
                amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
            }],
            rate_limit_seconds: None,
        };

        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // First claim should succeed
        let claim_info = message_info(&Addr::unchecked("user1"), &[]);
        let claim_res = execute_claim(deps.as_mut(), env.clone(), claim_info.clone()).unwrap();

        assert_eq!(claim_res.attributes.len(), 3);
        assert_eq!(claim_res.attributes[0].key, "action");
        assert_eq!(claim_res.attributes[0].value, "claim");
        assert_eq!(claim_res.attributes[1].key, "recipient");
        assert_eq!(claim_res.attributes[1].value, "user1");
        assert!(claim_res.messages.len() > 0);

        // Verify the BankMsg::Send was created
        match &claim_res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "user1");
                assert_eq!(amount[0].amount, Uint128::new(DEFAULT_NATIVE_AMOUNT));
                assert_eq!(amount[0].denom, NATIVE_DENOM);
            }
            _ => panic!("Unexpected message: {:?}", &claim_res.messages[0].msg),
        }

        // Second claim should fail due to rate limiting
        let err = execute_claim(deps.as_mut(), env.clone(), claim_info.clone()).unwrap_err();
        assert!(err.to_string().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_execute_claim_cw20() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let msg = InstantiateMsg {
            admin: None,
            tokens: vec![TokenConfig {
                denom: Denom::Cw20(Addr::unchecked("token_contract")),
                amount: Uint128::new(DEFAULT_CW20_AMOUNT),
            }],
            rate_limit_seconds: None,
        };

        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Mock balance for CW20 tokens
        let cw20_token_address = String::from("token_contract");

        // Clone necessary variables to avoid moving them into the closure
        let cw20_token_address_clone = cw20_token_address.clone();

        // Mock the balance
        deps.querier.update_wasm(move |query| {
            let cw20_token_address = cw20_token_address_clone.clone();
            let env = mock_env();

            match query {
                WasmQuery::Smart { contract_addr, msg } => {
                    if contract_addr == &cw20_token_address.clone() {
                        if let Ok(cw20::Cw20QueryMsg::Balance { address }) = from_json(&msg) {
                            if address == env.contract.address.to_string() {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&BalanceResponse {
                                        balance: Uint128::from(100_000_000u128),
                                    })
                                    .unwrap(),
                                ));
                            }
                        }
                    }
                    SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: "".to_string(),
                    })
                }
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "".to_string(),
                }),
            }
        });

        // First claim should succeed
        let claim_info = message_info(&Addr::unchecked("user1"), &[]);
        let claim_res = execute_claim(deps.as_mut(), env.clone(), claim_info.clone()).unwrap();

        assert_eq!(claim_res.attributes.len(), 3);
        assert_eq!(claim_res.attributes[0].key, "action");
        assert_eq!(claim_res.attributes[0].value, "claim");
        assert_eq!(claim_res.attributes[1].key, "recipient");
        assert_eq!(claim_res.attributes[1].value, "user1");
        assert!(claim_res.messages.len() > 0);

        // Verify the WasmMsg::Execute was created for CW20 transfer
        match &claim_res.messages[0].msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                msg,
                funds,
            }) => {
                assert_eq!(contract_addr, "token_contract");
                let expected_msg = Cw20ExecuteMsg::Transfer {
                    recipient: "user1".to_string(),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                };
                let actual_msg: Cw20ExecuteMsg = from_json(&msg).unwrap();
                assert_eq!(actual_msg, expected_msg);
                assert!(funds.is_empty());
            }
            _ => panic!("Unexpected message: {:?}", &claim_res.messages[0].msg),
        }

        // Second claim should fail due to rate limiting
        let err = execute_claim(deps.as_mut(), env.clone(), claim_info.clone()).unwrap_err();
        assert!(err.to_string().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_update_token_config_success() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("old_token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // New token config to update
        let new_token_config = TokenConfig {
            denom: Denom::Cw20(Addr::unchecked("new_token_contract")),
            amount: Uint128::new(200_000_000),
        };

        // Execute the update token config function
        let update_info = message_info(&Addr::unchecked("admin"), &[]);
        let res =
            execute_update_token_config(deps.as_mut(), update_info, 1, new_token_config.clone())
                .unwrap();

        // Verify the response
        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "update_token_config");
        assert_eq!(res.attributes[1].key, "token_index");
        assert_eq!(res.attributes[1].value, "1");
        assert_eq!(res.attributes[2].key, "new_denom");
        assert_eq!(
            res.attributes[2].value,
            format!("{:?}", new_token_config.denom)
        );
        assert_eq!(res.attributes[3].key, "new_amount");
        assert_eq!(res.attributes[3].value, new_token_config.amount.to_string());

        // Verify state
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.tokens[1], new_token_config);
    }

    #[test]
    fn test_update_token_config_unauthorized() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("old_token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // New token config to update
        let new_token_config = TokenConfig {
            denom: Denom::Cw20(Addr::unchecked("new_token_contract")),
            amount: Uint128::new(200_000_000),
        };

        // Attempt to update token config by a non-admin
        let update_info = message_info(&Addr::unchecked("user"), &[]);
        let err = execute_update_token_config(deps.as_mut(), update_info, 1, new_token_config)
            .unwrap_err();

        // Verify error message
        assert_eq!(err.to_string(), "Generic error: Unauthorized");
    }

    #[test]
    fn test_update_token_config_invalid_index() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("old_token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // New token config to update
        let new_token_config = TokenConfig {
            denom: Denom::Cw20(Addr::unchecked("new_token_contract")),
            amount: Uint128::new(200_000_000),
        };

        // Attempt to update token config with an invalid index
        let update_info = message_info(&Addr::unchecked("admin"), &[]);
        let err = execute_update_token_config(deps.as_mut(), update_info, 5, new_token_config)
            .unwrap_err();

        // Verify error message
        assert_eq!(err.to_string(), "Generic error: Invalid token index");
    }

    #[test]
    fn test_update_rate_limit_success() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("old_token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Execute the update rate limit function
        let update_info = message_info(&Addr::unchecked("admin"), &[]);
        let new_rate_limit = 3600; // 1 hour
        let res = execute_update_rate_limit(deps.as_mut(), update_info, new_rate_limit).unwrap();

        // Verify the response
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "update_rate_limit");
        assert_eq!(res.attributes[1].key, "seconds");
        assert_eq!(res.attributes[1].value, new_rate_limit.to_string());

        // Verify state
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.rate_limit_seconds, new_rate_limit);
    }

    #[test]
    fn test_update_rate_limit_unauthorized() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("old_token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Attempt to update rate limit by a non-admin
        let update_info = message_info(&Addr::unchecked("user"), &[]);
        let new_rate_limit = 3600; // 1 hour
        let err =
            execute_update_rate_limit(deps.as_mut(), update_info, new_rate_limit).unwrap_err();

        // Verify error message
        assert_eq!(err.to_string(), "Generic error: Unauthorized");
    }

    #[test]
    fn test_set_admin_success() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("current_admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("old_token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Execute the set admin function
        let update_info = message_info(&Addr::unchecked("current_admin"), &[]);
        let new_admin = "new_admin".to_string();
        let res = execute_set_admin(deps.as_mut(), update_info, new_admin.clone()).unwrap();

        // Verify the response
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "set_admin");
        assert_eq!(res.attributes[1].key, "admin");
        assert_eq!(res.attributes[1].value, new_admin);

        // Verify state
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.admin, Addr::unchecked("new_admin"));
    }

    #[test]
    fn test_set_admin_unauthorized() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("current_admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("old_token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Attempt to set admin by a non-admin
        let update_info = message_info(&Addr::unchecked("user"), &[]);
        let new_admin = "new_admin".to_string();
        let err = execute_set_admin(deps.as_mut(), update_info, new_admin).unwrap_err();

        // Verify error message
        assert_eq!(err.to_string(), "Generic error: Unauthorized");
    }

    #[test]
    fn test_query_token_configs() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Query token configs
        let res = query_token_configs(deps.as_ref()).unwrap();

        // Verify the response
        assert_eq!(res.tokens.len(), 2);
        assert_eq!(res.tokens[0].denom, Denom::Native(NATIVE_DENOM.to_string()));
        assert_eq!(res.tokens[0].amount, Uint128::new(DEFAULT_NATIVE_AMOUNT));
        assert_eq!(
            res.tokens[1].denom,
            Denom::Cw20(Addr::unchecked("token_contract"))
        );
        assert_eq!(res.tokens[1].amount, Uint128::new(DEFAULT_CW20_AMOUNT));
    }

    #[test]
    fn test_query_rate_limit() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Query rate limit
        let res = query_rate_limit(deps.as_ref()).unwrap();

        // Verify the response
        assert_eq!(res.rate_limit_seconds, DEFAULT_RATE_LIMIT);
    }

    #[test]
    fn test_query_user_info_existing_user() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Set user info
        let user_addr = Addr::unchecked("user1");
        let user_info = UserInfo {
            last_claim_time: env.block.time.seconds(),
        };
        USER_CLAIMS
            .save(&mut deps.storage, &user_addr, &user_info)
            .unwrap();

        // Query user info
        let res = query_user_info(deps.as_ref(), "user1".to_string()).unwrap();

        // Verify the response
        assert_eq!(res.user_info, Some(user_info));
    }

    #[test]
    fn test_query_user_info_nonexistent_user() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Query user info for a non-existent user
        let res = query_user_info(deps.as_ref(), "nonexistent_user".to_string()).unwrap();

        // Verify the response
        assert_eq!(res.user_info, None);
    }

    #[test]
    fn test_query_admin() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Query admin
        let res = query_admin(deps.as_ref()).unwrap();

        // Verify the response
        assert_eq!(res.admin, "admin".to_string());
    }

    #[test]
    fn test_query_balance() {
        // Mock native balance
        let mut deps = mock_dependencies_with_balance(&[Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000),
        }]);

        // Mock balance for CW20 tokens
        let cw20_token_address = String::from("token_contract");

        // Clone necessary variables to avoid moving them into the closure
        let cw20_token_address_clone = cw20_token_address.clone();

        // Mock the balance
        deps.querier.update_wasm(move |query| {
            let cw20_token_address = cw20_token_address_clone.clone();
            let env = mock_env();

            match query {
                WasmQuery::Smart { contract_addr, msg } => {
                    if contract_addr == &cw20_token_address.clone() {
                        if let Ok(cw20::Cw20QueryMsg::Balance { address }) = from_json(&msg) {
                            if address == env.contract.address.to_string() {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&BalanceResponse {
                                        balance: Uint128::from(100_000_000u128),
                                    })
                                    .unwrap(),
                                ));
                            }
                        }
                    }
                    SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: "".to_string(),
                    })
                }
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "".to_string(),
                }),
            }
        });

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Query balances
        let res = query_balance(deps.as_ref(), env.clone()).unwrap();

        // Verify the response
        assert_eq!(res.balances.len(), 2);

        // Verify native balance
        if let Balance::Native(native_balance) = &res.balances[0] {
            assert_eq!(native_balance.0.len(), 1);
            assert_eq!(native_balance.0[0].denom, NATIVE_DENOM);
            assert_eq!(native_balance.0[0].amount, Uint128::new(100_000));
        } else {
            panic!("Expected native balance");
        }

        // Verify CW20 balance
        if let Balance::Cw20(cw20_balance) = &res.balances[1] {
            assert_eq!(cw20_balance.address, Addr::unchecked("token_contract"));
            assert_eq!(cw20_balance.amount, Uint128::new(100_000_000));
        } else {
            panic!("Expected CW20 balance");
        }
    }

    #[test]
    fn test_query_can_user_claim_first_time_user() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Query claim status for a first-time user
        let res = query_can_user_claim(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();

        // Verify the response
        assert!(res.can_claim);
        assert_eq!(res.seconds_until_next_claim, 0);
    }

    #[test]
    fn test_query_can_user_claim_within_rate_limit() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Set user info with a recent claim
        let user_addr = Addr::unchecked("user1");
        let user_info = UserInfo {
            last_claim_time: env.block.time.seconds(),
        };
        USER_CLAIMS
            .save(&mut deps.storage, &user_addr, &user_info)
            .unwrap();

        // Query claim status within the rate limit period
        let res = query_can_user_claim(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();

        // Verify the response
        assert!(!res.can_claim);
        assert!(res.seconds_until_next_claim > 0);
    }

    #[test]
    fn test_query_can_user_claim_after_rate_limit() {
        let mut deps = mock_dependencies();

        // Initialize the contract
        let init_msg = InstantiateMsg {
            admin: Some(Addr::unchecked("admin")),
            tokens: vec![
                TokenConfig {
                    denom: Denom::Native(NATIVE_DENOM.to_string()),
                    amount: Uint128::new(DEFAULT_NATIVE_AMOUNT),
                },
                TokenConfig {
                    denom: Denom::Cw20(Addr::unchecked("token_contract")),
                    amount: Uint128::new(DEFAULT_CW20_AMOUNT),
                },
            ],
            rate_limit_seconds: Some(DEFAULT_RATE_LIMIT),
        };
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "token"));
        let mut env = mock_env();
        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        // Set user info with an old claim
        let user_addr = Addr::unchecked("user1");
        let old_claim_time = env.block.time.seconds() - DEFAULT_RATE_LIMIT - 1;
        let user_info = UserInfo {
            last_claim_time: old_claim_time,
        };
        USER_CLAIMS
            .save(&mut deps.storage, &user_addr, &user_info)
            .unwrap();

        // Query claim status after the rate limit period has passed
        env.block.time = env.block.time.plus_seconds(DEFAULT_RATE_LIMIT + 1);
        let res = query_can_user_claim(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();

        // Verify the response
        assert!(res.can_claim);
        assert_eq!(res.seconds_until_next_claim, 0);
    }
}
