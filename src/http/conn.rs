use std::{
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};
use tokio::io::AsyncWriteExt;

use tokio::{io::BufStream, net::TcpStream};

use crate::{config::Config, http::handler::Handler};

use super::http11;

#[inline(always)]
pub async fn conn(
    stream: TcpStream,
    counter: Arc<AtomicI64>,
    cfg: &Config,
    handler: &Box<dyn Handler>,
) {
    http11::http11(stream, counter, cfg, handler).await;
}
