use std::net::SocketAddr;

use crate::config::service::ServiceConfig;

pub struct ForwardService(&'static ServiceConfig);

impl ForwardService {
    pub fn new(cfg: &'static ServiceConfig) -> Self {
        Self(cfg)
    }

    pub async fn serve<R: tokio::io::AsyncRead + Unpin, W: tokio::io::AsyncWrite + Unpin>(
        &self,
        r: R,
        w: W,
        addr: SocketAddr,
        config: &'static ServiceConfig,
    ) {
    }
}
