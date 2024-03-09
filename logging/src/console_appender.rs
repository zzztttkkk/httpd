use std::pin::Pin;

use crate::{appender::FilterFn, item::Item, Appender};

pub struct ConsoleAppender {
    rendername: String,
    filter_ptr: FilterFn,
    inner: tokio::io::Stdout,
}

impl tokio::io::AsyncWrite for ConsoleAppender {
    #[inline]
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let this = self.get_mut();
        let inner = Pin::new(&mut this.inner);
        inner.poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        let inner = Pin::new(&mut this.inner);
        inner.poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        let inner = Pin::new(&mut this.inner);
        inner.poll_shutdown(cx)
    }
}

impl Appender for ConsoleAppender {
    #[inline]
    fn renderer(&self) -> &str {
        &self.rendername
    }

    #[inline]
    fn filter(&self, item: &Item) -> bool {
        (&self.filter_ptr)(item)
    }
}

impl ConsoleAppender {
    pub fn new(renderer: &str, filter: FilterFn) -> Self {
        Self {
            rendername: renderer.to_string(),
            filter_ptr: filter,
            inner: tokio::io::stdout(),
        }
    }
}
