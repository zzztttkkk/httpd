use std::future::Future;
use std::pin::Pin;

use crate::http::ctx::Context;

type FutureType<'a> = Pin<Box<dyn Future<Output = ()> + Sync + Send + 'a>>;

pub trait Handler: Sync + Send {
    fn handler<'a: 'b, 'b>(&self, ctx: &'a mut Context) -> FutureType<'b>;
}

pub struct FuncHandler(Box<dyn Fn(&mut Context) -> FutureType + Send + Sync>);

impl Handler for FuncHandler {
    fn handler<'a: 'b, 'b>(&self, ctx: &'a mut Context) -> FutureType<'b> {
        (self.0)(ctx)
    }
}

impl FuncHandler {
    pub fn new(f: Box<dyn Fn(&mut Context) -> FutureType + Send + Sync>) -> Self {
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
