use tokio::io::AsyncBufReadExt;

use crate::config::Config;

use super::message::Message;

pub struct Request {
    pub(crate) msg: Box<Message>,
}

macro_rules! make_method_assert {
    ($name:ident, $value:expr) => {
        #[inline]
        pub fn $name(&self) -> bool {
            self.method() == $value
        }
    };
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

    #[inline]
    pub fn method(&self) -> &str {
        &self.msg.f0
    }

    make_method_assert!(method_is_connect, "CONNECT");
    make_method_assert!(method_is_delete, "DELETE");
    make_method_assert!(method_is_get, "GET");
    make_method_assert!(method_is_head, "HEAD");
    make_method_assert!(method_is_options, "OPTIONS");
    make_method_assert!(method_is_patch, "PATHC");
    make_method_assert!(method_is_post, "POST");
    make_method_assert!(method_is_put, "PUT");
    make_method_assert!(method_is_trace, "TRACE");
}
