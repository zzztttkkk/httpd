use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::http::{http2, ws};
use crate::http::ctx::{Context, Protocol};
use crate::http::handler::Handler;
use crate::http::request::Request;
use crate::http::rwtypes::AsyncStream;
use crate::utils;

pub async fn conn<T: AsyncStream + 'static>(stream: T, ac: Arc<AtomicI64>, cfg: &'static Config, handler: Arc<dyn Handler>) {
    let __ac = utils::AutoCounter::new(ac.clone());

    let rawstream = Arc::new(Mutex::new(
        BufStream::with_capacity(cfg.socket.read_buf_cap, cfg.socket.write_buf_cap, stream)
    ));

    loop {
        let input = rawstream.clone();
        let output = rawstream.clone();
        match Request::from11(input).await {
            Ok(req) => {
                let mut ctx = Context::new(req);
                handler.handler(&mut ctx).await;
                ctx.resp.to11(output).await;

                match ctx.upgrade_protocol {
                    Protocol::Websocket => {
                        tokio::spawn(ws::conn(rawstream, ac, cfg, handler));
                        return;
                    }
                    Protocol::Http2 => {
                        tokio::spawn(http2::conn(rawstream, ac, cfg, handler));
                        return;
                    }
                    Protocol::Nil => {}
                }
            }
            Err(_) => {
                let stream = rawstream.clone();
                let mut stream = stream.lock().await;
                stream.write("Hello World!".as_bytes()).await;
            }
        }
    }
}
