use std::{
    pin::Pin,
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt};

use tokio::{io::BufStream, net::TcpStream};

use crate::{
    config::Config,
    http::{
        context::Context, error::HTTPError, handler::Handler, http2, message, request,
        response::Response, websocket,
    },
};

use super::rwstream::RwStream;

pub async fn http11<T: RwStream + 'static>(
    stream: T,
    counter: Arc<AtomicI64>,
    cfg: &'static Config,
    handler: &'static Box<dyn Handler>,
) {
    let bufstream: BufStream<T>;
    if cfg.socket.read_buf_cap > 0 {
        bufstream =
            BufStream::with_capacity(cfg.socket.read_buf_cap, cfg.socket.write_buf_cap, stream)
    } else {
        bufstream = BufStream::new(stream);
    }

    let mut stream = Box::pin(bufstream);
    let mut rbuf = String::with_capacity(cfg.message.read_buf_cap);

    loop {
        tokio::select! {
            from_result = request::from11(stream.as_mut(), &mut rbuf, cfg) => {
                match from_result {
                    Ok(mut req) => {
                        let mut ctx = Context::new(req, Response::default(&mut req, cfg.message.disbale_compression));

                        handler.handle(&mut ctx).await;

                        let _ = ctx._resp.to11(stream.as_mut()).await;
                        if (stream.flush().await).is_err() {
                            return ;
                        }

                        if let Some(proto) = &ctx._upgrade_to {
                            match proto.as_str() {
                                "websocket" => {
                                    tokio::spawn(async move{
                                        websocket::conn(stream).await;
                                    });
                                    return;
                                }
                                ("h2c"| "http2"| "h2") => {
                                    tokio::spawn(async move{
                                        http2::conn(stream).await;
                                    });
                                    return;
                                }
                                _ => {
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let code = e.statuscode();
                        if code == message::READ_MSG_ERROR_MAYBE_HTTP2 {
                            rbuf.clear();
                            match stream.read_line(&mut rbuf).await {
                                Ok(line_size) => {
                                    if line_size != 2 {
                                        return;
                                    }
                                },
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
                                },
                                Err(_) => {
                                    return;
                                }
                            }
                            rbuf.clear();
                            match stream.read_line(&mut rbuf).await {
                                Ok(line_size) => {
                                    if line_size != 2 {
                                        return ;
                                    }
                                },
                                Err(_) => {
                                    return;
                                }
                            }
                            tokio::spawn(async move{
                                http2::conn(stream).await;
                            });
                            return;
                        }
                        if code < 100 {
                            return;
                        }
                        let _ = stream.write(format!("HTTP/1.0 {} Bad Request\r\nConnection: close\r\n\r\n", code).as_bytes()).await;
                        let _ = stream.flush().await;
                        return;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(cfg.http11.conn_idle_timeout)) => {
                return;
            }
        }
    }
}
