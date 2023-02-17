#![allow(unused)]

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use tokio::net::{TcpListener, TcpStream};

use crate::config::{Args, Config};
use crate::http::Handler;
use crate::utils::AutoCounter;

mod config;
mod http;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config;
    if !args.file.trim().is_empty() {
        let txt = std::fs::read_to_string(args.file.trim())?;
        config = toml::from_str(txt.as_str())?;
    } else {
        config = Config::default();
    }
    config.autofix();
    if (!args.addr.trim().is_empty()) {
        config.addr = args.addr.trim().to_string();
    }

    let config: &'static Config = unsafe { std::mem::transmute(&config) };

    http::serve(&config).await
}
