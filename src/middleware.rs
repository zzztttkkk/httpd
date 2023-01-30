use std::{any::Any, collections::HashMap};

use async_trait::async_trait;

use crate::{
    context::Context, error::HTTPError, handler::Handler, request::Request, response::Response,
};

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn pre(&mut self, ctx: &mut Context);

    async fn post(&mut self, ctx: &mut Context);
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
    async fn pre(&mut self, ctx: &mut Context) {
        if let Some(mut handler) = self.pre.as_mut() {
            handler.handle(ctx).await;
        }
    }

    async fn post(&mut self, ctx: &mut Context) {
        if let Some(mut handler) = self.post.as_mut() {
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
