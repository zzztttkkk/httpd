use std::sync::atomic::AtomicI64;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};

use crate::config::Config;
use crate::http::ctx::{Context, Protocol};
use crate::http::handler::Handler;
use crate::http::request::Request;
use crate::http::rwtypes::AsyncStream;
use crate::http::{http2, ws};
use crate::utils;

use super::message::ERR_MAYBE_HTTP2;

fn can_keep_alive(v: &str) -> bool {
    let major = v.as_bytes()[5];
    if (major == '0' as u8) {
        return false;
    }
    return v.ends_with(".0");
}

// https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#2-if-t-static-then-t-must-be-valid-for-the-entire-program
pub async fn conn<T: AsyncStream + 'static>(
    stream: T,
    ac: &'static AtomicI64,
    cfg: &'static Config,
    handler: &'static Box<dyn Handler>,
) {
    let __ac = utils::AutoCounter::new(ac);

    let mut stream = BufStream::with_capacity(
        cfg.socket.read_buf_cap.usize(),
        cfg.socket.write_buf_cap.usize(),
        stream,
    );
    let mut rbuf = String::with_capacity(cfg.http.read_buf_cap.usize());

    loop {
        tokio::select! {
            parse_result = Request::from11(&mut stream, &mut rbuf, cfg) => {
                match parse_result {
                    Ok(req) => {
                        let mut ctx = Context::new(req);
                        handler.handler(&mut ctx).await;
                        let _ = ctx.resp.to11(&mut stream).await;
                        if (stream.flush().await).is_err() {
                            break;
                        }
                        match ctx.upgrade_protocol {
                            Protocol::Websocket(wsh) => {
                                tokio::spawn(ws::conn(stream, ac, cfg, wsh));
                                return;
                            }
                            Protocol::Http2 => {
                                tokio::spawn(http2::conn(stream, ac, cfg, handler));
                                return;
                            }
                            Protocol::Nil => {
                                if(!can_keep_alive(ctx.req.msg.f2.as_str())){
                                    return;
                                }

                                if let Some(cv) = ctx.resp.msg.headers.get("connection") {
                                    if(cv == "close") {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Err(ev) => {
                        if ev == ERR_MAYBE_HTTP2 {
                            rbuf.clear();
                            match stream.read_line(&mut rbuf).await {
                                Ok(line_size) => {
                                    if line_size != 2 {
                                        return;
                                    }
                                }
                                Err(_) => {
                                    return;
                                }
                            }
                            rbuf.clear();
                            match stream.read_line(&mut rbuf).await {
                                Ok(line_size) => {
                                    if line_size != 4 || &rbuf[0..2] != "SM" {
                                        return;
                                    }
                                }
                                Err(_) => {
                                    return;
                                }
                            }
                            rbuf.clear();
                            match stream.read_line(&mut rbuf).await {
                                Ok(line_size) => {
                                    if line_size != 2 {
                                        return;
                                    }
                                }
                                Err(_) => {
                                    return;
                                }
                            }
                            tokio::spawn(http2::conn(stream, ac, cfg, handler));
                        }
                        break;
                    }
                }
            }
            _  = tokio::time::sleep(cfg.http11.conn_idle_timeout.duration()) => {
                break;
            }
        }
    }
}
