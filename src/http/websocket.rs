use std::pin::Pin;

use tokio::{io::BufStream, net::TcpStream};

use super::context::Context;

pub fn handshake(ctx: &mut Context) -> bool {
    false
}

pub async fn conn(bufstream: Pin<Box<BufStream<TcpStream>>>) {}
