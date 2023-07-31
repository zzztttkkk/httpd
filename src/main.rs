// #![allow(unused)]

use clap::Parser;

use crate::config::{Args, Config};
use crate::http::context::{ContextPtr};
use crate::http::server::Server;

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
    if !args.addr.trim().is_empty() {
        config.addr = args.addr.trim().to_string();
    }

    let handler = |mut ctx: ContextPtr| async move {
        println!("{} {}", ctx.request.method(), ctx.remote_addr());
        let _ = ctx.sync().await;
        ctx.request.method();
    };
    let mut server = Server::new(config.clone());
    server.listen().await;
    server.serve(handler).await;
    return Ok(());
}
