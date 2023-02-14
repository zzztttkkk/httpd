use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufStream};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::http::rwtypes::AsyncStream;
use crate::http::Handler;
use crate::utils;

use super::ws_handler::WebSocketHandler;

pub async fn conn<RW: AsyncRead + AsyncWrite>(
    stream: BufStream<RW>,
    ac: &'static AtomicI64,
    cfg: &'static Config,
    handler: Arc<dyn WebSocketHandler>,
) {
    let __ac = utils::AutoCounter::new(ac);
}
