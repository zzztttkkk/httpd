use std::sync::Arc;

use crate::{
    ctx::ConnContext,
    http2,
    message::{Message, MessageReadCode},
    protocols::Protocol,
    services::common::Service,
    ws,
};

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
    #[cfg(debug_assertions)]
    {
        log::trace!(service = service.config().name.as_str(); "connection made, {}", addr);
    }

    let cfg = service.config();
    let r = tokio::io::BufReader::with_capacity(cfg.tcp.read_stream_buf_size.0, r);
    let w = tokio::io::BufWriter::with_capacity(cfg.tcp.read_stream_buf_size.0, w);
    let mut ctx = ConnContext::new(r, w, addr, over_tls, service.config());

    let mut reqmsg = Message::default();
    let mut respmsg = Message::default();

    loop {
        match reqmsg.read_headers(&mut ctx).await {
            MessageReadCode::Ok => match reqmsg.read_const_length_body(&mut ctx).await {
                MessageReadCode::Ok => {
                    match service.http(&ctx, &mut reqmsg, &mut respmsg).await {
                        Ok(next_protocol) => match (&mut respmsg).write_to(&mut ctx).await {
                            Ok(_) => match next_protocol {
                                Protocol::Current { keep_alive } => {
                                    if !keep_alive {
                                        break;
                                    }
                                    reqmsg.clear();
                                    respmsg.clear();
                                    continue;
                                }
                                Protocol::WebSocket => {
                                    ws::serve(ctx, reqmsg).await;
                                    return;
                                }
                                Protocol::Http2 => {
                                    http2::serve(ctx, reqmsg).await;
                                    return;
                                }
                            },
                            Err(e) => {
                                log::debug!("send response failed, {}", e);
                                break;
                            }
                        },
                        Err(e) => {
                            log::error!(service=cfg.name.as_str(); "handle failed, {}", e);
                            break;
                        }
                    };
                }
                MessageReadCode::ConnReadError => {
                    break;
                }
                e => {
                    #[cfg(debug_assertions)]
                    {
                        log::trace!("read request body failed, {:?}", e);
                    }
                    break;
                }
            },
            MessageReadCode::ConnReadError => {
                break;
            }
            e => {
                #[cfg(debug_assertions)]
                {
                    log::trace!("read request header failed, {:?}", e);
                }
                break;
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        log::trace!(service = service.config().name.as_str(); "connection lost, {}", addr);
    }
}
