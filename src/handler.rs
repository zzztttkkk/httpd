use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use async_trait::async_trait;

use crate::context::Context;
use crate::error::HTTPError;
use crate::request::Request;
use crate::response::Response;

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, ctx: &mut Context);
}

macro_rules! impl_for_raw_ptr {
    ($name:ident, $target:tt) => {
        impl Deref for $name {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                unsafe { std::mem::transmute(self.0) }
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { std::mem::transmute(self.0) }
            }
        }

        impl $name {
            pub(crate) fn new(v: usize) -> Self {
                Self(v)
            }

            pub fn ptr(&self) -> *const Self {
                unsafe { std::mem::transmute(self.0) }
            }
        }
    };
}

pub(crate) struct CtxRawPtr(usize);

impl_for_raw_ptr!(CtxRawPtr, Context);

type HandlerFnFutureType = Pin<Box<dyn Future<Output = ()> + Send>>;
type HandlerFnType = Box<dyn (Fn(CtxRawPtr) -> HandlerFnFutureType) + Send + Sync>;

pub struct FuncHandler(HandlerFnType);

impl FuncHandler {
    pub(crate) fn new(f: HandlerFnType) -> Box<Self> {
        Box::new(Self(f))
    }
}

#[async_trait]
impl Handler for FuncHandler {
    async fn handle(&self, ctx: &mut Context) {
        unsafe { ((self.0)(CtxRawPtr(std::mem::transmute(ctx)))).await }
    }
}

#[macro_export]
macro_rules! func {
    ($content:expr) => {
        $crate::handler::FuncHandler::new(Box::new(move |_| Box::pin(async move { $content })))
    };
    (_, $content:expr) => {
        $crate::handler::FuncHandler::new(Box::new(move |_| Box::pin(async move { $content })))
    };
    ($ctx:ident, $content:expr) => {
        $crate::handler::FuncHandler::new(Box::new(move |mut $ctx| {
            Box::pin(async move { $content })
        }))
    };
}
