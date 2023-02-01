use std::pin::Pin;

use tokio::{io::BufStream, net::TcpStream};

use super::context::Context;

pub async fn websocket_handshake(ctx: &mut Context) -> bool {
    false
}

pub async fn websocket_conn(bufstream: Pin<Box<BufStream<TcpStream>>>) {}
