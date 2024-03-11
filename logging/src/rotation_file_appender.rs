use std::{future::Future, pin::Pin, task::Poll};

use tokio::io::{AsyncWrite, AsyncWriteExt};
use utils::luxon;

use crate::{Appender, FileAppender};

type IoFuture<T> = Pin<Box<dyn Future<Output = std::io::Result<T>> + Send + Sync>>;

enum RotationStage {
    None(u64),
    Flush,                             // flush file
    Rename(IoFuture<()>),              // rename current log file
    Reopen(IoFuture<tokio::fs::File>), // reset file
}

pin_project_lite::pin_project! {
    struct Rotation<'a> {
        file: &'a mut FileAppender,
        fp: String,
        stage: RotationStage,

        #[pin]
        _pin: std::marker::PhantomPinned,
    }
}

impl<'a> Future for Rotation<'a> {
    type Output = std::io::Result<()>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        match this.stage {
            RotationStage::None(at) => {
                let now = luxon::unix();
                if now < *at {
                    return Poll::Ready(Ok(()));
                }
                *this.stage = RotationStage::Flush;
                Poll::Pending
            }
            RotationStage::Flush => {
                let file = Pin::new(&mut *this.file);
                match file.poll_flush(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => {
                            *this.stage =
                                RotationStage::Rename(Box::pin(tokio::fs::rename("", "")));
                            Poll::Pending
                        }
                        Err(e) => Poll::Ready(Err(e)),
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
            RotationStage::Rename(fut) => {
                let fut = Pin::new(fut);
                match fut.poll(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Pending,
                        Err(e) => Poll::Ready(Err(e)),
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
            RotationStage::Reopen(_) => todo!(),
        }
    }
}

pub struct RotationFileAppender {
    inner: FileAppender,
    stage: RotationStage,
    fp: String,
}

impl tokio::io::AsyncWrite for RotationFileAppender {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let this = self.get_mut();
        let inner = Pin::new(&mut this.inner);

        let v = inner.poll_write(cx, buf);
        v
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        let inner = Pin::new(&mut this.inner);
        inner.poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        let inner = Pin::new(&mut this.inner);
        inner.poll_shutdown(cx)
    }
}

impl Appender for RotationFileAppender {
    fn renderer(&self) -> &str {
        todo!()
    }

    fn filter(&self, item: &crate::item::Item) -> bool {
        todo!()
    }
}
