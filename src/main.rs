#![allow(dead_code)]
#![allow(unused)]

extern crate core;

use std::io::Write;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use middleware::FuncMiddleware;
use tokio::io::{AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};

use crate::config::Config;
use crate::context::Context;
use crate::error::HTTPError;
use crate::fs::FsHandler;
use crate::handler::Handler;
use crate::mux::UnsafeMux;
use crate::response::Response;

mod compress;
mod config;
mod context;
mod error;
mod fs;
mod handler;
mod headers;
mod message;
mod middleware;
mod multi_values_map;
mod mux;
mod request;
mod response;
mod sync;
mod uri;

struct AliveCounter {
    counter: Arc<AtomicI64>,
}

impl AliveCounter {
    fn new(counter: Arc<AtomicI64>) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self { counter }
    }
}

impl Drop for AliveCounter {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

async fn http11(
    stream: TcpStream,
    counter: Arc<AtomicI64>,
    cfg: Config,
    mut handler: Box<dyn Handler>,
) {
    let mut stream = Box::pin(BufStream::new(stream));
    let mut rbuf = String::with_capacity(cfg.read_buf_cap);

    loop {
        tokio::select! {
            from_result = request::from11(stream.as_mut(), &mut rbuf, &cfg) => {
                match from_result {
                    Ok(mut req) => {
                        let _ = AliveCounter::new(counter.clone());

                        let mut resp = Response::default(&mut req);

                        let mut ctx = Context::new(
                            unsafe{std::mem::transmute(req.as_mut())},
                            unsafe{std::mem::transmute(resp.as_mut())}
                        );

                        handler.handle(&mut ctx).await;

                        let _ = resp.to11(stream.as_mut()).await;
                        if let Err(_) = (stream.flush().await) {
                            return ;
                        };
                    }
                    Err(e) => {
                        let code = e.statuscode();
                        if code < 100 {
                            return;
                        }
                        let _ = stream.write(format!("HTTP/1.0 {} Bad Request\r\nContent-Length: 12\r\n\r\nHello World!", e).as_bytes()).await;
                        let _ = stream.flush().await;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                return;
            }
        }
    }
}

fn new_mux() -> UnsafeMux {
    let mut mux = UnsafeMux::new();

    mux.apply(FuncMiddleware::new(
        pre!(ctx, {
            ctx.set("begin", Box::new(std::time::SystemTime::now()));
        }),
        post!(ctx, {
            let begin = *(ctx.get::<std::time::SystemTime>("begin").unwrap());

            let mut req = ctx.request();
            let now = chrono::Local::now();
            println!(
                "[{}] {} {} {}us",
                now.to_rfc3339(),
                req.method().to_string(),
                req.uri().path().clone(),
                std::time::SystemTime::now()
                    .duration_since(begin)
                    .unwrap()
                    .as_micros(),
            );
        }),
    ));

    mux.register(
        "/static/httpd/source/",
        FsHandler::new("./", "/static/httpd/source"),
    );

    mux.register(
        "/",
        func!(ctx, {
            let _ = ctx.response().write("hello world!".repeat(50).as_bytes());
        }),
    );
    mux
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    let listener = TcpListener::bind(&config.addr).await.unwrap();
    let alive_counter: Arc<AtomicI64> = Arc::new(AtomicI64::new(0));
    let mux = new_mux();
    let mux_ptr: usize = unsafe { std::mem::transmute(&mux) };

    println!(
        "[{}] httpd listening @ {}, Pid: {}",
        chrono::Local::now(),
        &config.addr,
        std::process::id()
    );

    loop {
        tokio::select! {
            ar = listener.accept() => {
                match ar {
                    Err(_) => {
                        continue;
                    }
                    Ok((stream, _)) => {
                        let counter = alive_counter.clone();
                        let cfg = config.clone();
                        tokio::spawn(async move {
                            http11(stream, counter, cfg, func!(ctx, {
                                let result = unsafe {
                                    let mux: &mut UnsafeMux = std::mem::transmute(mux_ptr);
                                    mux.handle(&mut *ctx).await
                                };
                                result
                            })).await;
                        });
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("[{}] httpd is preparing to shutdown", chrono::Local::now());
                loop {
                    if alive_counter.load(Ordering::Relaxed) < 1 {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                println!("[{}] httpd is gracefully shutdown", chrono::Local::now());
                return Ok(());
            }
        }
    }
}
