use serde::Deserialize;

use super::{bytes_size::BytesSize, duration_in_millis::DurationInMillis};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct HttpConfig {
    #[serde(default, alias = "KeepAlive")]
    keep_alive: bool,

    #[serde(default, alias = "IdleTimeout")]
    idle_timeout: DurationInMillis,

    #[serde(default, alias = "MaxHeaderLineSize")]
    max_header_line_size: BytesSize,

    #[serde(default, alias = "MaxHeadersCount")]
    max_headers_count: u32,

    #[serde(default, alias = "MaxBodySize")]
    max_body_size: u32,

    #[serde(default, alias = "Compression")]
    compression: bool,
}

impl HttpConfig {
    pub fn autofix(&mut self) {}
}
