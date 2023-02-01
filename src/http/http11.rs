use std::{
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};
use tokio::io::AsyncWriteExt;

use tokio::{io::BufStream, net::TcpStream};

use crate::{
    config::Config,
    http::{
        context::Context,
        error::HTTPError,
        handler::Handler,
        request,
        response::Response,
        websocket::{websocket_conn, websocket_handshake},
    },
};

pub async fn http11(
    stream: TcpStream,
    counter: Arc<AtomicI64>,
    cfg: &Config,
    handler: &Box<dyn Handler>,
) {
    let bufstream: BufStream<TcpStream>;
    if cfg.socket.read_buf_cap > 0 {
        bufstream =
            BufStream::with_capacity(cfg.socket.read_buf_cap, cfg.socket.write_buf_cap, stream)
    } else {
        bufstream = BufStream::new(stream);
    }

    let mut stream = Box::pin(bufstream);
    let mut rbuf = String::with_capacity(cfg.message.read_buf_cap);

    loop {
        tokio::select! {
            from_result = request::from11(stream.as_mut(), &mut rbuf, cfg) => {
                match from_result {
                    Ok(mut req) => {
                        let mut resp = Response::default(&mut req, cfg.message.disbale_compression);

                        let mut ctx = Context::new(
                            unsafe{std::mem::transmute(req.as_mut())},
                            unsafe{std::mem::transmute(resp.as_mut())}
                        );

                        if req.method() == "GET" {
                            if let Some(conn) = req.headers().get("connection"){
                                if (conn.to_lowercase() == "upgrade"){
                                    if let Some(proto_info) = req.headers().get("upgrade"){
                                        let proto_info = proto_info.to_lowercase();
                                        if proto_info.starts_with("websocket") {
                                            if !(websocket_handshake(&mut ctx).await){
                                                return;
                                            }

                                            let _ = resp.to11(stream.as_mut()).await;
                                            if let Err(_) = (stream.flush().await) {
                                                return ;
                                            }

                                            tokio::spawn(async move {
                                                websocket_conn(stream).await;
                                            });
                                            return;
                                        }
                                    }
                                }
                            }
                        }

                        handler.handle(&mut ctx).await;

                        let _ = resp.to11(stream.as_mut()).await;
                        if let Err(_) = (stream.flush().await) {
                            return ;
                        }
                    }
                    Err(e) => {
                        let code = e.statuscode();
                        if code < 100 {
                            return;
                        }
                        let _ = stream.write(format!("HTTP/1.0 {} Bad Request\r\nContent-Length: 12\r\n\r\nHello World!", e).as_bytes()).await;
                        let _ = stream.flush().await;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(cfg.http11.conn_idle_timeout)) => {
                return;
            }
        }
    }
}
