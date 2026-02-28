use rocket::{Response, http::Status, response::Responder};

use rocket_ws::result::Error as WsError;
use std::io::Cursor;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Rocket(#[from] rocket::Error),

    #[error(transparent)]
    Surreal(#[from] surrealdb::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("{0}")]
    XCustomMessage(&'static str),

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    PasswordHash(#[from] argon2::password_hash::Error),

    #[error(transparent)]
    RegexError(#[from] regex::Error),

    #[error(transparent)]
    ValidationError(#[from] validator::ValidationError),

    #[error(transparent)]
    ValidationErrors(#[from] validator::ValidationErrors),

    #[error(transparent)]
    WSError(#[from] rocket_ws::result::Error),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let msg = self.to_string();
        let status = match self {
            AppError::Jwt(_) => Status::Unauthorized,
            AppError::ValidationError(_) => Status::BadRequest,
            AppError::ValidationErrors(_) => Status::BadRequest,
            _ => Status::InternalServerError,
        };

        Response::build()
            .status(status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(msg.len(), Cursor::new(msg))
            .ok()
    }
}

impl From<AppError> for WsError {
    fn from(e: AppError) -> Self {
        WsError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    }
}
