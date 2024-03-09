use std::{io::Write, task::Poll};

use crate::{item::Item, Appender};

pub struct ConsoleAppender<T: Fn(&Item) -> bool + Send + Sync + Unpin + 'static> {
    rendername: String,
    f: T,
}

impl<T: Fn(&Item) -> bool + Send + Sync + Unpin + 'static> tokio::io::AsyncWrite
    for ConsoleAppender<T>
{
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match std::io::stdout().write_all(buf) {
            Ok(_) => Poll::Ready(Ok(buf.len())),
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(std::io::stdout().flush())
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl<T: Fn(&Item) -> bool + Send + Sync + Unpin + 'static> Appender for ConsoleAppender<T> {
    fn renderer(&self) -> &str {
        &self.rendername
    }

    fn filter(&self, item: &Item) -> bool {
        (&self.f)(item)
    }
}

impl<T: Fn(&Item) -> bool + Send + Sync + Unpin + 'static> ConsoleAppender<T> {
    pub fn new(renderer: &str, filter: T) -> Self {
        Self {
            rendername: renderer.to_string(),
            f: filter,
        }
    }
}
