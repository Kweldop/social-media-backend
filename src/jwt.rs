use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};
use serde::{Deserialize, Serialize};
use std::env;

use crate::{AppResult, error::AppError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
    token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(PartialEq)]
enum TokenType {
    AccessToken,
    RefreshToken,
}

pub fn jwt_secret() -> Vec<u8> {
    env::var("JWT_SECRET")
        .expect("JWT_SECRET not set")
        .into_bytes()
}

pub fn generate_access_token(user_id: String) -> AppResult<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        token_type: TokenType::AccessToken,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&jwt_secret()),
    )?;

    Ok(token)
}

pub fn verify_token(token: String) -> AppResult<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&jwt_secret()),
        &Validation::default(),
    )?;
    if data.claims.token_type != TokenType::AccessToken {
        return Err(AppError::XCustomMessage("Invalid Token"));
    }

    Ok(data.claims)
}

pub fn generate_refresh_token(user_id: String) -> AppResult<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .unwrap()
        .timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        exp: expiration,
        token_type: TokenType::RefreshToken,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&jwt_secret()),
    )?;
    Ok(token)
}

pub fn refresh_access_token(refresh_token: &str) -> AppResult<String> {
    let decoded = decode::<Claims>(
        refresh_token,
        &DecodingKey::from_secret(&jwt_secret()),
        &Validation::default(),
    )?;
    let claims = decoded.claims;
    if claims.token_type != TokenType::RefreshToken {
        return Err(AppError::XCustomMessage("Invalid token"));
    }

    generate_access_token(claims.sub)
}

pub struct AuthUser {
    pub user_id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = req.headers().get_one("Authorization");
        let token = match auth_header {
            Some(h) if h.starts_with("Bearer ") => &h[7..],
            _ => return Outcome::Error((Status::Unauthorized, ())),
        }
        .to_string();

        let decoded = verify_token(token);

        match decoded {
            Ok(data) => Outcome::Success(AuthUser { user_id: data.sub }),
            Err(_) => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
