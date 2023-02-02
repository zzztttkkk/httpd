use std::pin::Pin;

use tokio::{
    io::{AsyncRead, AsyncWrite, BufStream},
    net::TcpStream,
};

use super::{context::Context, rwstream::RwStream};

pub fn handshake(ctx: &mut Context) -> bool {
    false
}

pub async fn conn<T: RwStream>(bufstream: Pin<Box<BufStream<T>>>) {
	tokio::io::split(bufstream);
    todo!("http2")
}
