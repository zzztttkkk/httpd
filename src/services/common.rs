use std::future::Future;

use utils::anyhow;

use crate::{config::service::ServiceConfig, ctx::ConnContext, message::Message};

pub trait Service {
    fn config(&self) -> &'static ServiceConfig;

    fn init(&mut self) -> impl Future<Output = anyhow::Result<()>> + Send;

    fn handle<
        R: tokio::io::AsyncBufReadExt + Unpin + Send,
        W: tokio::io::AsyncWriteExt + Unpin + Send,
    >(
        &self,
        ctx: &ConnContext<R, W>,
        req: &mut Message,
        resp: &mut Message,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
