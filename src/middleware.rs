use std::{any::Any, collections::HashMap};

use async_trait::async_trait;

use crate::{
    error::HTTPError,
    handler::{Handler, HandlerResult},
    request::Request,
    response::Response,
};

pub type MiddlewareResult = HandlerResult;

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn pre(&mut self, req: &mut Request, resp: &mut Response) -> MiddlewareResult;

    async fn post(&mut self, req: &mut Request, resp: &mut Response) -> MiddlewareResult;
}

pub struct FuncMiddleware {
    pre: Option<Box<dyn Handler>>,
    post: Option<Box<dyn Handler>>,
}

impl FuncMiddleware {
    pub fn new(pre: Option<Box<dyn Handler>>, post: Option<Box<dyn Handler>>) -> Box<Self> {
        Box::new(Self { pre, post })
    }
}

#[async_trait]
impl Middleware for FuncMiddleware {
    async fn pre(&mut self, req: &mut Request, resp: &mut Response) -> MiddlewareResult {
        if let Some(mut handler) = self.pre.as_mut() {
            return handler.handle(req, resp).await;
        }
        Ok(())
    }

    async fn post(&mut self, req: &mut Request, resp: &mut Response) -> MiddlewareResult {
        if let Some(mut handler) = self.post.as_mut() {
            return handler.handle(req, resp).await;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! mwfunc {
    ($content:expr) => {
        Some(func!($content))
    };
    (_, _, $content:expr) => {
        Some(func!(_, _, $content))
    };
    ($req:ident, _, $content:expr) => {
        Some(func!($req, _, $content))
    };
    (_, $resp:ident, $content:expr) => {
        Some(func!(_, $resp, $content))
    };
    ($req:ident, $resp:ident, $content:expr) => {
        Some(func!($req, $resp, $content))
    };
}
