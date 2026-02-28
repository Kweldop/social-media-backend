use once_cell::sync::Lazy;
use regex::Regex;
use rocket::{FromForm, fs::TempFile};
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, sql::Thing};
use validator::Validate;

use crate::AppResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Thing,
    pub username: String,
    pub profile_picture: Option<String>,
    pub email: String,
    pub mobile_number: String,
    pub followers_count: i64,
    pub following_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBUser {
    pub id: Thing,
    pub username: String,
    pub profile_picture: Option<String>,
    pub email: String,
    pub mobile_number: String,
    pub followers_count: i64,
    pub following_count: i64,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub profile_picture: Option<String>,
    pub email: String,
    pub followers_count: i64,
    pub following_count: i64,
    pub mobile_number: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            profile_picture: user.profile_picture,
            email: user.email,
            mobile_number: user.mobile_number,
            followers_count: user.followers_count,
            following_count: user.following_count,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Incorrect email"))]
    pub email: Option<String>,
    #[validate(length(min = 3, message = "Incorrect Username"))]
    pub username: Option<String>,
    #[validate(length(min = 6, message = "Password should be atleast 6 letters"))]
    pub password: String,
}

fn init_phone_re() -> AppResult<Regex> {
    Ok(Regex::new(r"^\+?[1-9]\d{9,14}$")?)
}
static PHONE_RE: Lazy<Regex> = Lazy::new(|| init_phone_re().unwrap());

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Incorrect email"))]
    pub email: String,
    #[validate(length(min = 3, message = "Incorrect Username"))]
    pub username: String,
    #[validate(length(min = 6, message = "Password should be atleast 6 letters"))]
    pub password: String,
    #[validate(regex(path = *PHONE_RE,message="Incorrect Mobile number"))]
    pub mobile_number: String,
}

impl From<RegisterRequest> for LoginRequest {
    fn from(value: RegisterRequest) -> Self {
        Self {
            email: Some(value.email),
            username: Some(value.username),
            password: value.password,
        }
    }
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Follow {
    pub id: Thing,
    pub follower_id: Thing,
    pub following_id: Thing,
    pub created_at: Datetime,
}

#[derive(FromForm)]
pub struct Upload<'r> {
    pub file: TempFile<'r>,
}
