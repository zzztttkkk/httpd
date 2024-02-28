use serde::Deserialize;

use self::{http::HttpConfig, tcp::TcpConfig};

mod bytes_size;
mod duration_in_millis;
mod http;
mod split_uint;
mod tcp;
mod tls;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Config {
    #[serde(default)]
    pub tcp: TcpConfig,

    #[serde(default)]
    pub http: HttpConfig,
}

impl Config {
    pub fn autofix(&mut self) {
        self.tcp.autofix();
        self.http.autofix();
    }
}
