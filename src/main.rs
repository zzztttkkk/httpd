#![allow(dead_code)]
#![allow(unused)]

extern crate core;

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use clap::Parser;
use tokio::io::{AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};

use crate::config::Config;
use crate::error::HTTPError;
use crate::fs::FsHandler;
use crate::handler::Handler;
use crate::response::Response;

mod error;
mod router;
mod request;
mod headers;
mod response;
mod handler;
mod compress;
mod message;
mod config;
mod fs;
mod uri;
mod multi_value_map;

struct AliveCounter {
    counter: Arc<AtomicI64>,
}

impl AliveCounter {
    fn new(counter: Arc<AtomicI64>) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self {
            counter
        }
    }
}

impl Drop for AliveCounter {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}


async fn http11(stream: TcpStream, counter: Arc<AtomicI64>, cfg: Config) {
    let mut stream = Box::pin(BufStream::new(stream));
    let mut buf = String::with_capacity(cfg.read_buf_cap);
    let fsh = FsHandler::new("./", "/");

    loop {
        tokio::select! {
            r = request::from11(stream.as_mut(), &mut buf, &cfg) => {
                match r {
                    Ok(mut req) => {
                        let _ = AliveCounter::new(counter.clone());

                        let mut resp = Response::default(&mut req);

                        match fsh.handle(&mut req, &mut resp).await {
                            Ok(_) => {
                                resp.to(stream.as_mut());
                            }
                            Err(v) => {
                                println!("[{}] Request: {} {}", chrono::Local::now(), req.method(), req.rawpath());
                                let _ = stream.write(format!("HTTP/1.0 {} OK\r\nContent-Length: 12\r\n\r\nHello World!", v.statuscode()).as_bytes()).await;
                            }
                        };

                        let _ = stream.flush().await;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    let listener = TcpListener::bind(&config.addr).await.unwrap();
    let alive_counter: Arc<AtomicI64> = Arc::new(AtomicI64::new(0));

    println!("[{}] httpd listening @ {}, Pid: {}", chrono::Local::now(), &config.addr, std::process::id());

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
                            http11(stream, counter, cfg).await;
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
