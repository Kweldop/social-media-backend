use crate::AppResult;
use crate::error::AppError;
use dashmap::DashMap;
use tokio::sync::mpsc::UnboundedSender;

pub struct WsManager {
    users: DashMap<String, UnboundedSender<String>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            users: DashMap::new(),
        }
    }

    pub fn connect(&self, user: String, tx: UnboundedSender<String>) {
        self.users.insert(user, tx);
    }

    pub fn disconnect(&self, user: &str) {
        self.users.remove(user);
    }

    pub fn send_to(&self, user: String, msg: String) -> AppResult<()> {
        if let Some(tx) = self.users.get(&user) {
            let _ = tx
                .send(msg)
                .map_err(|_| AppError::XCustomMessage("Failed to send message"))?;
        }
        Ok(())
    }
}
