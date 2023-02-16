use clap::Parser;
use serde::Deserialize;

use super::{duration_in_mills::DurationInMills, size_in_bytes::SizeInBytes, tls::ConfigTLS};

#[derive(Deserialize, Clone, Default)]
pub struct ConfigHttp11 {
    #[serde(default)]
    pub conn_idle_timeout: DurationInMills,
}

impl ConfigHttp11 {
    fn autofix(&mut self) {
        self.conn_idle_timeout.less_or(1, 5 * 1000);
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct ConfigSocket {
    #[serde(default)]
    pub read_buf_cap: SizeInBytes,

    #[serde(default)]
    pub write_buf_cap: SizeInBytes,
}

impl ConfigSocket {
    fn autofix(&mut self) {
        self.read_buf_cap.less_or(1024, 1024 * 8);
        self.write_buf_cap.less_or(1024, 1024 * 8);
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct ConfigMessage {
    #[serde(default)]
    pub max_incoming_body_size: SizeInBytes,

    #[serde(default)]
    pub read_buf_cap: SizeInBytes,

    #[serde(default)]
    pub max_header_line_size: SizeInBytes,

    #[serde(default)]
    pub max_header_count: usize,

    #[serde(default)]
    pub max_first_line_size: SizeInBytes,

    #[serde(default)]
    pub disbale_compression: bool,
}

impl ConfigMessage {
    fn autofix(&mut self) {
        self.read_buf_cap.less_or(1, 1024 * 8); // 8KB
        self.max_header_line_size.less_or(1, 1024 * 8 + 16); // 8KB + 16 Byte
        self.max_first_line_size.less_or(1, 1024 * 6 + 64); // 6KB + 64Byte
        self.max_incoming_body_size.less_or(1, 1024 * 1024 * 20); // 20MB
        if self.max_header_count < 1 {
            self.max_header_count = 120;
        }
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub addr: String,

    #[serde(default)]
    pub log_config_file: String,

    #[serde(default)]
    pub tls: ConfigTLS,

    #[serde(default)]
    pub message: ConfigMessage,

    #[serde(default)]
    pub socket: ConfigSocket,

    #[serde(default)]
    pub http11: ConfigHttp11,
}

impl Config {
    pub fn autofix(&mut self) {
        self.tls.autofix();
        self.message.autofix();
        self.socket.autofix();
        self.http11.autofix();
    }
}

#[derive(Parser)]
#[command(name = "httpd")]
#[command(about = "A simple http server", long_about = None)]
pub struct Args {
    #[arg(name = "config", default_value = "")]
    /// config file path(toml)
    pub file: String,

    #[arg(long, default_value = "127.0.0.1:8080")]
    /// httpd listing address
    pub addr: String,
}
