use std::{str::FromStr, sync::Arc};

use chrono::Utc;
use rocket::{
    State,
    futures::{SinkExt, StreamExt},
    get, post,
    serde::json::Json,
    tokio,
};
use rocket_ws::result::Error as WsError;
use rocket_ws::{Channel, WebSocket};

use surrealdb_types::{Datetime, ToSql};
use tokio::sync::mpsc;
use validator::ValidationError;

use crate::{
    AppResult, DB, WS,
    chat::model::{
        Conversation, ConversationRequest, ConversationResponse, Message, MessageRequest,
        MessageResponse, MessageStatus, WsEvent,
    },
    db::parse_thing,
    error::AppError,
    jwt::AuthUser,
};

#[post("/create-conversation?<uid>")]
pub async fn create_conversation(
    uid: &str,
    auth_user: AuthUser,
    db: &State<DB>,
) -> AppResult<Json<ConversationResponse>> {
    let uid = parse_thing(uid)?;
    let myid = parse_thing(&auth_user.user_id)?;
    let res: Option<Conversation> = db
        .create("conversation")
        .content(ConversationRequest {
            created_at: Datetime::from(Utc::now()),
            pair_key: [uid.to_sql(), myid.to_sql()].join("_"),
            participants: vec![uid, myid],
        })
        .await?;
    let _conversation = res.ok_or(AppError::XCustomMessage("Failed to create Conversation"))?;
    Ok(Json(_conversation.into()))
}

#[get("/get-conversation-of-user?<uid>")]
pub async fn get_conversation_of_user(
    uid: &str,
    auth_user: AuthUser,
    db: &State<DB>,
) -> AppResult<Json<ConversationResponse>> {
    let pair_key = [uid, &auth_user.user_id].join("_");
    let res: Option<Conversation> = db
        .query("SELECT * FROM conversation WHERE pair_key=$key LIMIT 1")
        .bind(("key", pair_key))
        .await?
        .take::<Option<Conversation>>(0)?;
    let con = res.ok_or(AppError::XCustomMessage("Conversation not found"))?;
    Ok(Json(con.into()))
}

#[post("/send-message", data = "<req>")]
pub async fn send_message_request(
    req: Json<MessageRequest>,
    db: &State<DB>,
    auth_user: AuthUser,
) -> AppResult<Json<MessageResponse>> {
    let msg = save_message(db.inner().clone(), req.into_inner(), auth_user.user_id).await?;
    Ok(Json(msg.into()))
}

pub async fn save_message(db: DB, req: MessageRequest, auth_user: String) -> AppResult<Message> {
    let res: Option<Message> = db
        .query(
            "
            CREATE message SET
            conversation_id= $conid,
            sender_id= $uid,
            text=$text;
        ",
        )
        .bind(("conid", parse_thing(&req.conversation_id)?))
        .bind(("uid", parse_thing(&auth_user)?))
        .bind(("text", req.text.clone()))
        .await?
        .take(0)?;
    let res = res.ok_or(AppError::XCustomMessage("Cannot send message"))?;
    Ok(res)
}

#[get("/get-messages?<conid>&<cursor>&<limit>")]
pub async fn get_messages(
    conid: &str,
    cursor: Option<String>,
    limit: Option<u32>,
    db: &State<DB>,
    auth_user: AuthUser,
) -> AppResult<Json<Vec<MessageResponse>>> {
    let participants = verify_member(db, conid.to_string(), auth_user.user_id.clone()).await?;
    if !participants.contains(&auth_user.user_id) {
        return Err(AppError::Jwt(jsonwebtoken::errors::new_error(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        )));
    }
    let limit = limit.unwrap_or(20);
    let sql = if cursor.is_some() {
        "
           SELECT *
           FROM message
           WHERE conversation_id = $cid
           AND created_at < $cursor
           ORDER BY created_at DESC
           LIMIT $limit
           "
    } else {
        "
           SELECT *
           FROM message
           WHERE conversation_id = $cid
           ORDER BY created_at DESC
           LIMIT $limit
           "
    };
    {
        let mut res = db.query(sql).bind(("cid", parse_thing(conid)?));

        if let Some(cursor) = cursor {
            let cursor = Datetime::from_str(&cursor).map_err(|_| {
                AppError::ValidationError(ValidationError::new("Invalid cursor format"))
            })?;
            res = res.bind(("cursor", cursor));
        }
        let messages: Vec<Message> = res.bind(("limit", limit)).await?.take(0)?;
        let messages: Vec<MessageResponse> = messages.into_iter().map(Into::into).collect();
        return Ok(Json(messages));
    }
}

