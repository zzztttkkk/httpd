use crate::http::request::Request;
use crate::http::response::Response;

pub enum Protocol {
    Nil,
    Websocket,
    Http2,
}

pub struct Context {
    pub(crate) req: Box<Request>,
    pub(crate) resp: Response,

    pub(crate) upgrade_protocol: Protocol,
}

impl Context {
    pub fn new(req: Box<Request>) -> Self {
        Self {
            req,
            resp: Response {},
            upgrade_protocol: Protocol::Nil,
        }
    }

    pub fn upgrade(&mut self, protocol: Protocol) {
        self.upgrade_protocol = protocol;
    }
}
