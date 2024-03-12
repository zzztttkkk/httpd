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
    pub async fn new(
        fp: &str,
        bufsize: usize,
        renderer: &str,
        filter: FilterFn,
    ) -> anyhow::Result<Self> {
        let fp = tokio::fs::File::options()
            .append(true)
            .create(true)
            .open(fp)
            .await;

        let fp = anyhow::result(fp)?;

        Ok(Self {
            inner: tokio::io::BufWriter::with_capacity(bufsize, fp),
            rendername: renderer.to_string(),
            filter_ptr: filter,
        })
    }

    pub fn sync(
        fp: &str,
        bufsize: usize,
        renderer: &str,
        filter: FilterFn,
    ) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let runtime = anyhow::result(runtime)?;

        let ptr = std::sync::Arc::new(std::sync::Mutex::new(None));
        let ptrc = ptr.clone();
        runtime.block_on(async move {
            let mut ptr = ptrc.lock().unwrap();
            *ptr = Some(Self::new(fp, bufsize, renderer, filter).await)
        });

        std::mem::drop(runtime);

        let mut ptr = ptr.lock().unwrap();
        ptr.take().unwrap()
    }

    pub(crate) fn reopen(&mut self, file: tokio::fs::File) {
        self.inner = tokio::io::BufWriter::with_capacity(4096, file)
    }
}
