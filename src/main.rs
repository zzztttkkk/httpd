#![allow(dead_code)]
#![allow(unused)]

use std::sync::Arc;
use clap::Parser;
use crate::config::Config;
use crate::conn::on_conn;

mod config;
mod conn;

#[derive(clap::Parser, Debug)]
#[command(name = "httpd")]
#[command(about = "A simple http server", long_about = None)]
pub struct Args {
    #[arg(name = "config", long, short, default_value = "")]
    /// config file path(toml)
    pub file: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut config: Config;

    if !args.file.trim().is_empty() {
        let txt = std::fs::read_to_string(args.file.trim()).unwrap();
        config = toml::from_str(txt.as_str()).unwrap();
    } else {
        config = Config::default();
    }
    config.autofix();

    let config: &'static Config = unsafe { std::mem::transmute(&config) };

    let listener = tokio::net::TcpListener::bind(config.server.addr.clone()).await.unwrap();
    let tlscfg = config.server.tls.load();

    let mut logo = format!("httpd listening @ {}, pid {}", config.server.addr, std::process::id());
    if tlscfg.is_some() {
        logo = format!("{}, tls ✅", logo);
    }
    println!("{}", logo);

    match tlscfg {
        None => {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        if result.is_err() {
                            continue;
                        }

                        let (mut stream, addr) = result.unwrap();
                        tokio::spawn(async move {
                            let (r,w ) = stream.split();
                            on_conn(r, w, addr, config).await;
                        });
                    },
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    }
                }
            }
        }
        Some(tlscfg) => {
            let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tlscfg));

            loop {
                tokio::select! {
                    result = listener.accept() => {
                        if result.is_err() {
                            continue;
                        }

                        let (stream, addr) = result.unwrap();
                        let acceptor = acceptor.clone();
                        tokio::spawn(async move {
                            match acceptor.accept(stream).await {
                                Ok(stream) => {
                                    let (r, w) = tokio::io::split(stream);
                                    on_conn(r, w, addr, config).await;
                                }
                                Err(err) => {
                                    println!("httpd: tls handshake failed, {}", err);
                                }
                            }
                        });
                    },
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    }
                }
            }
        }
    }

    println!("httpd gracefully shutdown");
}
