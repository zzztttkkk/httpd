use tokio::io::AsyncBufReadExt;

use crate::config::Config;

use super::message::Message;

pub struct Request {
    msg: Box<Message>,
}

impl Request {
    pub async fn from11<'a, R: AsyncBufReadExt + Unpin>(
        reader: &'a mut R,
        buf: &'a mut String,
        cfg: &'static Config,
    ) -> Result<Request, u32> {
        return match Message::from11(reader, buf, cfg).await {
            Ok(msg) => {
                let mut req = Request { msg };
                match req.init(cfg) {
                    Ok(_) => Ok(req),
                    Err(ev) => Err(ev),
                }
            }
            Err(ev) => Err(ev),
        };
    }

    fn init(&mut self, cfg: &'static Config) -> Result<(), u32> {
        Ok(())
    }
}
