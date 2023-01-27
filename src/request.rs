use tokio::io::AsyncBufReadExt;

use crate::config::Config;
use crate::error::StatusCodeError;
use crate::message::Message;
use crate::uri::ReadonlyUri;

pub struct Request {
    msg: Message,
    uri: Option<ReadonlyUri>,
}


impl Request {
    pub fn method(&self) -> &str { self.msg.f0.as_str() }

    pub fn rawpath(&self) -> &str { self.msg.f1.as_str() }

    pub fn protoversion(&self) -> &str { self.msg.f2.as_str() }

    pub fn url(&mut self) {
        if self.uri.is_none() {
            self.uri = Some(ReadonlyUri::new(self.rawpath()));
        }
    }
}


pub async fn from11<Reader: AsyncBufReadExt + Unpin + Send>(reader: Reader, buf: &mut String, cfg: &Config) -> Result<Request, StatusCodeError> {
    return match Message::from11(reader, buf, cfg).await {
        Ok(msg) => {
            Ok(Request { msg, uri: None })
        }
        Err(e) => {
            Err(e)
        }
    }
}
