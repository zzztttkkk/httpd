use std::io::Write;

use crate::{
    config::service::ServiceConfig, ctx::ConnContext, message::Message, request::RequestQueryer,
    response::ResponseBuilder,
};

use super::common::Service;

pub struct HelloWorldService(&'static ServiceConfig);

impl HelloWorldService {
    pub fn new(cfg: &'static ServiceConfig) -> Self {
        Self(cfg)
    }
}

impl Service for HelloWorldService {
    fn serve<R: tokio::io::AsyncRead + Unpin + Send, W: tokio::io::AsyncWrite + Unpin + Send>(
        &self,
        r: R,
        w: W,
        addr: std::net::SocketAddr,
    ) -> impl std::future::Future<Output = ()> + Send {
        async move {
            log::trace!("connection made: {}", addr);

            let r = tokio::io::BufReader::with_capacity(self.0.tcp.read_stream_buf_size.0, r);
            let w = tokio::io::BufWriter::with_capacity(self.0.tcp.read_stream_buf_size.0, w);
            let mut ctx = ConnContext::new(r, w, addr, self.0);

            let mut reqmsg = Message::default();
            let reqmsg = &mut reqmsg;

            let mut buidler = ResponseBuilder::new();
            (&mut buidler)
                .with_code(200, "OK")
                .with_version(1, 1)
                .append_header("server", "httpd");

            let mut resp = buidler.finish();
            let resp = &mut resp;
            _ = resp.body.write("Hello World!".as_bytes());

            loop {
                match reqmsg.read_headers(&mut ctx).await {
                    crate::message::MessageReadCode::Ok => {
                        match reqmsg.read_const_length_body(&mut ctx).await {
                            crate::message::MessageReadCode::Ok => {
                                let request = RequestQueryer::new(reqmsg);
                                log::trace!(
                                    "receive request: {}, {} {} {}",
                                    addr,
                                    request.method(),
                                    request.url(),
                                    request.version()
                                );

                                match resp.write_to(&mut ctx).await {
                                    Ok(_) => {
                                        reqmsg.clear();
                                        continue;
                                    }
                                    Err(e) => {
                                        log::trace!("write response failed: {}, {}", addr, e);
                                        break;
                                    }
                                }
                            }
                            e => {
                                log::trace!("read http request body failed, {}, {:?}", addr, e);
                                break;
                            }
                        }
                    }
                    e => {
                        #[cfg(debug_assertions)]
                        {
                            log::trace!("read http request headers failed, {}, {:?}", addr, e);
                        }
                        break;
                    }
                }
            }
        }
    }

    fn init(&mut self) -> utils::anyhow::Result<()> {
        todo!()
    }
}
