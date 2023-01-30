use tokio::io::AsyncBufReadExt;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::context::Context;
use crate::error::StatusCodeError;
use crate::headers::Headers;
use crate::message::{BodyBuf, Message};
use crate::uri::ReadonlyUri;

pub struct Request {
    msg: Box<Message>,
    _uri: Option<ReadonlyUri>,
}

impl Request {
    #[inline(always)]
    pub fn method(&self) -> &str {
        self.msg.f0.as_str()
    }

    #[inline(always)]
    pub fn rawpath(&self) -> &str {
        self.msg.f1.as_str()
    }

    #[inline(always)]
    pub fn protoversion(&self) -> &str {
        self.msg.f2.as_str()
    }

    #[inline(always)]
    pub fn uri(&mut self) -> &mut ReadonlyUri {
        if self._uri.is_none() {
            self._uri = Some(ReadonlyUri::new(self.rawpath()));
        }
        return self._uri.as_mut().unwrap();
    }

    #[inline(always)]
    pub fn headers(&mut self) -> &mut Headers {
        &mut self.msg.headers
    }

    #[inline(always)]
    pub fn body(&mut self) -> Option<&mut BodyBuf> {
        self.msg.bodybuf.as_mut()
    }
}

pub async fn from11<Reader: AsyncBufReadExt + Unpin + Send>(
    reader: Reader,
    buf: &mut String,
    cfg: &Config,
) -> Result<Box<Request>, StatusCodeError> {
    return match Message::from11(reader, buf, cfg).await {
        Ok(msg) => Ok(Box::new(Request { msg, _uri: None })),
        Err(e) => Err(e),
    };
}
