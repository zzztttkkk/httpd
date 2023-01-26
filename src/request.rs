use tokio::io::AsyncBufReadExt;

use crate::config::Config;
use crate::error::StatusCodeError;
use crate::message::Message;
use crate::uri::Uri;

pub struct Request<'a> {
    msg: Message,
    uri: Uri<'a>,
}


impl<'a> Request<'a> {
    pub fn method(&self) -> &str { self.msg.f0.as_str() }

    pub fn rawpath(&self) -> &str { self.msg.f1.as_str() }

    pub fn protoversion(&self) -> &str { self.msg.f2.as_str() }

    pub fn url(&mut self) -> &Uri<'a> {
        &self.uri
    }
}


pub async fn from11<'a, Reader: AsyncBufReadExt + Unpin + Send>(reader: Reader, buf: &mut String, cfg: &Config) -> Result<Request<'a>, StatusCodeError> {
    return match Message::from11(reader, buf, cfg).await {
        Ok(msg) => {
            Ok(Request { msg, uri: Uri::new("") })
        }
        Err(e) => {
            Err(e)
        }
    }
}
