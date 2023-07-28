use std::{future::Future, pin::Pin, sync::Arc};

use tokio::sync::Mutex;

use super::message::Context;

type HandlerFutureType = Pin<Box<dyn Future<Output = ()> + Send>>;
type ContextType = Arc<Mutex<Context>>;

pub trait Handler: Send + Sync {
    fn handle(&self, ctx: ContextType) -> HandlerFutureType;
}

type HandlerFuncType = dyn (Fn(ContextType) -> HandlerFutureType) + Send + Sync;

pub struct FuncHandler(Box<HandlerFuncType>);

impl FuncHandler {
    pub fn new(func: Box<HandlerFuncType>) -> Self {
        return Self(func);
    }
}

impl Handler for FuncHandler {
    fn handle(&self, ctx: ContextType) -> HandlerFutureType {
        return (self.0)(ctx);
    }
}
