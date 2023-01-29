use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::HTTPError;
use crate::handler::Handler;
use crate::request::Request;
use crate::response::Response;

pub struct Mux {
    map: HashMap<String, Box<dyn Handler>>,
    not_found: Option<Box<dyn Handler>>,
}

impl Mux {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            not_found: None,
        }
    }

    pub fn register(&mut self, pattern: &str, handler: Box<dyn Handler>) {
        self.map.insert(pattern.to_string(), handler);
    }
}

#[async_trait]
impl Handler for Mux {
    async fn handle(
        &mut self,
        req: &mut Request,
        resp: &mut Response,
    ) -> Result<(), Box<dyn HTTPError + Send>> {
        let mut tmp = req.uri().path().as_str();

        loop {
            if tmp.is_empty() {
                break;
            }

            match self.map.get_mut(tmp) {
                None => {
                    if tmp.len() == 1 {
                        break;
                    }
                    match tmp.rfind('/') {
                        None => {}
                        Some(idx) => {
                            tmp = &(tmp[0..idx + 1]);
                        }
                    }
                }
                Some(handler) => {
                    return handler.handle(req, resp).await;
                }
            }
        }

        return match &mut self.not_found {
            None => {
                resp._status_code = 404;
                Ok(())
            }
            Some(func) => func.handle(req, resp).await,
        };
    }
}
