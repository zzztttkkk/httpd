use serde::Deserialize;

use self::{http::HttpConfig, logging::LoggingConfig, tcp::TcpConfig};

mod bytes_size;
mod duration_in_millis;
mod http;
mod logging;
mod split_uint;
mod tcp;
mod tls;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Config {
    #[serde(default, alias = "Logging", alias = "Log")]
    pub logging: LoggingConfig,

    #[serde(default, alias = "Tcp")]
    pub tcp: TcpConfig,

    #[serde(default, alias = "Http")]
    pub http: HttpConfig,
}

impl Config {
    pub fn autofix(&mut self) {
        self.logging.autofix();
        self.tcp.autofix();
        self.http.autofix();
    }
}
