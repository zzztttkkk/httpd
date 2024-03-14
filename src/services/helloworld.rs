use std::io::Write;

use utils::anyhow;

use crate::{
    config::service::ServiceConfig,
    ctx::ConnContext,
    message::{Message, MessageReadCode},
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
        async {
            let mut resp = ResponseWriter::from(resp);
            let resp = &mut resp;
            resp.code(200, "OK")
                .version(1, 1)
                .header("server", "httpd.rs");

            Ok(())
        }
    }
}
