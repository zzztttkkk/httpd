use std::net::SocketAddr;
use std::sync::Arc;

use tokio::{
    io::{AsyncRead, AsyncWrite},
};
use crate::http::context::Context;
use crate::http::handler::Handler;


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

    pub(crate) async fn handle(&mut self, handler: Arc<dyn Handler>) {
        let mut ctx = Context::new(self.remote_addr);
        let ptr = ctx.ptr();

        let mut reader = tokio::io::BufReader::new(&mut self.istream);
        let mut writer = tokio::io::BufWriter::new(&mut self.ostream);
        let mut buf = Vec::<u8>::with_capacity(1024);

        loop {
            let read_status = ctx.request.readfrom11(&mut reader, &mut buf).await;
            if read_status > 0 {
                break;
            }

            handler.handle(ptr).await;

            if let Err(_) = ctx.response.writeto11(&mut writer, &mut buf).await {
                break;
            }
        }
    }
}
