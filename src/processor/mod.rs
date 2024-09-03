use std::sync::Arc;

use echo::EchoCommandProcessor;
use tracing::log;

use crate::{message::Command, socket::Session};
mod echo;

/// CommandProcessor trait 用于处理和匹配命令
trait CommandProcessor: Send + Sync {
    /// 处理命令
    fn process(&'static self, session: Arc<Session>, command: &Command);
    /// 检查命令能否被此处理器处理
    fn matches(&'static self, command: &Command) -> bool;
    // 获取处理器的名字.处理器的名字必须唯一
    fn get_name(&self) -> &str;
}

pub struct CommandDispatcher {
    processors: Vec<Box<dyn CommandProcessor>>,
}

impl CommandDispatcher {
    pub fn new() -> Self {
        let mut instance: CommandDispatcher = CommandDispatcher {
            processors: Vec::new(),
        };
        instance.register(Box::new(EchoCommandProcessor::new()));
        return instance;
    }

    fn register(&mut self, processor: Box<dyn CommandProcessor>) {
        let name = processor.get_name();
        log::info!("[CommandProcessor]已注册{}处理器", name);
        self.processors.push(processor)
    }
}

impl CommandDispatcher {
    pub fn dispatch(&'static self, session: Arc<Session>, command: &Command) {
        for processor in self.processors.iter() {
            if processor.matches(command) {
                processor.process(session.clone(), command);
                break;
            }
        }
    }
}
