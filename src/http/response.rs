use std::sync::Arc;

use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::Mutex;

use crate::http::message::Message;

pub struct Response {
    pub(crate) msg: Message,
}

impl Response {
    pub fn new() -> Self {
        Self {
            msg: Message::new(),
        }
    }

    pub(crate) async fn to11<Writer: AsyncWriteExt + Unpin>(
        &mut self,
        writer: Arc<Mutex<Writer>>,
    ) -> std::io::Result<()> {
        self.msg.to11(writer).await
    }
}
