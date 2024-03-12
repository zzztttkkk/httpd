use tokio::io::AsyncWriteExt;

use crate::{appender::Appender, appender::FilterFn, item::Item};

pub struct ConsoleAppender {
    name: String,
    filter_ptr: FilterFn,
    inner: tokio::io::Stdout,
}

#[async_trait::async_trait]
impl Appender for ConsoleAppender {
    async fn writeall(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.inner.write_all(buf).await
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush().await
    }

    fn renderer(&self) -> &str {
        &self.name
    }

    fn filter(&self, item: &Item) -> bool {
        (self.filter_ptr)(item)
    }
}

impl ConsoleAppender {
    pub fn new(renderer: &str, filter: FilterFn) -> Self {
        Self {
            name: renderer.to_string(),
            filter_ptr: filter,
            inner: tokio::io::stdout(),
        }
    }
}
