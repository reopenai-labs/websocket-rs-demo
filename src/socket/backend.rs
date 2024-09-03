use std::{borrow::Cow, sync::Arc, time::Duration};

use crate::{processor::CommandDispatcher, utils::time_util};
use axum::extract::ws::{self, CloseFrame, Message};
use dashmap::DashMap;
use flume::Sender;
use tokio::time::interval;
use tracing::log;

use super::session::Session;

pub struct WebSocketBackend {
    sessions: DashMap<String, Arc<Session>>,
    command_dispatcher: CommandDispatcher,
}

impl WebSocketBackend {
    pub fn new() -> Self {
        WebSocketBackend {
            sessions: DashMap::new(),
            command_dispatcher: CommandDispatcher::new(),
        }
    }
}

impl WebSocketBackend {
    pub fn create_session(&self, sink: Sender<Message>) -> Arc<Session> {
        let session = Arc::new(Session::new(sink));
        let duplicate = session.clone();
        self.sessions.insert(session.id.clone(), session);
        return duplicate;
    }

    pub fn drop_session<T: AsRef<str>>(&self, session_id: T) {
        let id = session_id.as_ref();
        self.sessions.remove(id);
    }

    pub fn get_command_dispatcher(&self) -> &CommandDispatcher {
        &self.command_dispatcher
    }

    pub fn start_session_chekcer(&'static self) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(120));
            log::info!("[WebSocket]已经启动Session活跃状态检测程序.每120s将会触发一次检查.");
            loop {
                interval.tick().await;
                let timestamp = time_util::current_timestamp();
                for session in self.sessions.iter() {
                    if session.is_expired(timestamp) {
                        let _ = session.send_message(Message::Close(Some(CloseFrame {
                            code: ws::close_code::NORMAL,
                            reason: Cow::from("expired"),
                        })));
                    }
                }
            }
        });
    }
}
