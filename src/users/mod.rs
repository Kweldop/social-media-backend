use rocket::{Route, routes};

use crate::users::user_service::*;

pub mod model;
pub mod user_service;

pub fn routes() -> Vec<Route> {
    routes![
        login_request,
        register_request,
        get_user_details_from_token,
        refresh_token,
        get_follower_list,
        follow_user,
        get_following_list,
        unfollow_user,
        update_profile_picture
    ]
}
