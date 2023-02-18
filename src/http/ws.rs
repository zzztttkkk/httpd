use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite, BufStream};

use crate::config::Config;
use crate::utils;

use super::ctx::Context;
use super::ws_handler::WebSocketHandler;

pub async fn handshake(ctx: &mut Context) {
    if !ctx.request().method_is_get() {
        return;
    }
}

pub async fn conn<RW: AsyncRead + AsyncWrite>(
    stream: BufStream<RW>,
    ac: &'static AtomicI64,
    cfg: &'static Config,
    handler: Arc<dyn WebSocketHandler>,
) {
    let __ac = utils::AutoCounter::new(ac);
}
