use std::collections::HashMap;

use async_trait::async_trait;

use crate::http::context::Context;
use crate::http::handler::Handler;
use crate::http::middleware::{FuncMiddleware, Middleware};
use crate::utils;

pub struct Mux {
    middleware: Vec<Box<dyn Middleware>>,
    map: HashMap<String, Box<dyn Handler>>,
    not_found: Option<Box<dyn Handler>>,
}

impl Mux {
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

unsafe impl Send for Mux {}
unsafe impl Sync for Mux {}

#[async_trait]
impl Handler for Mux {
    async fn handle(&self, ctx: &mut Context) {
        // the `tmp` used before the user change the request path
        let mut tmp: &str = unsafe { std::mem::transmute(ctx.request().uri().path().as_str()) };

        loop {
            if tmp.is_empty() {
                break;
            }

            match self.map.get(tmp) {
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

                    for m in &self.middleware {
                        m.pre(ctx).await;
                        let _r = sync.read().await;
                        if ctx._pre_stop {
                            break;
                        }
                    }

                    handler.handle(ctx).await;

                    for m in (&self.middleware).iter().rev() {
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

        match &self.not_found {
            None => {
                ctx.response()._status_code = 404;
            }
            Some(func) => {
                func.handle(ctx).await;
            }
        };
    }
}

pub static ACCESS_LOG_BEGIN_KEY: &str = "__acl_begin";

impl Mux {
    pub fn enable_access_log(&mut self, _fp: &str) {
        self.apply(FuncMiddleware::new(
            pre!(ctx, {
                ctx.set(ACCESS_LOG_BEGIN_KEY, Box::new(utils::Time::now()));
            }),
            post!(ctx, {
                let begin = *(ctx.get::<utils::LocalTime>(ACCESS_LOG_BEGIN_KEY).unwrap());

                let mut req = ctx.request();
                let now = utils::Time::now();

                let mut code = ctx.response()._status_code;
                if code == 0 {
                    code = 200;
                }

                println!(
                    "[{}] {} {} {} {}us",
                    now.format(utils::DEFAULT_TIME_LAYOUT),
                    req.method().to_string(),
                    req.uri().path().clone(),
                    code,
                    utils::Time::duration(now, begin)
                        .num_microseconds()
                        .unwrap(),
                );
            }),
        ));
    }
}