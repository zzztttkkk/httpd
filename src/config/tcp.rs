use serde::Deserialize;

use super::{bytes_size::BytesSize, tls::TlsConfig};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct TcpConfig {
    #[serde(default, alias = "address", alias = "Address", alias = "Addr")]
    pub addr: String,

    #[serde(default)]
    pub tls: TlsConfig,

    #[serde(default, alias = "ReadBufSize")]
    pub read_buf_size: BytesSize,

    #[serde(default, alias = "WriteBufSize")]
    pub write_buf_size: BytesSize,

    #[serde(default, alias = "BufSize")]
    pub buf_size: BytesSize,
}

impl TcpConfig {
    pub fn autofix(&mut self) -> Option<String> {
        match self.tls.autofix() {
            Some(e) => {
                return Some(e);
            }
            None => {}
        }

        if self.addr.is_empty() {
            self.addr = "127.0.0.1:8080".to_string();
        }
        if self.read_buf_size.0 < 4096 {
            self.read_buf_size.0 = 8 * 1024;
        }
        if self.write_buf_size.0 < 4096 {
            self.write_buf_size.0 = 8 * 1024;
        }
        if self.buf_size.0 < 4096 {
            self.buf_size.0 = 8 * 1024;
        }

        None
    }
}
