use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
};

use crate::{AppResult, error::AppError};

pub async fn init() -> AppResult<Surreal<Client>> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.signin(Root {
        password: "root",
        username: "root",
    })
    .await?;
    db.use_ns("social_media").await?;
    db.use_db("social_media").await?;
    Ok(db)
}

pub fn parse_thing_to_record(id: &str) -> AppResult<(String, String)> {
    let (table, record_id) = id
        .split_once(':')
        .ok_or(AppError::XCustomMessage("Invalid id"))?;

    Ok((table.to_string(), record_id.to_string()))
}

pub fn parse_thing(id: &str) -> AppResult<Thing> {
    let (table, record_id) = id
        .split_once(':')
        .ok_or(AppError::XCustomMessage("Invalid ID"))?;

    Ok(Thing::from((table, record_id)))
}
