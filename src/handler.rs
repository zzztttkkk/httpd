use std::pin::Pin;

use async_trait::async_trait;

use crate::request::Request;
use crate::response::Response;

#[async_trait]
pub trait Handler {
    async fn handle(&self, mut req: Pin<Box<Request>>, mut resp: Pin<Box<Response>>) -> Result<(), i32>;
}
