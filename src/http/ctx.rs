use std::sync::Arc;

use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::ws_handler::WebSocketHandler;

pub enum Protocol {
    Nil,
    Websocket(Arc<dyn WebSocketHandler>),
    Http2,
}

pub struct Context {
    pub(crate) req: Request,
    pub(crate) resp: Response,

    pub(crate) upgrade_protocol: Protocol,
}

impl Context {
    pub fn new(mut req: Request) -> Self {
        let mut resp = Response::new();
        if let Some(ct) = req.msg.headers.accept_encoding() {
            resp.msg.headers.set_content_encoding(ct);
            resp.msg.output_compress_type = Some(ct);
        }

        Self {
            req,
            resp,
            upgrade_protocol: Protocol::Nil,
        }
    }

    pub fn upgrade(&mut self, protocol: Protocol) {
        self.upgrade_protocol = protocol;
    }

    pub fn request(&mut self) -> &mut Request {
        &mut self.req
    }

    pub fn response(&mut self) -> &mut Response {
        &mut self.resp
    }
}
