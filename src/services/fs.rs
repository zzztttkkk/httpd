use std::net::SocketAddr;

use crate::config::service::ServiceConfig;

pub struct FsService {
    cfg: &'static ServiceConfig,
}

impl FsService {
    pub fn new(cfg: &'static ServiceConfig) -> Self {
        Self { cfg }
    }

    pub async fn serve<R: tokio::io::AsyncRead + Unpin, W: tokio::io::AsyncWrite + Unpin>(
        &self,
        r: R,
        w: W,
        addr: SocketAddr,
    ) {
    }
}
