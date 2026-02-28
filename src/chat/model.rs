use serde::{Deserialize, Serialize};
use surrealdb_types::{Datetime, RecordId, SurrealValue, ToSql};

#[derive(Debug, Deserialize, SurrealValue)]
pub struct Conversation {
    pub id: RecordId,
    pub created_at: Datetime,
    pub pair_key: String,
    pub participants: Vec<RecordId>,
}
#[derive(Debug, Serialize, SurrealValue)]
pub struct ConversationRequest {
    pub created_at: Datetime,
    pub pair_key: String,
    pub participants: Vec<RecordId>,
}

#[derive(Debug, Deserialize, Serialize, SurrealValue)]
pub struct ConversationResponse {
    pub id: String,
    pub created_at: Datetime,
    pub pair_key: String,
    pub participants: Vec<String>,
}

impl From<Conversation> for ConversationResponse {
    fn from(value: Conversation) -> Self {
        Self {
            id: value.id.to_sql(),
            created_at: value.created_at,
            pair_key: value.pair_key,
            participants: value.participants.into_iter().map(|e| e.to_sql()).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, SurrealValue)]
pub struct Message {
    pub id: RecordId,
    pub conversation_id: RecordId,
    pub created_at: Datetime,
    pub read_at: Option<Datetime>,
    pub sender_id: RecordId,
    pub status: MessageStatus,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRequest {
    pub conversation_id: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, SurrealValue)]
#[serde(rename_all = "UPPERCASE")]
pub enum MessageStatus {
    Sent,
    Delivered,
    Seen,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub conversation_id: String,
    pub created_at: Datetime,
    pub read_at: Option<Datetime>,
    pub sender_id: String,
    pub status: MessageStatus,
    pub text: String,
}

impl From<Message> for MessageResponse {
    fn from(msg: Message) -> Self {
        Self {
            id: msg.id.to_sql(),
            conversation_id: msg.conversation_id.to_sql(),
            created_at: msg.created_at,
            read_at: msg.read_at,
            sender_id: msg.sender_id.to_sql(),
            status: msg.status,
            text: msg.text,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WsEvent {
    Message { message: String },
    Delivered { message_id: String },
    Seen { message_id: String },
}
