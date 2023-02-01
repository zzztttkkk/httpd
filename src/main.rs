#![allow(unused)]

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use tokio::net::TcpListener;

use crate::config::{Args, Config};
use crate::http::conn;

mod config;
mod http;
mod utils;

fn init_log(fp: String) {
    let cfg = log4rs::config::RawConfig::default();
    if !fp.is_empty() {}

    log4rs::init_raw_config(Default::default());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config;
    if let Some(cf) = args.file {
        let txt = std::fs::read_to_string(cf.as_str())?;
        config = toml::from_str(txt.as_str())?;
    } else {
        config = Config::default();
    }
    config.autofix();

    init_log(config.log_config_file.clone());

    let mut addr: String = args.addr.clone();
    if !config.addr.is_empty() {
        addr = config.addr.clone();
    }
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!(
        "[{}] httpd listening @ {}, Pid: {}",
        utils::Time::currentstr(),
        &addr,
        std::process::id()
    );

    let alive_counter: Arc<AtomicI64> = Arc::new(AtomicI64::new(0));
    let handler = func!({});
    let handler_ptr: usize = unsafe { std::mem::transmute(&handler) };
    let cfg_ptr: usize = unsafe { std::mem::transmute(&config) };

    loop {
        tokio::select! {
            ar = listener.accept() => {
                match ar {
                    Err(_) => {
                        continue;
                    }
                    Ok((stream, _)) => {
                        let counter = alive_counter.clone();
                        tokio::spawn(async move {
                            conn(stream, counter, unsafe{ std::mem::transmute(cfg_ptr) }, unsafe{ std::mem::transmute(handler_ptr) }).await;
                        });
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("[{}] httpd is preparing to shutdown", utils::Time::currentstr());
                loop {
                    if alive_counter.load(Ordering::Relaxed) < 1 {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                println!("[{}] httpd is gracefully shutdown", utils::Time::currentstr());
                return Ok(());
            }
        }
    }
}
