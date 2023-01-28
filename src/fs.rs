use std::io::Write;
use std::pin::Pin;

use async_trait::async_trait;

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
        Box::new(
            Self {
                root: root.to_string(),
                prefix: prefix.to_string(),
            }
        )
    }
}

#[async_trait]
impl Handler for FsHandler {
    async fn handle(&mut self, req: &mut Request, resp: &mut Response) -> Result<(), Box<dyn HTTPError + Send>> {
        println!("Req: {:?} {:?}", req as *mut Request, req.headers().compress_type("accept-encoding"));
        let _ = resp.write("Hello".repeat(100).as_bytes()).unwrap();
        let _ = resp.write("World".repeat(100).as_bytes()).unwrap();

        Ok(())
    }
}
