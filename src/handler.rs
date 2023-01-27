use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::error::HTTPError;
use crate::request::Request;
use crate::response::Response;

type HandlerResult = Result<(), Box<dyn HTTPError + Send>>;

#[async_trait]
pub trait Handler: Send {
    async fn handle(&mut self, req: &mut Request, resp: &mut Response) -> HandlerResult;
}

type FnFutureType = Pin<Box<dyn Future<Output=HandlerResult> + Send>>;
type FnType = Box<dyn (FnMut(&mut Request, &mut Response) -> FnFutureType) + Send>;

pub struct FuncHandler(FnType);

impl FuncHandler {
    pub(crate) fn new(f: FnType) -> Self { Self(f) }
}

#[async_trait]
impl Handler for FuncHandler {
    async fn handle(&mut self, req: &mut Request, resp: &mut Response) -> Result<(), Box<dyn HTTPError + Send>> {
        self.0(req, resp).await;
        Ok(())
    }
}

#[macro_export]
macro_rules! func {
    ($content:expr) => {
        std::boxed::Box::new(
            $crate::handler::FuncHandler::new(
                Box::new(
                    move |_, _| {
                        Box::pin(async move { $content })
                    }
                )
            )
        )
    };
    ($req:ident, _, $content:expr) => {
        std::boxed::Box::new(
            $crate::handler::FuncHandler::new(
                Box::new(
                    move |$req, _| {
                        Box::pin(async move { $content })
                    }
                )
            )
        )
    };
    (_, $resp:ident, $content:expr) => {
        std::boxed::Box::new(
            $crate::handler::FuncHandler::new(
                Box::new(
                    move |_, $resp| {
                        Box::pin(async move { $content })
                    }
                )
            )
        )
    };
    ($req:ident, $resp:ident, $content:expr) => {
        std::boxed::Box::new(
            $crate::handler::FuncHandler::new(
                Box::new(
                    move |$req, $resp| {
                        Box::pin(async move { $content })
                    }
                )
            )
        )
    };
}
