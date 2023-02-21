pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Pong,
    Close,
    Continuation,
}
