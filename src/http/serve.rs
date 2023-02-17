use std::{
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::net::{TcpListener, TcpStream};

use crate::{
    config::Config,
    http::{self, Handler},
    utils::{self, AutoCounter},
};

async fn is_alive(stream: &TcpStream) -> bool {
    let mut tmp = [0; 1];
    tokio::select! {
        rr = stream.peek(&mut tmp) =>  {
            return  rr.is_ok();
        }
        _ = tokio::time::sleep(Duration::from_millis(3)) => {
            return false;
        }
    }
}

pub async fn serve(config: &'static Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = config.addr.clone();
    let listener = TcpListener::bind(&addr).await.unwrap();
    let mut tls_acceptor: Option<tokio_rustls::TlsAcceptor> = None;
    let mut proto = "http";
    if let Some(tls_cfg) = config.tls.load() {
        tls_acceptor = Some(tokio_rustls::TlsAcceptor::from(Arc::new(tls_cfg)));
        proto = "https"
    }

    println!(
        "[{}] httpd listening @ {}({}), Pid: {}",
        utils::Time::currentstr(),
        proto,
        &addr,
        std::process::id()
    );

    let ac = AtomicI64::new(0);
    let ac: &'static AtomicI64 = unsafe { std::mem::transmute(&ac) };
    let wc = AtomicI64::new(0);
    let wc: &'static AtomicI64 = unsafe { std::mem::transmute(&wc) };

    let handler: Box<dyn Handler> = Box::new(func!(ctx, {
        println!("XXX");
        ctx.response().text("hello world");
    }));
    let handler: &'static Box<dyn Handler> = unsafe { std::mem::transmute(&handler) };

    loop {
        tokio::select! {
            ar = listener.accept() => {
                match ar {
                    Err(_) => {
                        continue;
                    }
                    Ok((stream, _)) => {
                        let tls_acceptor = tls_acceptor.clone();
                        tokio::spawn(async move {
                            if(config.socket.max_alive_sockets > 0){
                                if(config.socket.max_waiting_sockets > 0 && wc.load(Ordering::Relaxed) > config.socket.max_waiting_sockets){
                                    return;
                                }

                                if(ac.load(Ordering::Relaxed) >= config.socket.max_alive_sockets){
                                    let __awc = AutoCounter::new(wc);

                                    let mut waitting_times: i64 = 0;
                                    while(ac.load(Ordering::Relaxed) > config.socket.max_alive_sockets) {
                                        tokio::time::sleep(config.socket.waiting_step.duration()).await;
                                        waitting_times +=1;
                                        if(waitting_times >= config.socket.max_waiting_times){
                                            return;
                                        }
                                    }

                                    if(!(is_alive(&stream).await)){
                                        return;
                                    }
                                }
                            }


                            if let Some(tls_acceptor) = tls_acceptor {
                                if let Ok(stream) = tls_acceptor.accept(stream).await{
                                    http::conn(stream, ac, config, handler).await;
                                }
                                return;
                            }
                            http::conn(stream, ac, config, handler).await;
                        });
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("[{}] httpd is preparing to shutdown", utils::Time::currentstr());
                loop {
                    if ac.load(Ordering::Relaxed) < 1 {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                println!("[{}] httpd is gracefully shutdown", utils::Time::currentstr());
                return Ok(());
            }
        }
    }
}
