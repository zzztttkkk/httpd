use std::{fs::File, io::BufReader, path::Path};

use clap::Parser;
use serde::Deserialize;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

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
pub struct ConfigTLS {
    #[serde(default)]
    pub cert: String,

    #[serde(default)]
    pub key: String,
}

impl ConfigTLS {
    fn autofix(&mut self) {}

    pub fn load(&self) -> Option<ServerConfig> {
        if self.cert.is_empty() && self.key.is_empty() {
            return None;
        }

        let mut certs = Vec::new();
        for e in rustls_pemfile::certs(&mut BufReader::new(
            File::open(Path::new(self.cert.as_str())).unwrap(),
        ))
        .unwrap()
        {
            certs.push(Certificate(e));
        }
        let mut keys = Vec::new();
        for e in rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(
            File::open(Path::new(self.key.as_str())).unwrap(),
        ))
        .unwrap()
        {
            keys.push(PrivateKey(e));
        }
        if keys.is_empty() {
            for e in rustls_pemfile::rsa_private_keys(&mut BufReader::new(
                File::open(Path::new(self.key.as_str())).unwrap(),
            ))
            .unwrap()
            {
                keys.push(PrivateKey(e));
            }
        }
        return Some(
            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(certs, keys.remove(0))
                .unwrap(),
        );
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
    #[arg(default_value = "127.0.0.1:8080")]
    /// httpd listing address
    pub addr: String,

    #[arg(long)]
    /// config toml file path
    pub file: Option<String>,
}
