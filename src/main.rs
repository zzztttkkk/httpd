use crate::config::Config;
use crate::conn::on_conn;
use clap::Parser;
use std::sync::Arc;
use tracing::{info, trace};

mod config;
mod conn;
mod ctx;
mod http11;
mod message;
mod protocols;
mod request;
pub mod uitls;
mod ws;
mod compression;

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
    let _guards = config.logging.init();

    let listener = tokio::net::TcpListener::bind(config.tcp.addr.clone())
        .await
        .unwrap();
    let tlscfg = config.tcp.tls.load();

    let mut logo = format!(
        "listening @ {}, pid {}",
        config.tcp.addr,
        std::process::id()
    );
    if tlscfg.is_some() {
        logo = format!("{}, tls ✅", logo);
    }
    info!("{}", logo);

    match tlscfg {
        None => loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((mut stream, addr)) => {
                            tokio::spawn(async move {
                                let (r,w ) = stream.split();
                                on_conn(r, w, addr, config).await;
                            });
                        },
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            {
                                trace!("accept failed, {}", e);
                            }
                        },
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        },
        Some(tlscfg) => {
            let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tlscfg));
            let timeout = config.tcp.tls.timeout.0.clone();

            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Err(e) => {
                                #[cfg(debug_assertions)]
                                {
                                    trace!("accept failed, {}", e);
                                }
                            },
                            Ok((stream, addr)) => {
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
                                                    #[cfg(debug_assertions)]
                                                    {
                                                        trace!("tls handshake failed, {}, {}", addr, e);
                                                    }
                                                },
                                            }
                                        },
                                        None => {
                                            #[cfg(debug_assertions)]
                                            {
                                                trace!("tls handshake timeout, {}", addr);
                                            }
                                        },
                                    }
                                });
                            },
                        }
                    },
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    }
                }
            }
        }
    }

    info!("gracefully shutdown");
}
