# Practical Introduction to CosmWasm: Building a Social Media Smart Contract and Deploying on Neutron

In the age of decentralized applications and blockchain technology, creating a social media platform where users can interact, post content, like posts, and comment, all while leveraging the security and transparency of the blockchain, is an exciting venture. With CosmWasm, building such a platform becomes not only feasible but also efficient and scalable

In this guide, we'll walk you through the entire process of building a social media smart contract using CosmWasm and deploying it on the Neutron blockchain. Whether you're a seasoned blockchain developer or a curious newcomer, this tutorial will equip you with the knowledge and tools to create your decentralized social media application.

Let's get started on this exciting journey!

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
        cargo generate --git https://github.com/CosmWasm/cw-template.git --name social-media-contract
        cd social-media-contract
    ```

## Step 3: Define Data Structures

1. Data structure is central to the implementation of smart contracts. 
    it defines how contract storage and by extension the blockchain state.

2. It is important to design your smart contract state for optimal writes and reads.

3. We define the data structure in `src/state` like so:

    ```rust
        // src/state

        use schemars::JsonSchema;
        use serde::{Deserialize, Serialize};

        use cosmwasm_std::Addr;
        use cw_storage_plus::{Item, Map};

        pub type PostId = u64;

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
        pub struct Index {
            pub current_index: u64,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
        pub struct Post {
            pub id: u64,
            pub title: String,
            pub content: String,
            pub author: Addr,
            pub likes: u64,
            pub likers: Vec<Addr>,
            pub comments: Vec<(Addr, String)>,
            pub created_at: u64,
            pub updated_at: u64,
        }

        pub const INDEX: Item<Index> = Item::new("index");
        pub const POSTS: Map<PostId, Post> = Map::new("posts");
        pub const USER_POSTS: Map<String, Vec<PostId>> = Map::new("user_posts");
    ```

PS: **Comment out all the red squiggly lines**

## Step 4: Define entry points.

1. Entry points as their name suggests are exposed functions that can be used to trigger a contract.

2. Commonly used entry points include `instantiate`, `execute`, and `query`.
    Less common ones include `sudo` and `migrate`.

3. Modify `src/msg` like so:

    ```rust
        use crate::state::{Post, PostId};
        use cosmwasm_schema::{cw_serde, QueryResponses};

        #[cw_serde]
        pub struct InstantiateMsg {}

        #[cw_serde]
        pub enum ExecuteMsg {
            CreatePost {
                title: String,
                content: String,
            },
            UpdatePost {
                id: u64,
                title: String,
                content: String,
            },
            LikePost {
                id: u64,
            },
            Comment {
                id: u64,
                comment: String,
            },
            DeletePost {
                id: u64,
            },
        }

        #[cw_serde]
        #[derive(QueryResponses)]
        pub enum QueryMsg {
            // CurrentIndex returns the current index as a json-encoded u64
            #[returns(GetIndexResponse)]
            CurrentIndex {},

            // GetPost returns the post as a json-encoded Post
            #[returns(GetPostResponse)]
            GetPost { id: u64 },

            // GetPost returns the users post as a json-encoded Posts
            #[returns(GetUserPostsResponse)]
            GetUserPosts { user: String },
        }

        // We define a custom struct for each query response
        #[cw_serde]
        pub struct GetIndexResponse {
            pub current_index: u64,
        }

        // We define a custom struct for each query response
        #[cw_serde]
        pub struct GetPostResponse {
            pub post: Post,
        }

        // We define a custom struct for each query response
        #[cw_serde]
        pub struct GetUserPostsResponse {
            pub posts: Vec<PostId>,
        }
    ```

## Step 5: Implement Instantiate Function

1. Update the `instantiate` function in `src/contract` like so:
    
    ```rust
        #[cfg_attr(not(feature = "library"), entry_point)]
        pub fn instantiate(
            deps: DepsMut,
            _env: Env,
            info: MessageInfo,
            _msg: InstantiateMsg,
        ) -> Result<Response, ContractError> {
            // Set contract version. Useful for migration.
            set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

            // Initialize index
            let index = Index { current_index: 0 };
            INDEX.save(deps.storage, &index)?;

            // Emit event for logging
            Ok(Response::new()
                .add_attribute("action", "instantiate")
                .add_attribute("owner", info.sender))
        }
    ```

## Step 6: Implement Execute Functions

1. Update the `execute` function in `src/contract` like so:

    ```rust
        #[cfg_attr(not(feature = "library"), entry_point)]
        pub fn execute(
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: ExecuteMsg,
        ) -> Result<Response, ContractError> {
            match msg {
                ExecuteMsg::CreatePost { title, content } => create_post(deps, env, info, title, content),
                ExecuteMsg::UpdatePost { id, title, content } => {
                    update_post(deps, env, info, id, Some(title), Some(content))
                }
                ExecuteMsg::LikePost { id } => like_post(deps, info, id),
                ExecuteMsg::Comment { id, comment } => comment_on_post(deps, info, id, comment),
                ExecuteMsg::DeletePost { id } => delete_post(deps, info, id),
            }
        }
    ```

2. Implement the execute sub-functions. Modify the `execute` mod in `src/contract` like so:

    ```rust
        pub mod execute {
            use super::*;

            pub fn create_post(
                deps: DepsMut,
                env: Env,
                info: MessageInfo,
                title: String,
                content: String,
            ) -> Result<Response, ContractError> {
                // Get post id.
                let post_id = INDEX.load(deps.storage)?.current_index + 1;

                // Remove trailing space in title and content.
                let title = title.trim().to_string();
                let content = content.trim().to_string();

                // Construct new post
                let new_post = Post {
                    id: post_id,
                    title,
                    content,
                    author: info.sender.clone(),
                    likes: 0,
                    likers: vec![],
                    comments: vec![],
                    created_at: env.block.time.seconds(),
                    updated_at: env.block.time.seconds(),
                };

                // Update index
                INDEX.save(
                    deps.storage,
                    &Index {
                        current_index: post_id,
                    },
                )?;

                // Add to post storage
                POSTS.save(deps.storage, post_id, &new_post)?;

                // Add to user post storage
                let mut posts = USER_POSTS
                    .may_load(deps.storage, info.sender.to_string())?
                    .unwrap_or_default();
                posts.push(post_id);
                USER_POSTS.save(deps.storage, info.sender.to_string(), &posts)?;

                Ok(Response::new()
                    .add_attribute("action", "create_post")
                    .add_attribute("post_id", post_id.to_string()))
            }

            pub fn update_post(
                deps: DepsMut,
                env: Env,
                info: MessageInfo,
                id: u64,
                title: Option<String>,
                content: Option<String>,
            ) -> Result<Response, ContractError> {
                let mut post = POSTS.load(deps.storage, id)?;

                // Only author can update
                if post.author != info.sender {
                    return Err(ContractError::Unauthorized {});
                }

                if let Some(new_title) = title {
                    post.title = new_title.clone();
                }

                if let Some(new_content) = content {
                    post.content = new_content.clone();
                }

                post.updated_at = env.block.time.seconds();

                POSTS.save(deps.storage, id, &post)?;

                Ok(Response::new()
                    .add_attribute("action", "update_post")
                    .add_attribute("post_id", id.to_string()))
            }

            pub fn like_post(deps: DepsMut, info: MessageInfo, id: u64) -> Result<Response, ContractError> {
                let mut post = POSTS.load(deps.storage, id)?;
                post.likes += 1;
                post.likers.push(info.sender);

                POSTS.save(deps.storage, id, &post)?;

                Ok(Response::new()
                    .add_attribute("action", "like_post")
                    .add_attribute("post_id", id.to_string()))
            }

            pub fn comment_on_post(
                deps: DepsMut,
                info: MessageInfo,
                id: u64,
                comment: String,
            ) -> Result<Response, ContractError> {
                let mut post = POSTS.load(deps.storage, id)?;

                post.comments.push((info.sender, comment));

                POSTS.save(deps.storage, id, &post)?;

                Ok(Response::new()
                    .add_attribute("action", "like_post")
                    .add_attribute("post_id", id.to_string()))
            }

            pub fn delete_post(
                deps: DepsMut,
                info: MessageInfo,
                id: u64,
            ) -> Result<Response, ContractError> {
                let post = POSTS.load(deps.storage, id)?;

                // Only author can delete
                if post.author != info.sender {
                    return Err(ContractError::Unauthorized {});
                }

                POSTS.remove(deps.storage, id);

                let mut posts = USER_POSTS
                    .may_load(deps.storage, info.sender.to_string())?
                    .unwrap_or_default();

                let post_index = posts.iter().position(|post_id| post_id == &id);

                match post_index {
                    Some(index) => {
                        posts.remove(index);
                        USER_POSTS.save(deps.storage, info.sender.to_string(), &posts)?;

                        Ok(Response::new()
                            .add_attribute("action", "delete_post")
                            .add_attribute("post_id", id.to_string()))
                    }
                    None => return Err(ContractError::PostNotFound {}),
                }
            }
        }
    ```

## Step 7: Implement Query Functions

1. Queries are gas-less interactions with the blockchain. We can use queries to make metrics available from our smart contract.

2. In our case, we use queries to get a post by it's id or to get the post of a particular user.

3. Update the `query` function in `src/contract` like so:

    ```rust
        #[cfg_attr(not(feature = "library"), entry_point)]
        pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
            match msg {
                QueryMsg::CurrentIndex {} => to_json_binary(&query_index(deps)?),
                QueryMsg::GetPost { id } => to_json_binary(&query_post(deps, id)?),
                QueryMsg::GetUserPosts { user } => to_json_binary(&query_user_posts(deps, user)?),
            }
        }
    ```

4. Implement the query sub-functions. Modify the `query` mod in `src/contract` like so:

    ```rust
        pub mod query {
            use crate::msg::{GetIndexResponse, GetPostResponse, GetUserPostsResponse};

            use super::*;

            pub fn query_index(deps: Deps) -> StdResult<GetIndexResponse> {
                let current_index = INDEX.load(deps.storage)?.current_index;

                Ok(GetIndexResponse { current_index })
            }

            pub fn query_post(deps: Deps, id: u64) -> StdResult<GetPostResponse> {
                let post = POSTS.load(deps.storage, id)?;

                Ok(GetPostResponse { post })
            }

            pub fn query_user_posts(deps: Deps, user: String) -> StdResult<GetUserPostsResponse> {
                let state = USER_POSTS.load(deps.storage, user)?;
                Ok(GetUserPostsResponse { posts: state })
            }
        }
    ```

## Step 8: Add Contract Errors

1. Add these contract errors in the `src/error.rs`

    ```rust
        use cosmwasm_std::StdError;
        use thiserror::Error;

        #[derive(Error, Debug)]
        pub enum ContractError {
            #[error("{0}")]
            Std(#[from] StdError),

            #[error("Unauthorized")]
            Unauthorized {},

            #[error("PostNotFound")]
            PostNotFound {},
        }
    ```

## Step 9: Build project

1. Run the command below to build smart contract.

    ```sh
        cargo wasm
    ```

## Step 10: Add tests

1. Tests help us gain confidence in our code.

2. Add these tests cases to the `test` mod in `src/contract` like so:

    ```rust
        #[cfg(test)]
        mod tests {
            use crate::msg::{GetIndexResponse, GetPostResponse};

            use super::*;
            use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
            use cosmwasm_std::{coins, from_json, Addr};
            use cw_storage_plus::KeyDeserialize;

            #[test]
            fn proper_initialization() {
                let mut deps = mock_dependencies();

                let msg = InstantiateMsg {};
                let info = message_info(
                    &Addr::from_vec("creator".as_bytes().to_vec()).unwrap(),
                    &coins(1_000_000, "hackATOM"),
                );

                // we can just call .unwrap() to assert this was a success
                let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
                assert_eq!(0, res.messages.len());

                // it worked, let's query the state
                let res = query(deps.as_ref(), mock_env(), QueryMsg::CurrentIndex {}).unwrap();
                let value: GetIndexResponse = from_json(&res).unwrap();
                assert_eq!(0, value.current_index);
            }

            #[test]
            fn create_post() {
                let mut deps = mock_dependencies();

                // Instantiate contract
                let msg = InstantiateMsg {};
                let info = message_info(
                    &Addr::from_vec("creator".as_bytes().to_vec()).unwrap(),
                    &coins(1_000_000, "hackATOM"),
                );
                let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

                // Create post
                let info = message_info(
                    &Addr::from_vec("anyone".as_bytes().to_vec()).unwrap(),
                    &coins(10, "hackATOM"),
                );
                let title = "title".to_string();
                let content = "content".to_string();
                let author = Addr::from_vec("anyone".as_bytes().to_vec()).unwrap();
                let msg = ExecuteMsg::CreatePost {
                    title: title.clone(),
                    content: content.clone(),
                };
                let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

                // Get update current index.
                let res = query(deps.as_ref(), mock_env(), QueryMsg::CurrentIndex {}).unwrap();
                let value: GetIndexResponse = from_json(&res).unwrap();
                let new_index = value.current_index;

                let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPost { id: 1 }).unwrap();
                let value: GetPostResponse = from_json(&res).unwrap();

                // Should be assigned current index
                assert_eq!(new_index, value.post.id);

                // Check title
                assert_eq!(title, value.post.title);

                // Check content
                assert_eq!(content, value.post.content);

                // Check author
                assert_eq!(author, value.post.author);
            }

            #[test]
            fn update_post() {
                let mut deps = mock_dependencies();

                // Instantiate contract
                let msg = InstantiateMsg {};
                let info = message_info(
                    &Addr::from_vec("creator".as_bytes().to_vec()).unwrap(),
                    &coins(1_000_000, "hackATOM"),
                );
                let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

                let info = message_info(
                    &Addr::from_vec("anyone".as_bytes().to_vec()).unwrap(),
                    &coins(10, "hackATOM"),
                );
                let title = "title".to_string();
                let content = "content".to_string();

                let msg = ExecuteMsg::CreatePost {
                    title: title.clone(),
                    content: content.clone(),
                };
                let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

                // Get update current index.
                let res = query(deps.as_ref(), mock_env(), QueryMsg::CurrentIndex {}).unwrap();
                let value: GetIndexResponse = from_json(&res).unwrap();
                let new_index = value.current_index;

                // Update post
                let info = message_info(
                    &Addr::from_vec("anyone".as_bytes().to_vec()).unwrap(),
                    &coins(10, "hackATOM"),
                );
                let another_title = "another title".to_string();
                let another_content = "another content".to_string();

                let msg = ExecuteMsg::UpdatePost {
                    id: new_index,
                    title: another_title.clone(),
                    content: another_content.clone(),
                };
                let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

                // Check updates
                let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPost { id: 1 }).unwrap();
                let value: GetPostResponse = from_json(&res).unwrap();

                // Check title
                assert_eq!(another_title, value.post.title);

                // Check content
                assert_eq!(another_content, value.post.content);
            }

            #[test]
            fn delete_post() {
                let mut deps = mock_dependencies();

                // Instantiate contract
                let msg = InstantiateMsg {};
                let info = message_info(
                    &Addr::from_vec("creator".as_bytes().to_vec()).unwrap(),
                    &coins(1_000_000, "hackATOM"),
                );
                let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

                let info = message_info(
                    &Addr::from_vec("anyone".as_bytes().to_vec()).unwrap(),
                    &coins(10, "hackATOM"),
                );
                let title = "title".to_string();
                let content = "content".to_string();

                let msg = ExecuteMsg::CreatePost {
                    title: title.clone(),
                    content: content.clone(),
                };
                let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

                // Get update current index.
                let res = query(deps.as_ref(), mock_env(), QueryMsg::CurrentIndex {}).unwrap();
                let value: GetIndexResponse = from_json(&res).unwrap();
                let new_index = value.current_index;

                // Delete post
                let info = message_info(
                    &Addr::from_vec("anyone".as_bytes().to_vec()).unwrap(),
                    &coins(10, "hackATOM"),
                );

                let msg = ExecuteMsg::DeletePost { id: new_index };
                let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

                // Check updates
                let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPost { id: 1 });

                // Check it's deleted
                assert!(res.is_err());
            }
        }
    ```

3. Run the tests with the command below:

    ```sh
        cargo test
    ```

## 11: Deployment Steps

1. Initialize node js by running:

    ```sh
        npm init -y
    ```

2. Copy the Deployment Script of the Code into `scripts/deploy.js` like so:

    ```js
    const { SigningCosmWasmClient } = require("@cosmjs/cosmwasm-stargate");
    const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
    const { GasPrice } = require("@cosmjs/stargate");
    const fs = require("fs");

    require('dotenv').config();

    const rpcEndpoint = "https://rpc-palvus.pion-1.ntrn.tech";
    const mnemonic = process.env.MNEMONIC;
    const wasmFilePath = "./artifacts/social_media.wasm";

    async function main() {
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
        prefix: "neutron",
    });

    const [firstAccount] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient.connectWithSigner(
        rpcEndpoint,
        wallet,
        {
        gasPrice: GasPrice.fromString("0.025untrn"),
        }
    );

    const wasmCode = fs.readFileSync(wasmFilePath);
    const uploadReceipt = await client.upload(firstAccount.address, wasmCode, "auto");
    console.log("Upload successful, code ID:", uploadReceipt.codeId);

    const initMsg = {}; // Your init message
    const instantiateReceipt = await client.instantiate(firstAccount.address, uploadReceipt.codeId, initMsg, "Social Media Contract", "auto");
    console.log("Contract instantiated at:", instantiateReceipt.contractAddress);
    }

    main().catch(console.error);
    ```

3. Install dependencies:

    ```sh
        npm install
    ```

4. Optimize wasm artifact

    ```sh
        docker run --rm -v "$(pwd)":/code \              
        --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
        --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
        cosmwasm/optimizer:0.16.0
    ```

5. Run the deployment:

    ```sh
        node scripts/deploy.js
    ```
![alt text](../cosmos-guides/images/successful%20deployment.png)

## Congratulations! 🥂

You've successfully built and tested a social media smart contract. This tutorial has equipped you with the skills to:

- Bootstrap a new CosmWasm project.
- Customize CRUD operations to fit your smart contract's needs.

## Resources

- [CosmWasm Book](https://book.cosmwasm.com/).
- [cw-template](https://github.com/CosmWasm/cosmwasm-template).
- [Code repository](github.com/cenwadike/cosmos-guides)