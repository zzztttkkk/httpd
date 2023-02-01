use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Clone, Default)]
pub struct ConfigHttp11 {
    #[serde(default)]
    pub conn_idle_timeout: u64,
}

impl ConfigHttp11 {
    fn autofix(&mut self) {
        if self.conn_idle_timeout < 1 {
            self.conn_idle_timeout = 5; // 5s
        }
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct ConfigSocket {
    #[serde(default)]
    pub read_buf_cap: usize,

    #[serde(default)]
    pub write_buf_cap: usize,
}

impl ConfigSocket {
    fn autofix(&mut self) {
        if self.read_buf_cap > 0 || self.write_buf_cap > 0 {
            if self.read_buf_cap < 1 {
                self.read_buf_cap = 8 * 1024; // 8KB
            }
            if self.write_buf_cap < 1 {
                self.write_buf_cap = 8 * 1024; // 8KB
            }
        }
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct ConfigMessage {
    #[serde(default)]
    pub max_incoming_body_size: usize,

    #[serde(default)]
    pub read_buf_cap: usize,

    #[serde(default)]
    pub max_header_line_size: usize,

    #[serde(default)]
    pub max_header_count: usize,

    #[serde(default)]
    pub max_first_line_size: usize,

    #[serde(default)]
    pub disbale_compression: bool,
}

impl ConfigMessage {
    fn autofix(&mut self) {
        if self.read_buf_cap < 1 {
            self.read_buf_cap = 4096; // 4KB
        }
        if self.max_header_line_size < 1 {
            self.max_header_line_size = 1024 * 8; // 8KB
        }
        if self.max_first_line_size < 1 {
            self.max_first_line_size = 1024 * 6; // 6KB
        }
        if self.max_header_count < 1 {
            self.max_header_count = 9999;
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
    pub message: ConfigMessage,

    #[serde(default)]
    pub socket: ConfigSocket,

    #[serde(default)]
    pub http11: ConfigHttp11,
}

impl Config {
    pub fn autofix(&mut self) {
        self.message.autofix();
        self.socket.autofix();
        self.http11.autofix();
    }
}

#[derive(Parser)]
#[command(name = "httpd")]
#[command(about = "A simple http server", long_about = None)]
pub struct Args {
    #[arg(default_value = "127.0.0.1:8080")]
    /// httpd listing address
    pub addr: String,

    #[arg(long)]
    /// config toml file path
    pub file: Option<String>,
}