use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use async_trait::async_trait;

use crate::error::HTTPError;
use crate::request::Request;
use crate::response::Response;

type HandlerResult = Result<(), Box<dyn HTTPError + Send>>;

#[async_trait]
pub trait Handler: Send {
    async fn handle(&mut self, req: &mut Request, resp: &mut Response) -> HandlerResult;
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
    };
}

pub struct RequestRawPtr(usize);

impl_for_raw_ptr!(RequestRawPtr, Request);

pub struct ResponseRawPtr(usize);

impl_for_raw_ptr!(ResponseRawPtr, Response);

type FnFutureType = Pin<Box<dyn Future<Output = HandlerResult> + Send>>;
type FnType = Box<dyn (FnMut(RequestRawPtr, ResponseRawPtr) -> FnFutureType) + Send>;

pub struct FuncHandler(FnType);

impl FuncHandler {
    pub(crate) fn new(f: FnType) -> Box<Self> {
        Box::new(Self(f))
    }
}

#[async_trait]
impl Handler for FuncHandler {
    async fn handle(
        &mut self,
        req: &mut Request,
        resp: &mut Response,
    ) -> Result<(), Box<dyn HTTPError + Send>> {
        unsafe {
            ((self.0)(
                RequestRawPtr(std::mem::transmute(req)),
                ResponseRawPtr(std::mem::transmute(resp)),
            ))
            .await
        }
    }
}

#[macro_export]
macro_rules! func {
    ($content:expr) => {
        $crate::handler::FuncHandler::new(Box::new(move |_, _| Box::pin(async move { $content })))
    };
    (_, _, $content:expr) => {
        $crate::handler::FuncHandler::new(Box::new(move |_, _| Box::pin(async move { $content })))
    };
    ($req:ident, _, $content:expr) => {
        $crate::handler::FuncHandler::new(Box::new(move |mut $req, _| {
            Box::pin(async move { $content })
        }))
    };
    (_, $resp:ident, $content:expr) => {
        $crate::handler::FuncHandler::new(Box::new(move |_, mut $resp| {
            Box::pin(async move { $content })
        }))
    };
    ($req:ident, $resp:ident, $content:expr) => {
        $crate::handler::FuncHandler::new(Box::new(move |mut $req, mut $resp| {
            Box::pin(async move { $content })
        }))
    };
}
