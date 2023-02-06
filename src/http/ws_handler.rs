pub enum WebSocketMessageType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
    Continuation,
}

pub struct WebSocketMessage {
    typ: WebSocketMessageType,
}

pub trait WebSocketConn {
    fn on_message(&mut self);
}

pub trait WebSocketHandler {}
