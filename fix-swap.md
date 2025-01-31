# Working with fungible tokens in cosmwasm: Building a Fix-rate swap

Embark on an exhilarating journey to master the art of token transfers in CosmWasm! 
Whether you're dealing with native tokens or the versatile CW20 tokens, this 
step-by-step guide will equip you with the knowledge and skills to execute flawless 
transfers within your smart contract. Get ready to dive into the world of blockchain 
with confidence and finesse!

In this guide, we'll walk you through the entire process of building a swap smart contract using 
CosmWasm. Whether you're a seasoned blockchain developer or a curious newcomer, this tutorial 
will equip you with the knowledge and tools to transfer fungible tokens between accounts.

## Step 1: Set Up Your Development Environment

1. **Install Rust and Cargo**: Ensure you have Rust and Cargo installed on your machine. You can install them using `rustup`:

   ```sh
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        source $HOME/.cargo/env
   ```

## Step 2: Scaffold the Project Using `cw-template`

1. **Install `cargo-generate`**: You can install `cargo-generate` using the command below:

    ```sh
        cargo install cargo-generate
    ```

2. **Generate the Project**: Use the cw-template to scaffold a new project:

    ```sh
        cargo generate --git https://github.com/CosmWasm/cw-template.git --name swap
        cd swap
    ```

## Step 3: Define Data Structures

1. Data structure is keep track of exchange rate 

2. We define the data structure in `src/state.rs` like so:

    ```rust
        // src/state.rs
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
    ```

## Step 4: Define entry points.

1. Entry points as their name suggests are exposed functions that can be used to trigger a contract.

2. Commonly used entry points include `instantiate`, `execute`, and `query`.
    Less common ones include `sudo` and `migrate`.

3. Modify `src/msg` like so:

    ```rust
        use cosmwasm_schema::cw_serde;
        use cosmwasm_std::Addr;

        use crate::state::TokenType;

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
                token_type: TokenType,
                recipient: Addr,
                amount_in: u128,
            },
        }

        #[cw_serde]
        pub enum QueryMsg {
            
        }
    ```

## Step 5: Implement Instantiate Function

1. Update the `instantiate` function in `src/contract.rs` like so:

    ```rust
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
            base_token: base_token.clone(),
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
            .add_attribute("base_token", base_token.to_string())
            .add_attribute("quote_token", quote_token.to_string()))
    }
    ```


## Step 6: Implement Execute Functions

1. Update the `execute` function in `src/contract.rs` like so:
    ```rust
    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(deps: DepsMut, _env: Env, _info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Swap {
                base_token,
                quote_token,
                token_type,
                recipient,
                amount_in,
            } => execute::execute_swap(
                deps,
                base_token,
                quote_token,
                token_type,
                recipient,
                amount_in,
            ),
        }
    }
    ```

2. Implement the execute sub-functions. Modify the `execute` mod in `src/contract.rs` like so:

    ```rust
       pub mod execute {
            use cosmwasm_std::{Coin, CosmosMsg, WasmMsg};
            use cw20::Cw20ExecuteMsg;
            use cosmwasm_std::Uint128;

            use crate::state::TokenType;

            use super::*;

            pub fn execute_swap(
                deps: DepsMut,
                info: MessageInfo,
                base_token: Addr,
                quote_token: Addr,
                token_type: TokenType,
                recipient: Addr,
                amount_in: u128,
            ) -> Result<Response, ContractError> {
                // Get exchange rate
                let exchange_rate = EXCHANGE_RATES.load(deps.storage, (base_token.clone(), quote_token.clone()))?;
                let attached = info.funds.first().unwrap();

                // Ensure correct tokens were attached
                assert!(attached.denom.eq_ignore_ascii_case("uatom"));
                assert!(attached.amount.eq(&<u128 as Into<Uint128>>::into(amount_in)));

                // get amount out
                let amount_out = amount_in * exchange_rate;

                // construct transfer msg
                let msg: CosmosMsg = if token_type == TokenType::Native {
                    // If token is native
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: recipient.to_string(),
                        amount: vec![Coin {
                            denom: "uatom".to_string(),
                            amount: amount_out.into(),
                        }],
                    })
                } else {
                    // if token is CW20 token
                    let cw20_contract_addr = quote_token;
                    let transfer_msg = Cw20ExecuteMsg::Transfer {
                        recipient: recipient.to_string(),
                        amount: amount_out.into(),
                    };
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: cw20_contract_addr.to_string(),
                        msg: to_json_binary(&transfer_msg)?,
                        funds: vec![],
                    })
                };

                Ok(Response::new()
                    .add_message(msg)
                    .add_attribute("action", "swap"))
            }
        }
    ```

## Step 7: Build project

1. Run the command below to build smart contract.

    ```sh
        cargo wasm
    ```

## Congratulations! 🥂

You've successfully built and tested a fixed rate smart contract. This tutorial has equipped you with the skills to:

- Bootstrap a new CosmWasm project.
- Tranfer native tokens
- Transfer fungible tokens

## Resources

- [CosmWasm Book](https://book.cosmwasm.com/).
- [cw-template](https://github.com/CosmWasm/cosmwasm-template).
- [Code repository](github.com/cenwadike/cosmos-guides)