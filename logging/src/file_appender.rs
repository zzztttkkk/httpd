use std::pin::Pin;

use utils::anyhow;

use crate::{appender::FilterFn, item::Item, Appender};

pub struct FileAppender {
    inner: tokio::io::BufWriter<tokio::fs::File>,
    filter_ptr: FilterFn,
    rendername: String,
}

impl tokio::io::AsyncWrite for FileAppender {
    #[inline]
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let this = self.get_mut();
        let fp = Pin::new(&mut this.inner);
        fp.poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        let fp = Pin::new(&mut this.inner);
        fp.poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        let fp = Pin::new(&mut this.inner);
        fp.poll_shutdown(cx)
    }
}

impl Appender for FileAppender {
    #[inline]
    fn renderer(&self) -> &str {
        &self.rendername
    }

    #[inline]
    fn filter(&self, item: &Item) -> bool {
        (self.filter_ptr)(item)
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
}
