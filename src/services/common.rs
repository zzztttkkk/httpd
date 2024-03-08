use std::future::Future;

use utils::anyhow;

pub trait Service {
    fn init(&mut self) -> anyhow::Result<()>;

    fn serve<R: tokio::io::AsyncRead + Unpin + Send, W: tokio::io::AsyncWrite + Unpin + Send>(
        &self,
        r: R,
        w: W,
        addr: std::net::SocketAddr,
    ) -> impl Future<Output = ()> + Send;
}
