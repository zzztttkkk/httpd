use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// httpd listing address
    #[arg(default_value = "127.0.0.1:8080")]
    pub addr: String,

    /// config toml file path
    #[arg(short)]
    pub file: Option<String>,

    /// max http message body size, < 1 means unlimits
    #[arg(long, default_value_t = 10 * 1024 * 1024)]
    pub max_body_size: i64,

    /// read buffer default cap
    #[arg(long, default_value_t = 4096)]
    pub read_buf_cap: usize,

    /// max http message header line size, < 1 means unlimits
    #[arg(long, default_value_t = 4097)]
    pub max_header_line_size: i64,
}

impl Config {
    pub fn new() -> Self {
        Self {
            addr: "".to_string(),
            file: None,
            max_body_size: 0,
            read_buf_cap: 0,
            max_header_line_size: 0,
        }
    }
}
