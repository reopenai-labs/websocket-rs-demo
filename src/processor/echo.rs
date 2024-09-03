use std::sync::Arc;

use serde_json::Value;

use crate::{
    message::{Command, SocketResponse},
    socket::Session,
};

use super::CommandProcessor;

pub struct EchoCommandProcessor {}

impl EchoCommandProcessor {
    pub fn new() -> Self {
        EchoCommandProcessor {}
    }
}

impl CommandProcessor for EchoCommandProcessor {
    fn process(&self, session: Arc<Session>, command: &Command) {
        let mut resoponse = SocketResponse::<Value>::from(command);
        if let Some(args) = &command.args {
            resoponse = resoponse.set_data(args.clone())
        }
        _ = session.send_response(resoponse);
    }

    fn matches(&self, command: &Command) -> bool {
        return command.op.as_str() == "echo";
    }
    
    fn get_name(&self) -> &str {
        "echo"
    }
}
