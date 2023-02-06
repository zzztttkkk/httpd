use crate::http::request::Request;
use crate::http::response::Response;

pub struct Context {
    req: Box<Request>,
    resp: Response,
}

impl Context {
    pub fn new(req: Box<Request>) -> Self {
        Self {
            req,
            resp: Response {},
        }
    }

    pub fn default() -> Self {
        Self {
            req: Box::new(Request {}),
            resp: Response {},
        }
    }

    pub fn x(&mut self) {
        println!("XXXX");
    }
}
