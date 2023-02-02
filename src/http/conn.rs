use std::{
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use tokio::{io::BufStream, net::TcpStream};

use crate::{config::Config, http::handler::Handler};

use super::{http11, rwstream::RwStream};

#[inline(always)]
pub async fn conn<T: RwStream + 'static>(
    stream: T,
    counter: Arc<AtomicI64>,
    cfg: &'static Config,
    handler: &'static Box<dyn Handler>,
) {
    http11::http11(stream, counter, cfg, handler).await;
}
