// #![allow(unused)]

use std::{sync::Arc, thread};

use clap::Parser;
use http::handler::FuncHandler;

use crate::config::{Args, Config};

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

    let handler = Arc::new(FuncHandler::new(Box::new(|ctx| {
        let ctx = ctx.clone();
        return Box::pin(async move {
            let ctx = ctx.lock().await;

            println!("{:?}", ctx.req.msg.header.get_content_length());

            println!(
                "----ctx: {:p} thread: {:?}",
                &ctx.req,
                thread::current().id()
            );
            return ();
        });
    })));

    let mut server = http::server::Server::new(config.clone());

    server.listen().await;
    server.serve(handler).await;
    return Ok(());
}
