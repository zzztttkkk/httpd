use tokio::io::AsyncBufReadExt;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::http::context::Context;
use crate::http::error::StatusCodeError;
use crate::http::headers::Headers;
use crate::http::message::{BodyBuf, Message};
use crate::utils::ReadonlyUri;

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

    pub fn upgrade_to(&mut self) -> Option<String> {
        if self.method() != "GET" {
            return None;
        }
        if let Some(conn) = self.headers().get("connection") {
            if (conn.to_lowercase() == "upgrade") {
                if let Some(proto_info) = self.headers().get("upgrade") {
                    return Some(proto_info.to_ascii_lowercase());
                }
            }
        }
        None
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
