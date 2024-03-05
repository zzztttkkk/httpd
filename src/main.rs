use crate::{
    config::Config,
    services::{fs::FsService, helloworld::HelloWorldService},
};
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
mod services;
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
                                    service.serve(r, w, addr, $config).await;
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
                                        let handshake_result;
                                        if !timeout.is_zero() {
                                            match tokio::time::timeout(timeout, acceptor.accept(stream)).await {
                                                Ok(r) => {
                                                    handshake_result = Some(r);
                                                },
                                                Err(_) => {
                                                    handshake_result = None;
                                                },
                                            }
                                        }else{
                                            handshake_result = Some(acceptor.accept( stream).await);
                                        }

                                        match handshake_result {
                                            Some(handshake_result) => {
                                                match handshake_result {
                                                    Ok(stream) => {
                                                        let (r, w) = tokio::io::split(stream);
                                                        service.serve(r, w, addr, $config).await;
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
async fn run(config: &'static ServiceConfig) {
    let _guards = config.logging.init();

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
        config::service::Service::Forward { target_addr, rules } => todo!(),
        config::service::Service::Upstream {
            target_addrs,
            rules,
        } => todo!(),
    }
}

fn main() -> anyhow::Result<()> {
    let config: Config = load_config()?;
    let config: &'static Config = unsafe { std::mem::transmute(&config) };

    let _guards = config.logging.init();

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

    info!("gracefully shutdown");
    Ok(())
}
