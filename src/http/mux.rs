use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use super::handler::{ContextType, Handler, HandlerFutureType};

pub struct Mux {
    map: Arc<RwLock<HashMap<String, Arc<Box<dyn Handler>>>>>,
    not_found_handler: Arc<Box<dyn Handler>>,
}

impl Handler for Mux {
    fn handle(&self, ctx: ContextType) -> HandlerFutureType {
        let map = self.map.clone();
        let not_found = self.not_found_handler.clone();

        return Box::pin(async move {
            let _ctx = ctx.lock().await;
            let mut path = _ctx.req.path().to_string();
            drop(_ctx);
            let map = map.read().await;

            loop {
                if path.is_empty() {
                    return not_found.handle(ctx.clone()).await;
                }

                match map.get(path.as_str()) {
                    Some(handler) => {
                        return handler.handle(ctx.clone()).await;
                    }
                    None => {
                        match path.rfind('/') {
                            Some(ridx) => {
                                path = path[..ridx].to_string();
                            }
                            None => todo!(),
                        }
                        continue;
                    }
                }
            }
        });
    }
}
