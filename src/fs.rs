use std::io::Write;

use async_trait::async_trait;

use crate::context::Context;
use crate::error::HTTPError;
use crate::handler::Handler;
use crate::request::Request;
use crate::response::Response;

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
    async fn handle(&mut self, ctx: &mut Context) -> Result<(), Box<dyn HTTPError + Send>> {
        println!(
            "Req: {:?} {:?}",
            ctx.request() as *mut Request,
            ctx.request().headers().compress_type("accept-encoding")
        );
        let _ = ctx
            .response()
            .write("Hello".repeat(100).as_bytes())
            .unwrap();
        let _ = ctx
            .response()
            .write("World".repeat(100).as_bytes())
            .unwrap();
        Ok(())
    }
}
