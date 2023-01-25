use tokio::io::AsyncBufReadExt;

use crate::message::{Message, ReadStatus};

pub struct Request {
    msg: Message,
}

static MAX_BODY_SIZE: i64 = 10 * 1024 * 1024;

impl Request {
    fn onstatus(status: ReadStatus, msg: &Message) -> Result<(), i32> {
        match status {
            ReadStatus::Headers => {
                if msg.f0.is_empty() || msg.f1.is_empty() || msg.f2.is_empty() {
                    return Err(1);
                }
            }
            ReadStatus::Body(size) => {
                if size > MAX_BODY_SIZE {
                    return Err(1);
                }
            }
            ReadStatus::Ok => {
                
            }
            _ => {}
        }
        Ok(())
    }
}

impl Request {
    pub fn method(&self) -> &str { self.msg.f0.as_str() }

    pub fn rawpath(&self) -> &str { self.msg.f1.as_str() }

    pub fn protoversion(&self) -> &str { self.msg.f2.as_str() }
}

pub async fn from11<Reader: AsyncBufReadExt + Unpin + Send>(reader: Reader, buf: &mut String) -> Result<Request, i32> {
    return match Message::from11(reader, buf, Request::onstatus).await {
        Ok(msg) => {
            Ok(Request { msg })
        }
        Err(e) => {
            Err(e)
        }
    }
}
