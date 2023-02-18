use std::future::Future;
use std::pin::Pin;

use crate::http::ctx::Context;

pub type FutureType<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub trait Handler: Send + Sync {
    fn handler<'a: 'b, 'b>(&self, ctx: &'a mut Context) -> FutureType<'b>;
}

type HandlerFuncType = dyn (Fn(&mut Context) -> FutureType) + Send + Sync;

pub struct FuncHandler(Box<HandlerFuncType>);

impl Handler for FuncHandler {
    fn handler<'a: 'b, 'b>(&self, ctx: &'a mut Context) -> FutureType<'b> {
        (self.0)(ctx)
    }
}

impl FuncHandler {
    pub fn new(f: Box<HandlerFuncType>) -> Self {
        FuncHandler(f)
    }
}

#[macro_export]
macro_rules! func {
    ($ctx:ident, $content:expr) => {
        $crate::http::handler::FuncHandler::new(Box::new(move |$ctx| {
            Box::pin(async move { $content })
        }))
    };
    ($content:expr) => {
        $crate::http::handler::FuncHandler::new(Box::new(move |_| {
            Box::pin(async move { $content })
        }))
    };
}
