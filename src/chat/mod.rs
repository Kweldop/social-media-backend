use crate::chat::chat_service::*;
use rocket::{Route, routes};

mod chat_service;
pub mod model;

pub fn routes() -> Vec<Route> {
    routes![
        create_conversation,
        get_conversation_of_user,
        send_message_request,
        get_messages,
        ws
    ]
}
