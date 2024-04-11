use crate::utils::{anyhow, luxon};

use super::{
    appender::{Appender, Filter},
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

    fn service(&self) -> usize {
        self.inner.service()
    }

    #[inline]
    fn filter(&self, item: &super::item::Item) -> bool {
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
    pub fn daily(
        service: usize,
        fp: &str,
        bufsize: usize,
        renderer: &str,
        filter: Box<dyn Filter>,
    ) -> anyhow::Result<Self> {
        Self::new(RotationKind::Daily, service, fp, bufsize, renderer, filter)
    }

    pub fn new(
        kind: RotationKind,
        service: usize,
        fp: &str,
        bufsize: usize,
        renderer: &str,
        filter: Box<dyn Filter>,
    ) -> anyhow::Result<Self> {
        let inner = FileAppender::new(service, fp, bufsize, renderer, filter)?;
        let mut this = Self {
            inner,
            rotate_at: 0,
            kind,
            fp: fp.to_string(),
            name_prefix: "()".to_string(),
            base_name: "()".to_string(),
            file_ext: "()".to_string(),
        };
        this.next();

        let path = std::path::Path::new(fp);
        let ext = path
            .extension()
            .map_or("".to_string(), |v| v.to_string_lossy().to_string());
        this.file_ext = ext.clone();
        let filename = path
            .file_name()
            .map_or(None, |v| v.to_str().map_or(None, |v| Some(v.to_string())));
        let filename = anyhow::option(filename, "empty filename")?;
        this.name_prefix = (&fp[..fp.len() - filename.len()]).to_string();

        if ext.is_empty() {
            this.base_name = filename.clone();
        } else {
            this.base_name = (&filename.as_str()[..filename.len() - ext.len() - 1]).to_string();
        }
        Ok(this)
    }

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
        let mills = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        if mills < self.rotate_at {
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
#[cfg(test)]
mod tests {
    use crate::{logging::appender, utils::anyhow};

    use super::RotationFileAppender;

    #[test]
    fn test_rotation_new() -> anyhow::Result<()> {
        let _ = RotationFileAppender::new(
            super::RotationKind::Daily,
            0,
            "../log/v.log",
            8092,
            "",
            appender::filter(|_| true),
        )?;
        Ok(())
    }
}
