use jsonwebtoken::crypto::{CryptoProvider, rust_crypto};
use rocket::data::{Limits, ToByteUnit};
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::{error::AppError, ws::WsManager};

mod chat;
mod common_service;
mod db;
mod error;
mod jwt;
mod posts;
mod users;
mod ws;

type WS = Arc<WsManager>;
type DB = Arc<Surreal<Client>>;
type AppResult<T> = Result<T, AppError>;

#[rocket::main]
async fn main() -> AppResult<()> {
    CryptoProvider::install_default(&rust_crypto::DEFAULT_PROVIDER).unwrap();
    dotenvy::dotenv().ok();
    std::fs::create_dir_all("data/profile-pictures").ok();
    std::fs::create_dir_all("data/posts").ok();
    let db = db::init().await?;

    rocket::build()
        .configure(rocket::Config {
            address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: 8080,
            limits: Limits::new()
                .limit("file", (20).megabytes())
                .limit("form", 20.megabytes()),
            ..Default::default()
        })
        .manage(Arc::new(db))
        .manage(Arc::new(WsManager::new()))
        .mount("/user-service", users::routes())
        .mount("/post-service", posts::routes())
        .mount("/", rocket::fs::FileServer::from("data"))
        .mount("/chat-service", chat::routes())
        .launch()
        .await?;
    Ok(())
}
