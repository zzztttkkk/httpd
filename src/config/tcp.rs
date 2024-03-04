use serde::Deserialize;

use crate::uitls::anyhow;

use super::{bytes_size::BytesSize, tls::TlsConfig};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct TcpConfig {
    #[serde(default, alias = "address", alias = "Address", alias = "Addr")]
    pub addr: String,

    #[serde(default)]
    pub tls: TlsConfig,

    #[serde(default, alias = "ReadStreamBufSize")]
    pub read_stream_buf_size: BytesSize,

    #[serde(default, alias = "WriteStreamBufSize")]
    pub write_stream_buf_size: BytesSize,

    #[serde(default, alias = "BufSize")]
    pub buf_size: BytesSize,
}

impl TcpConfig {
    pub fn autofix(&mut self, root: Option<&Self>) -> anyhow::Result<()> {
        self.tls.autofix(match root {
            Some(v) => Some(&v.tls),
            None => None,
        })?;

        match root {
            Some(root) => {
                if self.read_stream_buf_size.0 < 1 {
                    self.read_stream_buf_size = root.read_stream_buf_size;
                }
                if self.write_stream_buf_size.0 < 1 {
                    self.write_stream_buf_size = root.write_stream_buf_size;
                }
                if self.buf_size.0 < 1 {
                    self.buf_size = root.buf_size;
                }
            }
            None => {}
        }

        if self.read_stream_buf_size.0 < 4096 {
            self.read_stream_buf_size.0 = 8 * 1024;
        }
        if self.write_stream_buf_size.0 < 4096 {
            self.write_stream_buf_size.0 = 8 * 1024;
        }
        if self.buf_size.0 < 4096 {
            self.buf_size.0 = 8 * 1024;
        }

        Ok(())
    }
}
