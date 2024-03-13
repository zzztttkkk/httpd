use std::any;

use utils::{anyhow, luxon};

use crate::{
    appender::{Appender, FilterFn},
    file_appender::FileAppender,
};

pub enum RotationKind {
    Hourly,
    Daily,
}

pub struct RotationFileAppender {
    inner: FileAppender,

    rotate_at: u128,
    kind: RotationKind,

    fp: String,

    name_prefix: String,
    base_name: String,
    file_ext: String,
}

#[async_trait::async_trait]
impl Appender for RotationFileAppender {
    #[inline]
    fn renderer(&self) -> &str {
        self.inner.renderer()
    }

    #[inline]
    fn filter(&self, item: &crate::item::Item) -> bool {
        self.inner.filter(item)
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
    #[inline]
    fn calcnext(&self) -> u128 {
        match self.kind {
            RotationKind::Hourly => luxon::endofhour(None).unwrap(),
            RotationKind::Daily => luxon::endofday(None).unwrap(),
        }
    }

    #[inline]
    fn next(&mut self) {
        self.rotate_at = self.calcnext();
    }

    async fn rotate(&mut self) -> bool {
        let now = std::time::SystemTime::now();
        if now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            < self.rotate_at
        {
            return false;
        }

        let newname;
        match self.kind {
            RotationKind::Hourly => {
                newname = format!(
                    "{}{}{}",
                    &self.name_prefix,
                    &self.base_name,
                    luxon::fmtlocal(now, ".%Y%m%d%H")
                );
            }
            RotationKind::Daily => {
                newname = format!(
                    "{}{}{}",
                    &self.name_prefix,
                    &self.base_name,
                    luxon::fmtlocal(now, ".%Y%m%d")
                );
            }
        }

        let mut i = 0;
        loop {
            let _name;
            if i == 0 {
                _name = format!("{}.{}", newname, self.file_ext);
            } else {
                _name = format!("{}.{}.{}", newname, luxon::unixnanos(), self.file_ext);
            }

            match tokio::fs::rename(self.fp.as_str(), _name.as_str()).await {
                Ok(_) => {
                    break;
                }
                Err(e) => {
                    if i == 9 {
                        eprintln!("rotation rename failed, {}", e);
                        self.next();
                        return false;
                    }
                    i += 1;
                    continue;
                }
            }
        }

        match self.inner.reopen(&self.fp).await {
            Ok(_) => {
                self.next();
                true
            }
            Err(e) => {
                self.next();
                eprintln!("rotation reopen failed, {}", e);
                false
            }
        }
    }
}
