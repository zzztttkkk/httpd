use std::sync::Arc;

use crate::{ctx::ConnContext, message::Message, services::common::Service};

pub(crate) async fn serve<
    R: tokio::io::AsyncRead + Unpin + Send,
    W: tokio::io::AsyncWrite + Unpin + Send,
>(
    service: Arc<impl Service>,
    r: R,
    w: W,
    addr: std::net::SocketAddr,
    over_tls: bool,
) {
    let cfg = service.config();
    let r = tokio::io::BufReader::with_capacity(cfg.tcp.read_stream_buf_size.0, r);
    let w = tokio::io::BufWriter::with_capacity(cfg.tcp.read_stream_buf_size.0, w);
    let mut ctx = ConnContext::new(r, w, addr, over_tls, service.config());

    let mut reqmsg = Message::default();
    let mut respmsg = Message::default();

    loop {
        match reqmsg.read_headers(&mut ctx).await {
            crate::message::MessageReadCode::Ok => {
                match reqmsg.read_const_length_body(&mut ctx).await {
                    crate::message::MessageReadCode::Ok => {
                        match service.handle(&ctx, &mut reqmsg, &mut respmsg).await {
                            Ok(_) => {
                                // TODO keep-alive

                                reqmsg.clear();
                                respmsg.clear();
                                continue;
                            }
                            Err(e) => {
                                log::error!(service=cfg.name.as_str(); "handle failed, {}", e);
                                break;
                            }
                        };
                    }
                    crate::message::MessageReadCode::ConnReadError => todo!(),
                    e => {
                        break;
                    }
                }
            }
            crate::message::MessageReadCode::ConnReadError => {
                break;
            }
            e => {
                break;
            }
        }
    }
}
