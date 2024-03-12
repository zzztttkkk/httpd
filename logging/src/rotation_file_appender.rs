use std::{env::set_current_dir, future::Future, pin::Pin, task::Poll};

use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{appender::Appender, file_appender::FileAppender};

pub struct RotationFileAppender {
    inner: FileAppender,
    fp: String,
}

#[async_trait::async_trait]
impl Appender for RotationFileAppender {
    fn renderer(&self) -> &str {
        todo!()
    }

    fn filter(&self, item: &crate::item::Item) -> bool {
        todo!()
    }

    async fn writeall(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self.inner.writeall(buf).await {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush().await
    }
}

impl RotationFileAppender {
    async fn rotate(&mut self) {}
}
