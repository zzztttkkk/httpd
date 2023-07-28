use std::net::SocketAddr;
use std::sync::Arc;

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};

use super::{handler::Handler, message::Context};

pub struct Connection<InStream: AsyncRead + Unpin, OutStream: AsyncWrite + Unpin> {
    remote_addr: SocketAddr,
    istream: InStream,
    ostream: OutStream,
}

impl<InStream: AsyncRead + Unpin, OutStream: AsyncWrite + Unpin> Connection<InStream, OutStream> {
    pub(crate) fn new(addr: SocketAddr, ins: InStream, outs: OutStream) -> Self {
        return Self {
            remote_addr: addr,
            istream: ins,
            ostream: outs,
        };
    }

    pub(crate) async fn handle<T: Handler>(&mut self, handler: Arc<T>) {
        let ctx = Arc::new(Mutex::new(Context::new(self.remote_addr)));

        let mut reader = tokio::io::BufReader::new(&mut self.istream);
        let mut writer = tokio::io::BufWriter::new(&mut self.ostream);
        let mut buf = Vec::<u8>::with_capacity(1024);

        loop {
            let mut _ctx = (ctx.lock()).await;
            buf.clear();
            let read_status = _ctx.req.msg.readfrom11(&mut reader, &mut buf).await;
            drop(_ctx);
            if read_status > 0 {
                break;
            }

            handler.handle(ctx.clone()).await;

            let mut _ctx = (ctx.lock()).await;
            buf.clear();
            if let Err(_) = _ctx.resp.msg.writeto11(&mut writer, &mut buf).await {
                break;
            }

            if !_ctx.keep_alive() {
                break;
            }
        }
    }
}
