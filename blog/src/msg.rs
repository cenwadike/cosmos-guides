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
