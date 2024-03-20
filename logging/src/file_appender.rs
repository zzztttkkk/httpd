use tokio::io::AsyncWriteExt;
use utils::anyhow;

use crate::{appender::Appender, appender::Filter, item::Item};

pub struct FileAppender {
    inner: tokio::io::BufWriter<tokio::fs::File>,
    filter: Box<dyn Filter>,
    render_name: String,
    service_idx: usize,
    bufsize: usize,
}

#[async_trait::async_trait]
impl Appender for FileAppender {
    #[inline]
    fn renderer(&self) -> &str {
        &self.render_name
    }

    fn service(&self) -> usize {
        self.service_idx
    }

    #[inline]
    fn filter(&self, item: &Item) -> bool {
        (self.filter).filter(item)
    }

    async fn writeall(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.inner.write_all(buf).await
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush().await
    }
}

impl FileAppender {
    #[inline]
    pub(crate) fn open(fp: &str) -> std::io::Result<std::fs::File> {
        std::fs::File::options().append(true).create(true).open(fp)
    }

    pub fn new(
        service: usize,
        fp: &str,
        mut bufsize: usize,
        renderer: &str,
        filter: Box<dyn Filter>,
    ) -> anyhow::Result<Self> {
        let fp = anyhow::result(Self::open(fp))?;

        let fp = tokio::fs::File::from_std(fp);

        if bufsize < 8092 {
            bufsize = 8092;
        }

        Ok(Self {
            service_idx: service,
            inner: tokio::io::BufWriter::with_capacity(bufsize, fp),
            render_name: renderer.to_string(),
            filter,
            bufsize,
        })
    }

    #[inline]
    pub(crate) async fn reopen(&mut self, fp: &str) -> anyhow::Result<()> {
        let fp = anyhow::result(
            tokio::fs::File::options()
                .append(true)
                .create(true)
                .open(fp)
                .await,
        )?;
        self.inner = tokio::io::BufWriter::with_capacity(self.bufsize, fp);
        Ok(())
    }
}
