#![allow(dead_code)]
#![allow(unused)]
#![allow(clippy::needless_return)]

use crate::config::Config;
use crate::conn::on_conn;
use clap::Parser;
use std::ops::Deref;
use std::sync::Arc;

mod config;
mod conn;

#[derive(clap::Parser, Debug)]
#[command(name = "httpd")]
#[command(about = "A simple http server", long_about = None)]
pub struct Args {
    #[arg(name = "config", default_value = "")]
    /// config file path(toml)
    pub file: String,
}

#[cfg(debug_assertions)]
fn parse_args() -> Args {
    Args::parse_from(vec!["httpd", "./httpd.toml"])
}

#[cfg(not(debug_assertions))]
fn parse_args() -> Args {
    Args::parse()
}

#[tokio::main]
async fn main() {
    let args = parse_args();

    let mut config: Config = Default::default();
    if !args.file.trim().is_empty() {
        let txt = std::fs::read_to_string(args.file.trim()).unwrap();
        config = toml::from_str(txt.as_str()).unwrap();
    }
    config.autofix();

    let config: &'static Config = unsafe { std::mem::transmute(&config) };

    let listener = tokio::net::TcpListener::bind(config.server.addr.clone())
        .await
        .unwrap();
    let tlscfg = config.server.tls.load();

    let mut logo = format!(
        "httpd listening @ {}, pid {}",
        config.server.addr,
        std::process::id()
    );
    if tlscfg.is_some() {
        logo = format!("{}, tls ✅", logo);
    }
    println!("{}", logo);

    match tlscfg {
        None => loop {
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
        },
        Some(tlscfg) => {
            let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tlscfg));
            let timeout = config.server.tls.timeout.deref().clone();

            loop {
                tokio::select! {
                    result = listener.accept() => {
                        if result.is_err() {
                            continue;
                        }

                        let (stream, addr) = result.unwrap();
                        let acceptor = acceptor.clone();

                        tokio::spawn(async move {
                            let mut handshake_result = None;

                            if !timeout.is_zero() {
                                tokio::select! {
                                    r = acceptor.accept(stream) => {
                                        handshake_result = Some(r);
                                    }
                                    _ =  tokio::time::sleep(timeout) => {}
                                }
                            } else {
                                handshake_result = Some(acceptor.accept(stream).await);
                            }

                            match handshake_result {
                                Some(handshake_result) => {
                                    match handshake_result {
                                        Ok(stream) => {
                                            let (r, w) = tokio::io::split(stream);
                                            on_conn(r, w, addr, config).await;
                                        },
                                        Err(e) => {
                                        },
                                    }
                                },
                                None => {},
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
