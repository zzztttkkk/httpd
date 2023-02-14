use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use tokio::io::{AsyncWriteExt, BufStream};

use crate::config::Config;
use crate::http::ctx::{Context, Protocol};
use crate::http::handler::Handler;
use crate::http::request::Request;
use crate::http::rwtypes::AsyncStream;
use crate::http::{http2, ws};
use crate::utils;

pub async fn conn<T: AsyncStream + 'static>(
    stream: T,
    ac: &'static AtomicI64,
    cfg: &'static Config,
    handler: &'static Box<dyn Handler>,
) {
    let __ac = utils::AutoCounter::new(ac);

    let mut stream =
        BufStream::with_capacity(cfg.socket.read_buf_cap, cfg.socket.write_buf_cap, stream);
    let mut buf = String::with_capacity(cfg.message.read_buf_cap);

    loop {
        match Request::from11(&mut stream, &mut buf, cfg).await {
            Ok(req) => {
                let mut ctx = Context::new(req);
                handler.handler(&mut ctx).await;
                ctx.resp.to11(&mut stream).await;

                match ctx.upgrade_protocol {
                    Protocol::Websocket(wsh) => {
                        tokio::spawn(ws::conn(stream, ac, cfg, wsh));
                        return;
                    }
                    Protocol::Http2 => {
                        tokio::spawn(http2::conn(stream, ac, cfg, handler));
                        return;
                    }
                    Protocol::Nil => {}
                }
            }
            Err(_) => {
                stream.write("Hello World!".as_bytes()).await;
            }
        }
    }
}
