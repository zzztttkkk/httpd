use clap::Parser;
use serde::Deserialize;

use super::{duration_in_mills::DurationInMills, size_in_bytes::SizeInBytes, tls::ConfigTLS};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ConfigHttp11 {
    #[serde(default)]
    pub conn_idle_timeout: DurationInMills,
}

impl ConfigHttp11 {
    fn autofix(&mut self) {
        self.conn_idle_timeout.less_then(1, 5 * 1000);
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ConfigSocket {
    #[serde(default)]
    pub read_buf_cap: SizeInBytes,

    #[serde(default)]
    pub write_buf_cap: SizeInBytes,

    #[serde(default)]
    pub max_alive_sockets: i64,

    #[serde(default)]
    pub max_waiting_sockets: i64,

    #[serde(default)]
    pub waiting_step: DurationInMills,

    #[serde(default)]
    pub max_waiting_times: i64,
}

impl ConfigSocket {
    fn autofix(&mut self) {
        self.read_buf_cap.less_then(1024, 1024 * 8);
        self.write_buf_cap.less_then(1024, 1024 * 8);
        self.waiting_step.less_then(1, 50);
        if (self.max_waiting_times < 1) {
            self.max_waiting_times = 20;
        }
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ConfigHTTPMessage {
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

impl ConfigHTTPMessage {
    fn autofix(&mut self) {
        self.read_buf_cap.less_then(1, 1024 * 8); // 8KB
        self.max_header_line_size.less_then(1, 1024 * 8 + 16); // 8KB + 16 Byte
        self.max_first_line_size.less_then(1, 1024 * 6 + 64); // 6KB + 64Byte
        self.max_incoming_body_size.less_then(1, 1024 * 1024 * 20); // 20MB
        if self.max_header_count < 1 {
            self.max_header_count = 120;
        }
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Config {
    #[serde(default)]
    pub addr: String,

    #[serde(default)]
    pub tls: ConfigTLS,

    #[serde(default)]
    pub http: ConfigHTTPMessage,

    #[serde(default)]
    pub socket: ConfigSocket,

    #[serde(default)]
    pub http11: ConfigHttp11,
}

impl Config {
    pub fn autofix(&mut self) {
        self.tls.autofix();
        self.http.autofix();
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
