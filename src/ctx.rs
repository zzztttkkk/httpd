use std::net::SocketAddr;

use crate::config::service::ServiceConfig;

pub(crate) struct ConnContext<
    R: tokio::io::AsyncBufReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
> {
    pub(crate) reader: R,
    pub(crate) writer: W,
    pub(crate) buf: Vec<u8>,
    pub(crate) addr: SocketAddr,
    pub(crate) config: &'static ServiceConfig,
    pub(crate) over_tls: bool,
}

impl<R: tokio::io::AsyncBufReadExt + Unpin, W: tokio::io::AsyncWriteExt + Unpin> ConnContext<R, W> {
    pub(crate) fn new(
        r: R,
        w: W,
        addr: SocketAddr,
        over_tls: bool,
        config: &'static ServiceConfig,
    ) -> Self {
        Self {
            reader: r,
            writer: w,
            buf: Vec::with_capacity(config.tcp.buf_size.0),
            addr,
            over_tls,
            config,
        }
    }
}