async fn mark_delivered(db: DB, msg_id: String) -> AppResult<()> {
    let res = db
        .query("UPDATE $mid SET status=$status")
        .bind(("mid", parse_thing(&msg_id)?))
        .bind(("status", MessageStatus::Delivered))
        .await?;
    res.check()?;
    Ok(())
}

async fn mark_seen(db: DB, message_id: String) -> Result<String, AppError> {
    let now = Datetime::from(Utc::now());

    let res = db
        .query("UPDATE $message SET status=$status, read_at=$time")
        .bind(("message", parse_thing(&message_id)?))
        .bind(("status", MessageStatus::Seen))
        .bind(("time", now.clone()))
        .await?;
    res.check()?;
    Ok(now.to_string())
}

#[get("/ws/<conid>")]
pub async fn ws(
    conid: &str,
    ws: WebSocket,
    manager: &State<WS>,
    db: &State<DB>,
    auth_user: AuthUser,
) -> AppResult<Channel<'static>> {
    let participants = verify_member(db, conid.to_string(), auth_user.user_id.clone()).await?;
    let db = Arc::clone(db);
    let manager = Arc::clone(manager);
    let conid = conid.to_string();
    let recipient = participants
        .iter()
        .find(|e| **e != auth_user.user_id)
        .cloned()
        .ok_or(AppError::XCustomMessage("Recipient not found"))?;
    Ok(ws.channel(move |mut stream| {
        Box::pin(async move {
            let (tx, mut rx) = mpsc::unbounded_channel::<String>();
            manager.connect(auth_user.user_id.clone(), tx);

            loop {
                tokio::select! {

                    /* -------- Incoming WS message from client -------- */
                    Some(frame) = stream.next() => {
                        let frame = frame?;
                        if !frame.is_text() { continue; }
                        let event: WsEvent =serde_json::from_str(frame.to_text()?)
                                                        .map_err(|_| AppError::XCustomMessage("Invalid Message"))?;

                        match event {
                            WsEvent::Message { message } => {
                                let payload=send_message(db.clone(),message,conid.clone(),auth_user.user_id.clone()).await?;
                                manager.send_to(recipient.clone(), payload)?;
                            },
                            WsEvent::Delivered { message_id } => {
                                mark_delivered(db.clone(), message_id.clone()).await?;
                                let payload = serde_json::to_string( &WsEvent::Delivered { message_id }).map_err(|_| AppError::XCustomMessage("Serialize failed"))?;
                                manager.send_to(recipient.clone(), payload)?;
                            },
                            WsEvent::Seen { message_id } => {
                               let read_time= mark_seen(db.clone(), message_id.clone()).await?;
                               manager.send_to(recipient.clone(), read_time)?;
                            }
                        }

                    }


                    /* -------- Message coming FROM manager → send to client -------- */
                    Some(outgoing) = rx.recv() => {
                        stream.send(outgoing.into()).await?;
                    }


                    /* -------- Socket closed -------- */
                    else => break,
                }
            }

            manager.disconnect(&auth_user.user_id);

            Ok::<(), WsError>(())
        })
    }))
}

async fn verify_member(
    db: &State<DB>,
    conid: String,
    user_id: String,
) -> Result<Vec<String>, AppError> {
    let mut res = db
        .query(
            "
        SELECT *
        FROM conversation
        WHERE id = $cid
        AND participants CONTAINS $user
        ",
        )
        .bind(("cid", parse_thing(&conid)?))
        .bind(("user", parse_thing(&user_id)?))
        .await?;

    let conv: Option<Conversation> = res.take(0)?;

    let conv = conv.ok_or(AppError::Jwt(jsonwebtoken::errors::new_error(
        jsonwebtoken::errors::ErrorKind::InvalidToken,
    )))?;
    Ok(conv.participants.iter().map(|e| e.to_sql()).collect())
}

async fn send_message(
    db: DB,
    message: String,
    conid: String,
    user_id: String,
) -> AppResult<String> {
    let message = save_message(
        db.clone(),
        MessageRequest {
            conversation_id: conid.clone(),
            text: message,
        },
        user_id,
    )
    .await?;
    let message: MessageResponse = message.into();

    let payload = serde_json::to_string(&message)
        .map_err(|_| AppError::XCustomMessage("Serialize failed"))?;
    Ok(payload)
}
