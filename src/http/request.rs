use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio::sync::Mutex;

pub struct Request {}

impl Request {
    pub async fn from11<R: AsyncBufReadExt + Unpin>(reader: Arc<Mutex<R>>) -> Result<Box<Request>, i32> {
        let mut reader = reader.lock().await;
        let v = reader.read_u8().await;

        Err(12)
    }
}
