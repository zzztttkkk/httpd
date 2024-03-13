use tokio::io::{AsyncWrite, AsyncWriteExt};
use utils::luxon;

use crate::{appender::Appender, file_appender::FileAppender};

pub enum RotationKind {
    Hourly,
    Daily,
}

pub struct RotationFileAppender {
    inner: FileAppender,

    rat: u128,
    kind: RotationKind,

    name_prefix: String,
    base_name: String,
    file_ext: String,
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
            Ok(_) => {
                self.rotate().await;
                return Ok(());
            }
            Err(e) => {
                if self.rotate().await {
                    return self.inner.writeall(buf).await;
                }
                return Err(e);
            }
        }
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush().await
    }
}

impl RotationFileAppender {
    async fn rotate(&mut self) -> bool {

        let now = luxon::unixmills();
        if now < self.rat {
            return false;
        }

        let newname;
        match self.kind {
            RotationKind::Hourly => {
                newname = format!("{}{}{}", &self.name_prefix, &self.base_name, "");
            }
            RotationKind::Daily => {
                newname = format!("{}{}{}", &self.name_prefix, &self.base_name, "");
            }
        }

        for _ in 0..10 {}

        true
    }
}
