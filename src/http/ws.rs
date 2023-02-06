use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use tokio::io::AsyncBufReadExt;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::http::Handler;
use crate::http::rwtypes::AsyncStream;

pub async fn conn<T: AsyncBufReadExt>(stream: Arc<Mutex<T>>, ac: Arc<AtomicI64>, cfg: &'static Config, handler: Arc<dyn Handler>) {}
