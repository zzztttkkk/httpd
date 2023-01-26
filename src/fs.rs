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
    pub fn new(root: &str, prefix: &str) -> Self {
        Self {
            root: root.to_string(),
            prefix: prefix.to_string(),
        }
    }
}

#[async_trait]
impl Handler for FsHandler {
    async fn handle(&self, req: &mut Request, resp: &mut Response) -> Result<(), Box<dyn HTTPError + Send>> {
        println!("XXX{:?}", req as *mut Request);
        Ok(())
    }
}
