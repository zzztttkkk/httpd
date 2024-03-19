use std::net::SocketAddr;

use crate::config::service::ServiceConfig;

pub struct UpstreamService(&'static ServiceConfig);

impl UpstreamService {
    pub fn new(cfg: &'static ServiceConfig) -> Self {
        Self(cfg)
    }

    pub async fn serve<R: tokio::io::AsyncRead + Unpin, W: tokio::io::AsyncWrite + Unpin>(
        &self,
        _r: R,
        _w: W,
        _addr: SocketAddr,
    ) {
    }
}
