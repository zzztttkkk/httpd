use utils::anyhow;

use crate::{config::service::ServiceConfig, message::Message};

use super::common::Service;

pub struct FsService {
    cfg: &'static ServiceConfig,
}

impl FsService {
    pub fn new(cfg: &'static ServiceConfig) -> Self {
        Self { cfg }
    }
}

impl Service for FsService {
    fn config(&self) -> &'static ServiceConfig {
        self.cfg
    }

    async fn init(&mut self) -> utils::anyhow::Result<()> {
        Ok(())
    }

    fn http<
        R: tokio::io::AsyncBufReadExt + Unpin + Send,
        W: tokio::io::AsyncWriteExt + Unpin + Send,
    >(
        &self,
        ctx: &crate::ctx::ConnContext<R, W>,
        req: &mut Message,
        resp: &mut Message,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        async { Ok(()) }
    }
}
