use std::pin::Pin;

use async_trait::async_trait;

use crate::error::HTTPError;
use crate::request::Request;
use crate::response::Response;

#[async_trait]
pub trait Handler {
    async fn handle(&self, req: &mut Request, resp: &mut Response) -> Result<(), Box<dyn HTTPError + Send>>;
}
