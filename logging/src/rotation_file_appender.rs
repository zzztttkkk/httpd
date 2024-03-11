use std::{future::Future, pin::Pin, task::Poll};

use tokio::io::AsyncWriteExt;
use utils::luxon;

use crate::{Appender, FileAppender};

type IoFuture<T> = Pin<Box<dyn Future<Output = std::io::Result<T>> + Send + Sync>>;

enum RotationStage {
    None(u64),
    Flush(IoFuture<()>),               // flush file
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
        let this = self.get_mut();
        match &mut this.stage {
            RotationStage::None(time) => {
                let now = luxon::unix();
                if now < *time {
                    return Poll::Ready(Ok(()));
                }
                let f = Box::pin(this.file.flush());
                this.stage = RotationStage::Flush(f);
                Poll::Pending
            }
            RotationStage::Flush(ff) => {
                let ff = Pin::new(ff);
                match ff.poll(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => {
                            this.stage = RotationStage::Rename(Box::pin(tokio::fs::rename(
                                this.fp.clone(),
                                "to",
                            )));
                            return Poll::Pending;
                        }
                        Err(e) => {
                            return Poll::Ready(Err(e));
                        }
                    },
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                }
            }
            RotationStage::Rename(rf) => {
                let rf = Pin::new(rf);
                match rf.poll(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => {
                            let fp = this.fp.clone();
                            this.stage = RotationStage::Reopen(Box::pin(async move {
                                match tokio::fs::File::options()
                                    .create(true)
                                    .append(true)
                                    .open(fp)
                                    .await
                                {
                                    Ok(f) => Ok(f),
                                    Err(e) => Err(e),
                                }
                            }));
                            Poll::Pending
                        }
                        Err(e) => Poll::Ready(Err(e)),
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
            RotationStage::Reopen(rf) => {
                let rf = Pin::new(rf);
                match rf.poll(cx) {
                    Poll::Ready(result) => match result {
                        Ok(file) => {
                            this.file.reopen(file);
                            Poll::Ready(Ok(()))
                        }
                        Err(e) => Poll::Ready(Err(e)),
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
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
