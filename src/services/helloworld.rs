use std::net::SocketAddr;

use clap::builder;
use tracing::trace;

use crate::{
    config::service::ServiceConfig,
    ctx::ConnContext,
    http11,
    message::Message,
    protocols::Protocol,
    request::Request,
    response::{Response, ResponseBuilder},
    ws,
};

pub struct HelloWorldService(&'static ServiceConfig);

impl HelloWorldService {
    pub fn new(cfg: &'static ServiceConfig) -> Self {
        Self(cfg)
    }

    pub async fn serve<R: tokio::io::AsyncRead + Unpin, W: tokio::io::AsyncWrite + Unpin>(
        &self,
        r: R,
        w: W,
        addr: SocketAddr,
    ) {
        #[cfg(debug_assertions)]
        {
            trace!("connection made: {}", addr);
        }

        let r = tokio::io::BufReader::with_capacity(self.0.tcp.read_stream_buf_size.0, r);
        let w = tokio::io::BufWriter::with_capacity(self.0.tcp.read_stream_buf_size.0, w);
        let mut ctx = ConnContext::new(r, w, addr, self.0);

        let mut request = Request::default();

        let mut buidler = ResponseBuilder::new();
        (&mut buidler)
            .with_code(200, "OK")
            .with_version(1, 1)
            .append_header("server", "httpd");

        let resp = buidler.finish();

        println!("ptr: {:p}", &resp);

        loop {
            match (&mut (request.msg)).read_headers(&mut ctx).await {
                crate::message::MessageReadCode::Ok => {
                    match (&mut (request.msg)).read_const_length_body(&mut ctx).await {
                        crate::message::MessageReadCode::Ok => {}
                        _ => {}
                    }
                    break;
                }
                e => {
                    #[cfg(debug_assertions)]
                    {
                        trace!("read http message failed, {}, {:?}", ctx.addr, e);
                    }
                    break;
                }
            }
        }
    }
}
