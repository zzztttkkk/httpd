use std::{
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};
use tokio::io::AsyncWriteExt;

use tokio::{io::BufStream, net::TcpStream};

use crate::{config::Config, http::handler::Handler};

pub async fn conn(
    stream: TcpStream,
    counter: Arc<AtomicI64>,
    cfg: &Config,
    handler: &Box<dyn Handler>,
) {
}
