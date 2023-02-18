use std::any::Any;
use std::collections::HashMap;
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
    pub(crate) data: Option<HashMap<String, Box<dyn Any + Send + 'static>>>,
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
            data: None,
            req,
            resp,
            upgrade_protocol: Protocol::Nil,
        }
    }
}

impl Context {
    pub fn get<'s, 'k, T: Any + Send + 'static>(&'s mut self, key: &'k str) -> Option<&'s mut T> {
        match self.data.as_mut() {
            Some(m) => match m.get_mut(key) {
                Some(val) => val.downcast_mut(),
                None => None,
            },
            None => None,
        }
    }

    pub fn peek<'s, 'k, T: Any + Send + 'static>(&'s self, key: &'k str) -> Option<&'s T> {
        match self.data.as_ref() {
            Some(m) => match m.get(key) {
                Some(val) => val.downcast_ref(),
                None => None,
            },
            None => None,
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        match self.data.as_ref() {
            Some(m) => m.contains_key(key),
            None => false,
        }
    }

    pub fn set(&mut self, key: &str, val: Box<dyn Any + Send + 'static>) {
        match self.data.as_mut() {
            Some(m) => {
                m.insert(key.to_string(), val);
            }
            None => {
                let mut m = HashMap::new();
                m.insert(key.to_string(), val);
                self.data = Some(m);
            }
        }
    }
}

impl Context {
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
