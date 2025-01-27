use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub type PostId = u64;
pub type Username = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Index {
    pub current_index: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Profile {
    pub id: u64,
    pub addr: Addr,
    pub user_name: String,
    pub about: String,
    pub image: String, // URL to image
    pub followers: Vec<String>,
    pub following: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
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

pub const PROFILE_INDEX: Item<Index> = Item::new("profile_index");
pub const POST_INDEX: Item<Index> = Item::new("post_index");
pub const PROFILES: Map<Username, Profile> = Map::new("profiles");
pub const POSTS: Map<PostId, Post> = Map::new("posts");
pub const USER_POSTS: Map<String, Vec<PostId>> = Map::new("user_posts");
