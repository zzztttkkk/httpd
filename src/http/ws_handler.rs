use std::future::Future;
use std::sync::Arc;

use tokio::sync::Mutex;

pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Pong,
    Close,
    Continuation,
}

pub trait WebSocketConn {
    fn on_message(&mut self, msg: WebSocketMessage);
    fn send(&mut self);
}

pub trait WebSocketHandler: Sync + Send {
    fn handle(&self, conn: Arc<Mutex<dyn WebSocketConn>>) -> Box<dyn Future<Output=()>>;
}
