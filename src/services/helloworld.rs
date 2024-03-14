use std::io::Write;

use utils::anyhow;

use crate::{
    config::service::ServiceConfig,
    ctx::ConnContext,
    message::{Message, MessageReadCode},
    reqr::RequestReader,
    request::RequestQueryer,
    response::ResponseBuilder,
    respw::ResponseWriter,
};

use super::common::Service;

pub struct HelloWorldService(&'static ServiceConfig);

impl HelloWorldService {
    pub fn new(cfg: &'static ServiceConfig) -> Self {
        Self(cfg)
    }
}

impl Service for HelloWorldService {
    fn config(&self) -> &'static ServiceConfig {
        self.0
    }

    async fn init(&mut self) -> utils::anyhow::Result<()> {
        Ok(())
    }

    fn handle<
        R: tokio::io::AsyncBufReadExt + Unpin + Send,
        W: tokio::io::AsyncWriteExt + Unpin + Send,
    >(
        &self,
        ctx: &ConnContext<R, W>,
        req: &mut Message,
        resp: &mut Message,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        {
            log::trace!("Request From {}", ctx.addr);
            let req = RequestReader::from(&*req);
            log::trace!("{} {} {:?}", req.method(), req.rawuri(), req.version());
            req.headers().each(&mut |k, vs| {
                for v in vs {
                    log::trace!("{}: {}", k, v);
                }
                true
            });

            if req.body().is_empty() {
                log::trace!("RmptyBody");
            } else {
                match std::str::from_utf8(req.body().inner()) {
                    Ok(txt) => {
                        log::trace!("PrintableBody:\r\n---------------------\r\n{}\r\n---------------------", txt);
                    }
                    Err(_) => {
                        log::trace!("BinaryBody: {}", req.body().size());
                    }
                }
            }
        }

        async {
            let resp = {
                let mut w = ResponseWriter::from(resp);
                w.version(1, 1).code(200, "OK").header("server", "httpd.rs");
                w.end()
            };
            resp.body.write_all_to_internal("Hello world!".as_bytes());
            Ok(())
        }
    }
}
