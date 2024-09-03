use std::hash::Hash;
use std::sync::RwLock;

use anyhow::Error;
use axum::extract::ws::Message;
use flume::{Sender, TrySendError};
use serde::Serialize;
use tracing::log;

use crate::{
    message::SocketResponse,
    utils::{id_util, time_util},
};

#[derive(Debug)]
pub struct Session {
    pub id: String,
    sink: Sender<Message>,
    last_active_timestamp: RwLock<u128>,
}

impl Session {
    pub fn new(sink: Sender<Message>) -> Self {
        Session {
            sink,
            id: id_util::uuid(),
            last_active_timestamp: RwLock::new(time_util::current_timestamp()),
        }
    }

    pub fn update_active_time(&self) {
        if let Ok(mut last_active) = self.last_active_timestamp.write() {
            *last_active = time_util::current_timestamp();
        };
    }

    pub fn is_expired(&self, current_timestamp: u128) -> bool {
        let timestamp = self.last_active_timestamp.read().unwrap();
        (current_timestamp - *timestamp) >= 120000
    }

    pub fn send_message<T: Into<Message>>(&self, message: T) -> anyhow::Result<()> {
        let result = self.sink.try_send(message.into());
        if result.is_err() {
            let err = result.err().unwrap();
            match err {
                TrySendError::Full(_) => {
                    let len = self.sink.len();
                    log::info!(
                        "[Session]消息堆积太多已被阻塞,sessionId={}.message_number={}",
                        self.id,
                        len
                    );
                }
                TrySendError::Disconnected(_) => {
                    return Err(Error::from(err));
                }
            }
        }
        Ok(())
    }

    pub fn send_response<T: Serialize>(&self, response: SocketResponse<T>) -> anyhow::Result<()> {
        let data = serde_json::to_string(&response)?;
        self.send_message(data)
    }
}

impl Hash for Session {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Session {}
