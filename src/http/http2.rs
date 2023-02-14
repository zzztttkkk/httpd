use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite, BufStream};

use crate::config::Config;
use crate::http::Handler;

pub async fn conn<RW: AsyncWrite + AsyncRead>(
    stream: BufStream<RW>,
    ac: &'static AtomicI64,
    cfg: &'static Config,
    handler: &'static Box<dyn Handler>,
) {
}
