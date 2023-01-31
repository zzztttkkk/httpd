use std::io::Write;

use async_trait::async_trait;

use crate::context::Context;
use crate::handler::Handler;

pub struct FsHandler {
    root: String,
    prefix: String,
}

impl FsHandler {
    pub fn new(root: &str, prefix: &str) -> Box<Self> {
        Box::new(Self {
            root: root.to_string(),
            prefix: prefix.to_string(),
        })
    }
}

#[async_trait]
impl Handler for FsHandler {
    async fn handle(&self, ctx: &mut Context) {
        println!("Req: {:?}", ctx.request(),);
        let _ = ctx
            .response()
            .write("Hello".repeat(100).as_bytes())
            .unwrap();
        let _ = ctx
            .response()
            .write("World".repeat(100).as_bytes())
            .unwrap();
    }
}
