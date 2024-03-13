use tokio::io::AsyncWriteExt;
use utils::anyhow;

use crate::{appender::Appender, appender::FilterFn, item::Item};

pub struct FileAppender {
    inner: tokio::io::BufWriter<tokio::fs::File>,
    filter_ptr: FilterFn,
    rendername: String,
}

#[async_trait::async_trait]
impl Appender for FileAppender {
    #[inline]
    fn renderer(&self) -> &str {
        &self.rendername
    }

    #[inline]
    fn filter(&self, item: &Item) -> bool {
        (self.filter_ptr)(item)
    }

    async fn writeall(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.inner.write_all(buf).await
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush().await
    }
}

impl FileAppender {
    pub fn new(fp: &str, bufsize: usize, renderer: &str, filter: FilterFn) -> anyhow::Result<Self> {
        let fp = anyhow::result(std::fs::File::options().append(true).create(true).open(fp))?;

        let fp = tokio::fs::File::from_std(fp);

        Ok(Self {
            inner: tokio::io::BufWriter::with_capacity(bufsize, fp),
            rendername: renderer.to_string(),
            filter_ptr: filter,
        })
    }

    pub(crate) fn reopen(&mut self, file: tokio::fs::File) {
        self.inner = tokio::io::BufWriter::with_capacity(4096, file)
    }
}
