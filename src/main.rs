use std::sync::Arc;

use crate::{
    config::Config,
    services::{
        common::Service, forward::ForwardService, fs::FsService, helloworld::HelloWorldService,
        upstream::UpstreamService,
    },
};
use clap::Parser;
use config::service::ServiceConfig;
use utils::anyhow;

mod compression;
mod config;
mod ctx;
pub mod internal;
mod message;
mod protocols;
mod request;
mod response;
mod services;
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

async fn accept_loop(
    listener: tokio::net::TcpListener,
    tlscfg: Option<tokio_rustls::rustls::ServerConfig>,
    timeout: std::time::Duration,
    service: impl Service + Send + Sync + 'static,
) {
    if tlscfg.is_some() {
        tls_accept_loop(listener, tlscfg.unwrap(), timeout, service).await;
        return;
    }

    let service = Arc::new(service);
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((mut stream, addr)) => {
                        let service = service.clone();
                        tokio::spawn(async move {
                            let (r,w ) = stream.split();
                            service.serve(r, w, addr).await;
                        });
                    },
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        {
                            log::trace!("accept failed, {}", e);
                        }
                    },
                }
            },
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }
}

async fn tls_accept_loop(
    listener: tokio::net::TcpListener,
    tlscfg: tokio_rustls::rustls::ServerConfig,
    timeout: std::time::Duration,
    service: impl Service + Send + Sync + 'static,
) {
    let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(tlscfg));
    let service = Arc::new(service);
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        {
                            log::trace!("accept failed, {}", e);
                        }
                    },
                    Ok((stream, addr)) => {
                        let acceptor = acceptor.clone();
                        let service = service.clone();
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
                                                log::trace!("tls handshake failed, {}, {}", addr, e);
                                            }
                                        },
                                    }
                                },
                                None => {
                                    #[cfg(debug_assertions)]
                                    {
                                        log::trace!("tls handshake timeout, {}", addr);
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

async fn run(config: &'static ServiceConfig) -> anyhow::Result<()> {
    let listener = anyhow::result(tokio::net::TcpListener::bind(config.tcp.addr.clone()).await)?;

    let tlscfg = anyhow::result(config.tcp.tls.load())?;
    let mut logo = format!("listening @ {}", config.tcp.addr,);
    if tlscfg.is_some() {
        logo = format!("{}, tls ✅", logo);
    }
    log::info!("{}", logo);

    match &config.service {
        config::service::Service::HelloWorld => {
            let service = HelloWorldService::new(config);
            accept_loop(listener, tlscfg, config.tcp.tls.timeout.0, service).await;
        }
        config::service::Service::FileSystem { root: _ } => {
            let service = FsService::new(config);
            accept_loop(listener, tlscfg, config.tcp.tls.timeout.0, service).await;
        }
        config::service::Service::Forward {
            target_addr: _,
            rules: _,
        } => todo!(),
        config::service::Service::Upstream {
            target_addrs: _,
            rules: _,
        } => todo!(),
    };

    Ok(())
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
            set.spawn(async move { run(service).await });
        }

        while let Some(result) = set.join_next().await {
            match result {
                Err(e) => {
                    log::error!("join error, {:?}", e);
                }
                Ok(result) => match result {
                    Err(e) => {
                        log::error!("service serve error, {:?}", e);
                    }
                    _ => {}
                },
            }
        }
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
                match run(service).await {
                    Err(err) => {
                        log::error!("service serve error, {:?}", err);
                    }
                    _ => {}
                };
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

    logging::init(
        None,
        vec![Box::new(logging::ConsoleAppender::new(
            "ColorfulLineRenderer",
            Box::new(|_| true),
        ))],
        vec![Box::new(logging::ColorfulLineRenderer::default())],
    )?;

    log::info!("load configuration ok, pid: {}", std::process::id());

    if config.runtime.per_core.is_some() && config.runtime.per_core.unwrap() {
        run_per_core(config)?;
    } else {
        run_multi_threads(config)?;
    }

    log::info!("shutdown");
    log::logger().flush();
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_wait_all() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let mut fs = vec![];

                for _ in 0..10 {
                    fs.push(tokio::time::sleep(std::time::Duration::from_secs(1)));
                }

                println!(
                    "BEGIN: {}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
                for f in fs {
                    f.await
                }

                println!(
                    "END: {}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
            });
    }
}
