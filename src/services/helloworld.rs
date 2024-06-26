use crate::utils::anyhow;

use crate::{
    config::service::ServiceConfig, ctx::ConnContext, message::Message, protocols::Protocol,
    reqr::RequestReader, respw::ResponseWriter,
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

    async fn init(&mut self) -> crate::utils::anyhow::Result<()> {
        Ok(())
    }

    fn http<
        R: tokio::io::AsyncBufReadExt + Unpin + Send,
        W: tokio::io::AsyncWriteExt + Unpin + Send,
    >(
        &self,
        ctx: &ConnContext<R, W>,
        req: &mut Message,
        resp: &mut Message,
    ) -> impl std::future::Future<Output = anyhow::Result<Protocol>> + Send {
        let reqversion;

        {
            log::trace!(service = self.config().idx(); "Request From {}", ctx.addr);
            let req = RequestReader::from(&*req);
            log::trace!("{} {} {:?}", req.method(), req.rawuri(), req.version());
            req.headers().each(&mut |k, vs| {
                for v in vs {
                    log::trace!(service = self.config().idx(); "{}: {}", k, v);
                }
                true
            });

            reqversion = req.version();

            if req.body().is_empty() {
                log::trace!("EmptyBody");
            } else {
                match std::str::from_utf8(req.body().inner()) {
                    Ok(txt) => {
                        log::trace!(service = self.config().idx(); "PrintableBody:\r\n---------------------\r\n{}\r\n---------------------", txt);
                    }
                    Err(_) => {
                        log::trace!(service = self.config().idx(); "BinaryBody: {}", req.body().size());
                    }
                }
            }
        }

        async move {
            let mut keep_alive = true;

            let resp = {
                let mut w = ResponseWriter::from(resp);
                w.version(1, 1).code(200, "OK").header("server", "httpd.rs");

                match reqversion {
                    Ok((major, minor)) => {
                        w.version(major, minor);
                        keep_alive = major >= 1 && minor >= 1;
                    }
                    Err(_) => {}
                }

                w.end()
            };
            resp.body.write_all_to_internal("Hello world!".as_bytes());

            Ok(Protocol::Current { keep_alive })
        }
    }
}
