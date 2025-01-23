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
    pub created_at: u64,
    pub updated_at: u64,
}

pub const INDEX: Item<Index> = Item::new("index");
pub const POSTS: Map<PostId, Post> = Map::new("posts");
pub const USER_POSTS: Map<String, Vec<PostId>> = Map::new("user_posts");
