use tokio::io::AsyncWriteExt;
use crate::http::message::Message;

pub struct Response {
    msg: Message,
}

impl Response {
    pub fn new() -> Self {
        return Self { msg: Message::new() };
    }

    pub async fn writeto11<T: AsyncWriteExt + Unpin>(
        &mut self,
        stream: &mut T,
        buf: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        return self.msg.writeto11(stream, buf, false).await;
    }
}
