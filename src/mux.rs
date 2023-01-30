use std::collections::HashMap;

use async_trait::async_trait;

use crate::context::Context;
use crate::error::HTTPError;
use crate::handler::Handler;
use crate::middleware::Middleware;
use crate::request::Request;
use crate::response::Response;

pub struct UnsafeMux {
    middleware: Vec<Box<dyn Middleware>>,
    map: HashMap<String, Box<dyn Handler>>,
    not_found: Option<Box<dyn Handler>>,
}

impl UnsafeMux {
    pub fn new() -> Self {
        Self {
            middleware: vec![],
            map: HashMap::new(),
            not_found: None,
        }
    }

    pub fn register(&mut self, pattern: &str, handler: Box<dyn Handler>) {
        self.map.insert(pattern.to_string(), handler);
    }

    pub fn apply(&mut self, middleware: Box<dyn Middleware>) {
        self.middleware.push(middleware);
    }
}

unsafe impl Send for UnsafeMux {}
unsafe impl Sync for UnsafeMux {}

#[async_trait]
impl Handler for UnsafeMux {
    async fn handle(&mut self, ctx: &mut Context) {
        let mut tmp: &str = unsafe { std::mem::transmute(ctx.request().uri().path().as_str()) };

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
                    let sync = ctx.sync();

                    for m in &mut self.middleware {
                        m.pre(ctx).await;
                        let _r = sync.read().await;
                        if ctx._pre_stop {
                            break;
                        }
                    }

                    handler.handle(ctx).await;

                    for m in &mut self.middleware {
                        m.post(ctx).await;
                        let _r = sync.read().await;
                        if ctx._post_stop {
                            break;
                        }
                    }

                    return;
                }
            }
        }

        match &mut self.not_found {
            None => {
                ctx.response()._status_code = 404;
            }
            Some(func) => {
                func.handle(ctx).await;
            }
        };
    }
}
