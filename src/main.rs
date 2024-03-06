use crate::{
    config::Config,
    services::{
        forward::ForwardService, fs::FsService, helloworld::HelloWorldService,
        upstream::UpstreamService,
    },
};
use clap::Parser;
use config::service::ServiceConfig;
use tracing::{info, trace, Instrument};
use uitls::anyhow;

mod compression;
mod config;
mod ctx;
mod message;
mod protocols;
mod request;
mod response;
mod services;
pub mod uitls;
mod ws;

use tracing_futures::WithSubscriber;

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

macro_rules! services_dispatch {
    ($tlscfg:ident, $listener:ident, $config:ident, $service:ident) => {
        match $tlscfg {
            None => loop {
                tokio::select! {
                    result = $listener.accept() => {
                        match result {
                            Ok((mut stream, addr)) => {
                                let service = $service.clone();
                                tokio::spawn(async move {
                                    let (r,w ) = stream.split();
                                    service.serve(r, w, addr).await;
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
                let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(tlscfg));
                let timeout = $config.tcp.tls.timeout.0.clone();

                loop {
                    tokio::select! {
                        result = $listener.accept() => {
                            match result {
                                Err(e) => {
                                    #[cfg(debug_assertions)]
                                    {
                                        trace!("accept failed, {}", e);
                                    }
                                },
                                Ok((stream, addr)) => {
                                    let acceptor = acceptor.clone();
                                    let service = $service.clone();
                                    tokio::spawn(async move {
                                        let handshake_result = match tokio::time::timeout(timeout, acceptor.accept(stream)).await {
                                            Ok(r) => Some(r),
                                            Err(_) => None,
                                        };
                                        match handshake_result {
                                            Some(handshake_result) => {
                                                match handshake_result {
                                                    Ok(stream) => {
                                                        let (r, w) = tokio::io::split(stream);
                                                        service.serve(r, w, addr).await;
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
    };
}

#[tracing::instrument(skip(config), fields(name = config.name), name = "Service")]
async fn _run(config: &'static ServiceConfig) {
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

    match &config.service {
        config::service::Service::HelloWorld => {
            let service = std::sync::Arc::new(HelloWorldService::new(config));
            services_dispatch!(tlscfg, listener, config, service);
        }
        config::service::Service::FileSystem { root: _ } => {
            let service = std::sync::Arc::new(FsService::new(config));
            services_dispatch!(tlscfg, listener, config, service);
        }
        config::service::Service::Forward {
            target_addr: _,
            rules: _,
        } => {
            let service = std::sync::Arc::new(ForwardService::new(config));
            services_dispatch!(tlscfg, listener, config, service);
        }
        config::service::Service::Upstream {
            target_addrs: _,
            rules: _,
        } => {
            let service = std::sync::Arc::new(UpstreamService::new(config));
            services_dispatch!(tlscfg, listener, config, service);
        }
    }
}

async fn run(config: &'static ServiceConfig) {
    let mut sub = config.logging.init();
    let fut = _run(config);
    match sub.take() {
        Some(sub) => {
            fut.in_current_span().with_subscriber(sub).await;
        }
        None => {
            fut.await;
        }
    }
}

fn run_multi_threads(config: &'static Config) -> anyhow::Result<()> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    let mut builder = builder.enable_all();
    if config.runtime.worker_threads > 0 {
        builder = builder.worker_threads(config.runtime.worker_threads as usize);
    }

    let runtime = anyhow::result(builder.build())?;

    runtime.block_on(async {
        let mut set = tokio::task::JoinSet::new();

        for service in config.services.values() {
            set.spawn(async move {
                run(service).await;
            });
        }

        while let Some(_) = set.join_next().await {}
    });

    Ok(())
}

fn run_per_core(config: &'static Config) -> anyhow::Result<()> {
    let lock = std::sync::Arc::new(std::sync::RwLock::new(()));

    for service in config.services.values() {
        let lock = lock.clone();
        let builder = std::thread::Builder::new().name(format!("httpd.service:{}", service.name));
        let result = builder.spawn(move || -> anyhow::Result<()> {
            let _g = anyhow::result(lock.read())?;

            let mut builder = tokio::runtime::Builder::new_current_thread();
            let builder = builder.enable_all();
            let runtime = anyhow::result(builder.build())?;
            runtime.block_on(async {
                run(service).await;
            });
            Ok(())
        });
        _ = anyhow::result(result)?;
    }

    // this loop waiting for the threads hold the read lock
    loop {
        match lock.try_write() {
            Ok(g) => {
                std::mem::drop(g);
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            _ => {
                break;
            }
        }
    }

    let _g = anyhow::result(lock.write())?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let config: Config = load_config()?;
    let config: &'static Config = unsafe { std::mem::transmute(&config) };

    let dispatcher = match config.logging.init() {
        Some(sub) => Some(tracing::dispatcher::Dispatch::new(sub)),
        None => None,
    };

    let _g = match dispatcher.as_ref() {
        Some(v) => Some(tracing::dispatcher::set_default(v)),
        None => None,
    };

    info!("load configuration ok, pid: {}", std::process::id());

    if config.runtime.per_core.is_some() && config.runtime.per_core.unwrap() {
        run_per_core(config)?;
    } else {
        run_multi_threads(config)?;
    }

    info!("shutdown");
    Ok(())
}
