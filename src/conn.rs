use tracing::trace;

use crate::{config::Config, ctx::ConnContext, http11, protocols::Protocol, ws};
use std::net::SocketAddr;

pub async fn on_conn<R: tokio::io::AsyncRead + Unpin, W: tokio::io::AsyncWrite + Unpin>(
    r: R,
    w: W,
    addr: SocketAddr,
    config: &'static Config,
) {
    #[cfg(debug_assertions)]
    {
        trace!("connection made: {}", addr);
    }

    let r = tokio::io::BufReader::with_capacity(config.tcp.read_buf_size.0, r);
    let w = tokio::io::BufWriter::with_capacity(config.tcp.read_buf_size.0, w);
    let mut ctx = ConnContext::new(r, w, addr, config);

    match http11::serve(&mut ctx).await {
        Protocol::None => {}
        Protocol::WebSocket => {
            ws::serve(&mut ctx).await;
        }
        Protocol::Http2 => todo!("http2"),
    }
}
