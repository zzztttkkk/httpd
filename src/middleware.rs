use std::{any::Any, collections::HashMap};

use async_trait::async_trait;

use crate::{
    context::Context,
    error::HTTPError,
    handler::{Handler, HandlerResult},
    request::Request,
    response::Response,
};

pub type MiddlewareResult = HandlerResult;

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn pre(&mut self, ctx: &mut Context) -> MiddlewareResult;

    async fn post(&mut self, ctx: &mut Context) -> MiddlewareResult;
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
    async fn pre(&mut self, ctx: &mut Context) -> MiddlewareResult {
        if let Some(mut handler) = self.pre.as_mut() {
            return handler.handle(ctx).await;
        }
        Ok(())
    }

    async fn post(&mut self, ctx: &mut Context) -> MiddlewareResult {
        if let Some(mut handler) = self.post.as_mut() {
            return handler.handle(ctx).await;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! mwfunc {
    ($content:expr) => {
        Some(func!($content))
    };
    (_, $content:expr) => {
        Some(func!(_, _, $content))
    };
    ($ctx:ident, $content:expr) => {
        Some(func!($ctx, $content))
    };
}
