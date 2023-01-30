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
    _sync: Option<RwLock<()>>,
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

    pub fn sync(&mut self) -> &mut RwLock<()> {
        if self._sync.is_none() {
            self._sync = Some(RwLock::new(()));
        }
        self._sync.as_mut().unwrap()
    }

    pub fn ctx(&self) -> Option<&Context> {
        self.msg._ctx.as_ref()
    }

    pub fn ctx_mut(&mut self) -> &mut Context {
        if self.msg._ctx.is_none() {
            self.msg._ctx = Some(Context::new());
        }
        self.msg._ctx.as_mut().unwrap()
    }
}

pub async fn from11<Reader: AsyncBufReadExt + Unpin + Send>(
    reader: Reader,
    buf: &mut String,
    cfg: &Config,
) -> Result<Box<Request>, StatusCodeError> {
    return match Message::from11(reader, buf, cfg).await {
        Ok(msg) => Ok(Box::new(Request {
            msg,
            _uri: None,
            _sync: None,
        })),
        Err(e) => Err(e),
    };
}
