use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::sync::Mutex;

use crate::http::ctx::Context;
use crate::http::handler::Handler;
use crate::http::request::Request;
use crate::http::rwtypes::AsyncStream;

async fn http11<T: AsyncStream>(stream: T, handler: Arc<dyn Handler>) {
    let rawstream = Arc::new(Mutex::new(BufStream::with_capacity(4096, 4096, stream)));
    loop {
        let stream = rawstream.clone();
        match Request::from11(stream).await {
            Ok(req) => {
                let mut ctx = Context::new(req);
                handler.handler(&mut ctx).await;
            }
            Err(_) => {
                let stream = rawstream.clone();
                let mut stream = stream.lock().await;
                stream.write("Hello World!".as_bytes()).await;
            }
        }
    }
}
