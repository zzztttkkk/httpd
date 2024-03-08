use crate::config::service::ServiceConfig;

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
    fn serve<R: tokio::io::AsyncRead + Unpin + Send, W: tokio::io::AsyncWrite + Unpin + Send>(
        &self,
        r: R,
        w: W,
        addr: std::net::SocketAddr,
    ) -> impl std::future::Future<Output = ()> + Send {
        async move {}
    }

    fn init(&mut self) -> crate::uitls::anyhow::Result<()> {
        todo!()
    }
}
