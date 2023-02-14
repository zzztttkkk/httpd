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
    pub fn new(req: Request) -> Self {
        Self {
            req,
            resp: Response::new(),
            upgrade_protocol: Protocol::Nil,
        }
    }

    pub fn upgrade(&mut self, protocol: Protocol) {
        self.upgrade_protocol = protocol;
    }
}
