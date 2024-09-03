use socket::WebSocketServer;


pub mod socket;
pub mod utils;
pub mod processor;
pub mod message;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let server = WebSocketServer::new();
    server.run().await
}