use serde::Deserialize;

use super::{bytes_size::BytesSize, duration_in_millis::DurationInMillis};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct HttpConfig {
    #[serde(default, alias = "KeepAlive")]
    pub keep_alive: bool,

    #[serde(default, alias = "IdleTimeout")]
    pub idle_timeout: DurationInMillis,

    #[serde(default, alias = "MaxUrlSize")]
    pub max_url_size: BytesSize,

    #[serde(default, alias = "MaxHeaderLineSize")]
    pub max_header_line_size: BytesSize,

    #[serde(default, alias = "MaxHeadersCount")]
    pub max_headers_count: u32,

    #[serde(default, alias = "MaxBodySize")]
    pub max_body_size: BytesSize,

    #[serde(default, alias = "Compression")]
    pub compression: bool,
}

impl HttpConfig {
    pub fn autofix(&mut self) {
        if !self.idle_timeout.is_zero() && self.idle_timeout.as_millis() < 10000 {
            self.idle_timeout = DurationInMillis(std::time::Duration::from_millis(10000));
        }
        if self.max_url_size.u64() < 1 {
            self.max_url_size = BytesSize(8 * 1024); // 8KB
        }
        if self.max_header_line_size.u64() < 1 {
            self.max_header_line_size = BytesSize(6 * 1024); // 6KB
        }
        if self.max_headers_count < 1 {
            self.max_headers_count = 128;
        }
        if self.max_body_size.u64() < 1 {
            self.max_body_size = BytesSize(1024 * 1024 * 10); // 10MB
        }
    }
}
