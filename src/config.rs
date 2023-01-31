use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ConfigHttp11 {
    pub conn_idle_timeout: u64,
}

impl ConfigHttp11 {
    fn new() -> Self {
        Self {
            conn_idle_timeout: 0,
        }
    }

    fn autofix(&mut self) {
        if self.conn_idle_timeout < 1 {
            self.conn_idle_timeout = 5; // 5s
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct ConfigSocket {
    pub read_buf_cap: usize,

    pub write_buf_cap: usize,
}

impl ConfigSocket {
    fn new() -> Self {
        Self {
            read_buf_cap: 0,
            write_buf_cap: 0,
        }
    }

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

#[derive(Deserialize, Clone)]
pub struct ConfigMessage {
    pub max_incoming_body_size: i64,

    pub read_buf_cap: usize,

    pub max_header_line_size: i64,

    pub disbale_compression: bool,
}

impl ConfigMessage {
    fn new() -> Self {
        Self {
            max_incoming_body_size: 0,
            read_buf_cap: 0,
            max_header_line_size: 0,
            disbale_compression: false,
        }
    }
    fn autofix(&mut self) {
        if self.read_buf_cap < 1 {
            self.read_buf_cap = 4096; // 4KB
        }
        if self.max_header_line_size < 1 {
            self.max_header_line_size = 1024 * 8; // 8KB
        }
        if self.max_incoming_body_size < 1 {
            self.max_incoming_body_size = 1024 * 1024 * 10; // 10 MB
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub addr: String,

    pub message: ConfigMessage,

    pub socket: ConfigSocket,

    pub http11: ConfigHttp11,
}

impl Config {
    pub fn new() -> Self {
        Self {
            addr: "".to_string(),
            message: ConfigMessage::new(),
            socket: ConfigSocket::new(),
            http11: ConfigHttp11::new(),
        }
    }

    pub fn autofix(&mut self) {
        self.message.autofix();
        self.socket.autofix();
        self.http11.autofix();
    }
}

#[derive(Parser)]
#[command(name = "httpd")]
#[command(about = "A simple http server", long_about = None)]
pub(crate) struct Args {
    #[arg(default_value = "127.0.0.1:8080")]
    /// httpd listing address
    pub addr: String,

    #[arg(long)]
    /// config toml file path
    pub file: Option<String>,
}
