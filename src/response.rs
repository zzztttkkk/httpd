use std::io::Write;

use bytebuffer::ByteBuffer;
use flate2::Compression;
use tokio::io::AsyncWriteExt;

use crate::compress::CompressType;
use crate::headers::Headers;
use crate::message::{BodyBuf, Message};
use crate::request::Request;

pub struct Response {
    pub(crate) msg: Message,

    pub(crate) _status_code: u32,
}

impl Response {
    pub fn new() -> Self {
        Self { msg: Message::new(), _status_code: 0 }
    }

    pub fn default(req: &mut Request) -> Self {
        let mut resp = Self::new();
        resp.msg._compress_type = req.headers().compress_type("accept-encoding");
        resp
    }

    #[inline(always)]
    pub fn statuscode(&mut self, code: u32) -> &mut Self {
        self._status_code = code;
        self
    }

    #[inline(always)]
    pub fn headers(&mut self) -> &mut Headers { &mut self.msg.headers }

    fn tomsg(&mut self) {
        self.msg.flush();
        // todo
    }

    pub async fn to11<Writer: AsyncWriteExt + Unpin + Send>(&mut self, mut writer: Writer) {
        self.tomsg();
        self.msg.to11(writer).await;
    }
}

impl Write for Response {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.msg.write(buf) }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::compress::CompressType;
    use crate::response::Response;

    #[test]
    fn resp_wf() {
        let mut resp = Response::new();
        resp.msg._compress_type = Some(CompressType::Gzip);

        let _ = resp.write("hello".repeat(10).as_bytes()).unwrap();
        resp.flush().unwrap();
    }
}
