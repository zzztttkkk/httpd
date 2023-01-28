use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, MutexGuard};

use crate::error::HTTPError;
use crate::request::Request;
use crate::response::Response;

type HandlerResult = Result<(), Box<dyn HTTPError + Send>>;

#[async_trait]
pub trait Handler: Send {
    async fn handle(&mut self, req: &mut Request, resp: &mut Response) -> HandlerResult;
}

pub struct RequestWrapper(usize);

impl RequestWrapper {
    pub fn unwrap(&mut self) -> &mut Request { unsafe { std::mem::transmute(self.0) } }
}

pub struct ResponseWrapper(usize);

impl ResponseWrapper {
    pub fn unwrap(&mut self) -> &mut Response { unsafe { std::mem::transmute(self.0) } }
}

type FnFutureType = Pin<Box<dyn Future<Output=HandlerResult> + Send>>;
type FnType = Box<dyn (FnMut(RequestWrapper, ResponseWrapper) -> FnFutureType) + Send>;

pub struct FuncHandler(FnType);

impl FuncHandler {
    pub(crate) fn new(f: FnType) -> Box<Self> { Box::new(Self(f)) }
}

#[async_trait]
impl Handler for FuncHandler {
    async fn handle(&mut self, req: &mut Request, resp: &mut Response) -> Result<(), Box<dyn HTTPError + Send>> {
        unsafe {
            (
                (self.0)(
                    RequestWrapper(std::mem::transmute(req)),
                    ResponseWrapper(std::mem::transmute(resp)),
                )
            ).await
        }
    }
}

#[macro_export]
macro_rules! func {
    ($content:expr) => {
        $crate::handler::FuncHandler::new(
            Box::new(
                move |_, _| {
                    Box::pin(async move { $content })
                }
            )
        )
    };
    (_, _, $content:expr) => {
        $crate::handler::FuncHandler::new(
            Box::new(
                move |_, _| {
                    Box::pin(async move { $content })
                }
            )
        )
    };
    ($req:ident, _, $content:expr) => {
        $crate::handler::FuncHandler::new(
            Box::new(
                move |mut $req, _| {
                    Box::pin(async move { $content })
                }
            )
        )
    };
    (_, $resp:ident, $content:expr) => {
        $crate::handler::FuncHandler::new(
            Box::new(
                move |_, mut $resp| {
                    Box::pin(async move { $content })
                }
            )
        )
    };
    ($req:ident, $resp:ident, $content:expr) => {
        $crate::handler::FuncHandler::new(
            Box::new(
                move |mut $req, mut $resp| {
                    Box::pin(async move { $content })
                }
            )
        )
    };
}
