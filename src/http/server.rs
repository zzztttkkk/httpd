use std::sync::Arc;

use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::{config::Config, http::connection::Connection, utils};

use super::handler::Handler;

pub struct Server {
    config: Config,
    listener: Option<TcpListener>,
    tls_acceptor: Option<TlsAcceptor>,
}

impl Server {
    pub fn new(cfg: Config) -> Self {
        return Self {
            config: cfg,
            listener: None,
            tls_acceptor: None,
        };
    }

    pub async fn listen(&mut self) {
        let addr = self.config.addr.clone();
        self.listener = Some(TcpListener::bind(&addr).await.unwrap());

        if let Some(tls_cfg) = self.config.tls.load() {
            self.tls_acceptor = Some(TlsAcceptor::from(Arc::new(tls_cfg)));
        }

        println!(
            "[{}] httpd listening @ {} (Tls:{}), Pid: {}",
            utils::Time::currentstr(),
            &addr,
            self.tls_acceptor.is_some(),
            std::process::id()
        );
    }

    pub async fn serve<T: Handler+'static>(&mut self, handler: Arc<T>) {
        let listener = self.listener.as_ref().unwrap();

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _)) => {
                            let tlsaopt = self.tls_acceptor.clone();
                            let handler = handler.clone();

                            tokio::spawn(async move {
                                match tlsaopt {
                                    Some(tlsa)=>{
                                        if let Ok(stream) = tlsa.accept(stream).await {
                                            let (ins, outs) = tokio::io::split(stream);
                                            let mut conn = Connection::new(ins, outs);
                                            conn.handle(handler).await;
                                        }
                                    },
                                    None=>{
                                        let (ins, outs) = stream.into_split();

                                        let mut conn = Connection::new(ins, outs);
                                        conn.handle(handler).await;
                                    }
                                }
                            });
                        },
                        Err(_) => {
                            continue;
                        },
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    println!("[{}] httpd is preparing to shutdown", utils::Time::currentstr());
                    return ;
                }
            }
        }
    }
}
