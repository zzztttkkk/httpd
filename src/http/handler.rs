use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;

use crate::http::ctx::Context;

type FutureType<'a> = Pin<Box<dyn Future<Output=()> + Sync + Send + 'a>>;


pub trait Handler {
    fn handler<'a: 'b, 'b>(&self, ctx: &'a mut Context) -> FutureType<'b>;
}

struct FnHandler(Box<dyn Fn(&mut Context) -> FutureType + Send + Sync>);

impl Handler for FnHandler {
    fn handler<'a: 'b, 'b>(&self, ctx: &'a mut Context) -> FutureType<'b> {
        (self.0)(ctx)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use crate::http::ctx::Context;
    use crate::http::handler::{FnHandler, Handler};

    #[test]
    fn x() {
        let handler = FnHandler(Box::new(|ctx| {
            Box::pin(async move {
                ctx.x();
            })
        }));
        let handler: Box<dyn Handler> = Box::new(handler);

        tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async move {
            let mut ctx = Context::default();
            handler.handler(&mut ctx).await;
        });

        let x = Arc::new(Mutex::new(1));
        let y = x.clone();
        let handler = FnHandler(Box::new(move |ctx| {
            let x = y.clone();
            Box::pin(async move {
                let mut xv = x.lock().await;
                *xv += 1;
                ctx.x();
                println!("X: {}", *xv);
            })
        }));
        let handler: Box<dyn Handler> = Box::new(handler);

        tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async move {
            let mut ctx = Context::default();
            for _ in 0..100 {
                handler.handler(&mut ctx).await;
            }
        });

        let x = x.clone();
        tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async move {
            let xv = x.lock().await;
            println!("------:{}", *xv);
        });
    }
}
