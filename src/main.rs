use crate::config::Config;
use crate::conn::on_conn;
use clap::Parser;
use config::service::ServiceConfig;
use tracing::{info, trace};
use uitls::anyhow;

mod compression;
mod config;
mod conn;
mod ctx;
mod http11;
mod message;
mod protocols;
mod request;
pub mod uitls;
mod ws;

#[derive(clap::Parser, Debug)]
#[command(name = "httpd")]
#[command(about = "A simple http server", long_about = None)]
pub struct Args {
    #[arg(name = "config", default_value = "")]
    /// config file path(toml)
    pub file: String,
}

fn load_config() -> anyhow::Result<Config> {
    let args;
    #[cfg(debug_assertions)]
    {
        args = Args::parse_from(vec!["httpd", "./httpd.toml"]);
    }
    #[cfg(not(debug_assertions))]
    {
        args = Args::parse();
    }

    if !args.file.trim().is_empty() {
        return Config::load(&args.file);
    }

    let mut config = Config::default();
    config.logging.debug = Some(true);
    config.autofix()?;
    Ok(config)
}

async fn tls_loop(
    listener: &tokio::net::TcpListener,
    tlscfg: boring::ssl::SslAcceptor,
    config: &'static ServiceConfig,
) {
    let acceptor = std::sync::Arc::new(tlscfg);
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
                            let handshake_result;
                            if !timeout.is_zero() {
                                match tokio::time::timeout(timeout, tokio_boring::accept(&acceptor, stream)).await {
                                    Ok(r) => {
                                        handshake_result = Some(r);
                                    },
                                    Err(_) => {
                                        handshake_result = None;
                                    },
                                }

                            }else{
                                handshake_result = Some(tokio_boring::accept(&acceptor, stream).await);
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

#[tracing::instrument(skip(config), fields(name = config.name), name = "Service")]
async fn run(config: &'static ServiceConfig) {
    let listener;
    match tokio::net::TcpListener::bind(config.tcp.addr.clone()).await {
        Ok(v) => {
            listener = v;
        }
        Err(e) => {
            tracing::error!("bind failed, {}", e);
            return;
        }
    }

    let tlscfg = config.tcp.tls.load();
    if tlscfg.is_err() {
        tracing::error!("load tls failed, {}", tlscfg.err().unwrap());
        return;
    }
    let tlscfg = tlscfg.unwrap();

    let mut logo = format!("listening @ {}", config.tcp.addr,);
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
            tls_loop(&listener, tlscfg, config).await;
        }
    }
}

fn main() {
    let config: Config;
    match load_config() {
        Ok(v) => {
            config = v;
        }
        Err(e) => {
            println!("httpd: {}", e);
            return;
        }
    }
    let config = Box::new(config);

    let config: &'static Config = Box::leak(config);

    let _guards = config.logging.init();

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    let mut builder = builder.enable_all();
    if config.runtime.worker_threads > 0 {
        builder = builder.worker_threads(config.runtime.worker_threads as usize);
    }

    match builder.build() {
        Ok(runtime) => {
            runtime.block_on(async {
                let mut set = tokio::task::JoinSet::new();

                for service in config.services.values() {
                    set.spawn(async move {
                        run(service).await;
                    });
                }

                while let Some(_) = set.join_next().await {}
            });

            info!("gracefully shutdown");
        }
        Err(e) => {
            tracing::error!("launch tokio runtime failed, {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rwlock() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let mut set = tokio::task::JoinSet::new();
            for idx in 0..5 {
                set.spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    println!("{}", idx);
                });
            }

            while let Some(_) = set.join_next().await {}
        });

        println!("DONE");
    }
}
