#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use execute::{create_post, delete_post, update_post};
use query::{query_index, query_post, query_user_posts};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Index, Post, INDEX, POSTS, USER_POSTS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:blog";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

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
        ExecuteMsg::DeletePost { id } => delete_post(deps, info, id),
    }
}

pub mod execute {
    use crate::state::USER_POSTS;

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
        posts.push(new_post);
        USER_POSTS.save(deps.storage, info.sender.to_string(), &posts)?;

        Ok(Response::new()
            .add_attribute("method", "create_post")
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
        let mut posts = USER_POSTS
            .may_load(deps.storage, info.sender.to_string())?
            .unwrap_or_default();

        let post_index = posts.iter().position(|post| post.id == id);
        let post_id;

        match post_index {
            Some(index) => {
                post_id = index;
            }
            None => {
                return Err(ContractError::PostNotFound {});
            }
        }
        // Only author can update
        if post.author != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(new_title) = title {
            post.title = new_title.clone();
            posts[post_id].title = new_title;
        }

        if let Some(new_content) = content {
            post.content = new_content.clone();
            posts[post_id].content = new_content;
        }

        post.updated_at = env.block.time.seconds();
        posts[post_id].updated_at = env.block.time.seconds();

        POSTS.save(deps.storage, id, &post)?;

        USER_POSTS.save(deps.storage, info.sender.to_string(), &posts)?;

        Ok(Response::new()
            .add_attribute("method", "update_post")
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

        let post_index = posts.iter().position(|post| post.id == id);

        match post_index {
            Some(index) => {
                posts.remove(index);
                USER_POSTS.save(deps.storage, info.sender.to_string(), &posts)?;

                Ok(Response::new()
                    .add_attribute("method", "delete_post")
                    .add_attribute("post_id", id.to_string()))
            }
            None => return Err(ContractError::PostNotFound {}),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CurrentIndex {} => to_json_binary(&query_index(deps)?),
        QueryMsg::GetPost { id } => to_json_binary(&query_post(deps, id)?),
        QueryMsg::GetUserPosts { user } => to_json_binary(&query_user_posts(deps, user)?),
    }
}

pub mod query {
    use crate::msg::{GetIndexResponse, GetPostResponse, GetUserPostResponse};

    use super::*;

    pub fn query_index(deps: Deps) -> StdResult<GetIndexResponse> {
        let current_index = INDEX.load(deps.storage)?.current_index;

        Ok(GetIndexResponse { current_index })
    }

    pub fn query_post(deps: Deps, id: u64) -> StdResult<GetPostResponse> {
        let post = POSTS.load(deps.storage, id)?;

        Ok(GetPostResponse { post })
    }

    pub fn query_user_posts(deps: Deps, user: String) -> StdResult<GetUserPostResponse> {
        let state = USER_POSTS.load(deps.storage, user)?;
        Ok(GetUserPostResponse { posts: state })
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{GetIndexResponse, GetPostResponse, GetUserPostResponse};

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

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetUserPosts {
                user: "anyone".to_string(),
            },
        )
        .unwrap();
        let value: GetUserPostResponse = from_json(&res).unwrap();

        // Should be assigned current index
        assert_eq!(new_index, value.posts[0].id);

        // Check title
        assert_eq!(title, value.posts[0].title);

        // Check content
        assert_eq!(content, value.posts[0].content);

        // Check author
        assert_eq!(author, value.posts[0].author);
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
