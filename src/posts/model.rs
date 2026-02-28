use rocket::{FromForm, fs::TempFile};
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, sql::Thing};

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    id: Thing,
    content: String,
    caption: String,
    uid: Thing,
    likes_count: usize,
    created_at: Datetime,
}

#[derive(Debug, FromForm)]
pub struct PostFormRequest<'r> {
    pub content: TempFile<'r>,
    pub caption: String,
}

#[derive(Debug, Serialize)]
pub struct PostRequest {
    pub content: String,
    pub caption: String,
    pub uid: Thing,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: String,
    pub content: String,
    pub caption: String,
    pub uid: String,
    pub likes_count: usize,
    pub created_at: Datetime,
    pub liked_by_user: bool,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            id: post.id.to_string(),
            content: post.content,
            caption: post.caption,
            uid: post.uid.to_string(),
            likes_count: post.likes_count,
            created_at: post.created_at,
            liked_by_user: false,
        }
    }
}

#[derive(FromForm)]
pub struct FeedQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Like {
    pub id: Thing,
    pub post_id: Thing,
    pub user_ids: Vec<Thing>,
}

#[derive(Debug, Serialize)]
pub struct LikeResponse {
    pub id: String,
    pub post_id: String,
    pub user_ids: Vec<String>,
}

impl From<Like> for LikeResponse {
    fn from(like: Like) -> Self {
        Self {
            id: like.id.to_string(),
            post_id: like.post_id.to_string(),
            user_ids: like.user_ids.into_iter().map(|e| e.to_string()).collect(),
        }
    }
}
