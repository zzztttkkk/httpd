use std::future::Future;
use async_trait::async_trait;
use crate::http::context::{Context, ContextPtr};

pub type ContextType = ContextPtr;

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn handle(&self, ctx: ContextType);
}


#[async_trait]
impl<F, Fut> Handler for F where
    F: Send + Sync + 'static + Fn(ContextType) -> Fut,
    Fut: Future<Output=()> + Send + 'static
{
    async fn handle(&self, ctx: ContextType) {
        (self)(ctx).await;
    }
}


#[cfg(test)]
mod tests {
    use crate::http::context::ContextPtr;
    use crate::http::handler::{ContextType, Handler};

    fn add_handler(h: impl Handler) {}

    #[test]
    fn test() {
        add_handler(|ctx: ContextType| async move {});
    }
}
