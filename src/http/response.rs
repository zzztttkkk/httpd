use std::io::Write;

use tokio::io::AsyncWriteExt;

use crate::http::message::Message;

use super::status::{status_reason, STATUS_CODES};

pub struct Response {
    pub(crate) msg: Message,
    pub(crate) _status_code: u32,
}

impl Response {
    pub fn new() -> Self {
        Self {
            msg: Message::new(),
            _status_code: 0,
        }
    }

    fn tomsg(&mut self) {
        let _ = self.msg.flush();

        if self._status_code == 0 {
            self._status_code = 200;
        }

        self.msg.f1 = self._status_code.to_string();
        self.msg.f2 = status_reason(self._status_code).to_string();
    }

    pub(crate) async fn to11<Writer: AsyncWriteExt + Unpin>(
        &mut self,
        writer: &mut Writer,
    ) -> std::io::Result<()> {
        self.tomsg();
        self.msg.to11(writer).await
    }
}

impl Write for Response {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.msg.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Response {
    pub fn status(&mut self, code: u32) -> &mut Self {
        self._status_code = code;
        self
    }

    pub fn text(&mut self, txt: &str) -> &mut Self {
        self.msg.headers.set_content_type("text/plain");
        self.msg.write(txt.as_bytes());
        self
    }

    pub fn html(&mut self, html: &str) -> &mut Self {
        self.msg.headers.set_content_type("text/html");
        self.msg.write(html.as_bytes());
        self
    }
}
