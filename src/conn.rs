use std::net::SocketAddr;
use crate::config::Config;

pub async fn on_conn<R: tokio::io::AsyncRead, W: tokio::io::AsyncWrite>(r: R, w: W, addr: SocketAddr, config: &'static Config) {
    println!("CONNECTION MADE: {} {:p}", addr, config);
}
