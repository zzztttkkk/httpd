use crate::utils::anyhow;
use serde::Deserialize;

use super::{bytes_size::BytesSize, duration_in_millis::DurationInMillis};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct WebsocketConfig {
    #[serde(default, alias = "MaxFrameBodySize")]
    pub max_frame_body_size: BytesSize,

    #[serde(default, alias = "MaxMessageBodySize")]
    pub max_message_body_size: BytesSize,

    #[serde(default, alias = "ReadTimeout")]
    pub read_timeout: DurationInMillis,

    #[serde(default, alias = "Compression")]
    pub compression: Option<i32>,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct HttpConfig {
    #[serde(default, alias = "KeepAlive")]
    pub keep_alive: Option<bool>,

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
    pub compression: Option<i32>,

    #[serde(default, alias = "Websocket", alias = "ws")]
    pub websocket: Option<WebsocketConfig>,
}

impl HttpConfig {
    pub fn autofix(&mut self, root: Option<&Self>) -> anyhow::Result<()> {
        match root {
            Some(root) => {
                if self.keep_alive.is_none() {
                    self.keep_alive = root.keep_alive;
                }
                if self.idle_timeout.is_zero() {
                    self.idle_timeout = root.idle_timeout;
                }
                if self.max_url_size.0 < 1 {
                    self.max_url_size = root.max_url_size;
                }
                if self.max_header_line_size.0 < 1 {
                    self.max_header_line_size = root.max_header_line_size;
                }
                if self.compression.is_none() {
                    self.compression = root.compression;
                }
            }
            None => {}
        }

        // self.compression = std::cmp::min(11, self.compression);

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

        Ok(())
    }
}
