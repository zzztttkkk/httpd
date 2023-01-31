use std::{any::Any, collections::HashMap};

use async_trait::async_trait;

use crate::{
    context::Context, error::HTTPError, handler::Handler, request::Request, response::Response,
};

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn pre(&self, ctx: &mut Context);

    async fn post(&self, ctx: &mut Context);
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
    async fn pre(&self, ctx: &mut Context) {
        if let Some(handler) = self.pre.as_ref() {
            handler.handle(ctx).await;
        }
    }

    async fn post(&self, ctx: &mut Context) {
        if let Some(handler) = self.post.as_ref() {
            handler.handle(ctx).await;
        }
    }
}

#[macro_export]
macro_rules! pre {
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

#[macro_export]
macro_rules! post {
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
