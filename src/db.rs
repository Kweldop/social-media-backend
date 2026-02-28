use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Wss},
    opt::auth::Root,
    types::RecordId,
};

use crate::{AppResult, error::AppError};

pub async fn init() -> AppResult<Surreal<Client>> {
    let db =
        Surreal::new::<Wss>("kweldop-social-06ea93v719qnpe4b0mda7l3mjg.aws-aps1.surreal.cloud")
            .await?;
    db.signin(Root {
        password: "root".to_string(),
        username: "root".to_string(),
    })
    .await?;
    db.use_ns("main").await?;
    db.use_db("main").await?;
    Ok(db)
}

pub fn parse_thing_to_record(id: &str) -> AppResult<(String, String)> {
    let (table, record_id) = id
        .split_once(':')
        .ok_or(AppError::XCustomMessage("Invalid id"))?;

    Ok((table.to_string(), record_id.to_string()))
}

pub fn parse_thing(id: &str) -> AppResult<RecordId> {
    let (table, record_id) = id
        .split_once(':')
        .ok_or(AppError::XCustomMessage("Invalid ID"))?;

    Ok(RecordId::new(table, record_id))
}
