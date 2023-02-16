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
    let mut rbuf = String::with_capacity(cfg.message.read_buf_cap.usize());

    loop {
        tokio::select! {
            reqr = Request::from11(&mut stream, &mut rbuf, cfg) => {
                match reqr {
                    Ok(req) => {
                        let mut ctx = Context::new(req);
                        handler.handler(&mut ctx).await;
                        let _ = ctx.resp.to11(&mut stream).await;
                        if let Err(_) = stream.flush().await {
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
                            Protocol::Nil => {}
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
