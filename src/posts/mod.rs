use rocket::{Route, routes};

use crate::posts::post_service::*;

pub mod model;
pub mod post_service;

pub fn routes() -> Vec<Route> {
    routes![
        post,
        get_user_posts,
        get_feed,
        get_post_by_id,
        like_post,
        get_likes
    ]
}
