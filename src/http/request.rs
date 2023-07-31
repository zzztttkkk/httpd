use tokio::io::AsyncBufReadExt;
use crate::http::message::Message;

pub struct Request {
    msg: Message,
}

impl Request {
    pub fn new() -> Self {
        return Self { msg: Message::new() };
    }

    pub(crate) async fn readfrom11<T: AsyncBufReadExt + Unpin>(
        &mut self, stream: &mut T,
        buf: &mut Vec<u8>,
    ) -> u32 {
        return self.msg.readfrom11(stream, buf, true).await;
    }

    pub fn method(&self) -> &str {
        return self.msg.fl.0.as_str();
    }
}
