use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
};
use backend::WebSocketBackend;
use flume::Receiver;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::from_str;
pub use session::Session;
use tracing::log;

use crate::{
    message::{Command, SocketResponse},
    utils::id_util,
};

mod backend;
mod session;

pub struct WebSocketServer {
    backend: WebSocketBackend,
}

impl WebSocketServer {
    pub fn new() -> &'static Self {
        let server = WebSocketServer {
            backend: WebSocketBackend::new(),
        };
        Box::leak(Box::new(server))
    }
}

impl WebSocketServer {
    pub async fn run(&'static self) {
        log::info!("[Server]正在启动WebSocket服务...");
        let app = axum::Router::new()
            .route("/connect", get(WebSocketServer::handler))
            .with_state(&self.backend);
        self.backend.start_session_chekcer();
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
        log::info!("[Server]WebSocket服务启动完成.访问地址: ws://127.0.0.1:8000/connect");
        axum::serve(listener, app).await.unwrap();
    }

    async fn handler(
        ws: WebSocketUpgrade,
        State(state): State<&'static WebSocketBackend>,
    ) -> Response {
        ws.on_upgrade(|socket| Self::handle_socket(socket, state))
    }

    async fn handle_socket(socket: WebSocket, backend: &'static WebSocketBackend) {
        let (sink, receiver) = socket.split();
        let (tx, rx) = flume::bounded::<Message>(128);
        let session = backend.create_session(tx.clone());

        let mut send_task = tokio::spawn(async {
            Self::sink_event_loop(rx, sink).await;
        });

        let ss = session.clone();
        let mut recv_task = tokio::spawn(async {
            Self::receive_event_loop(receiver, ss, backend).await;
        });

        tokio::select! {
            _ = (&mut send_task) => {
                recv_task.abort();
            },
            _ = (&mut recv_task) => {
                send_task.abort();
            }
        }

        drop(tx);
        backend.drop_session(session.id.as_str())
    }

    async fn sink_event_loop(rx: Receiver<Message>, mut sink: SplitSink<WebSocket, Message>) {
        while let Ok(msg) = rx.recv_async().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    }

    async fn receive_event_loop(
        mut receiver: SplitStream<WebSocket>,
        session: Arc<Session>,
        backend: &'static WebSocketBackend,
    ) {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        session.update_active_time();
                        let msg = text.trim();
                        // 是否是ping消息
                        if msg.to_lowercase().as_str() == "ping" {
                            let _ = session.send_message("pong");
                            continue;
                        }
                        if let Ok(mut command) = from_str::<Command>(msg) {
                            if command.request_id.is_none() {
                                command.request_id = Some(id_util::uuid())
                            }
                            backend
                                .get_command_dispatcher()
                                .dispatch(session.clone(), &command);
                        } else {
                            let response = SocketResponse::<String>::default().with_bad_command();
                            let _ = session.send_response(response);
                        }
                    }
                    Message::Ping(msg) => {
                        session.update_active_time();
                        let _ = session.send_message(Message::Pong(msg));
                    }
                    Message::Pong(msg) => {
                        session.update_active_time();
                        let _ = session.send_message(Message::Ping(msg));
                    }
                    Message::Close(status) => {
                        let _ = session.send_message(Message::Close(status));
                        backend.drop_session(session.id.as_str());
                        break;
                    }
                    Message::Binary(_) => {
                        let _ = session
                            .send_message(Message::from("Binary data type is not supported"));
                    }
                }
            }
        }
    }
}
