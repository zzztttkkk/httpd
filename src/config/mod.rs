pub(crate) use crate::config::server::ServerConfig;
use serde::Deserialize;

mod bytes_size;
mod duration_in_millis;
mod server;
mod split_uint;
mod tls;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
}

impl Config {
    pub fn autofix(&mut self) {
        self.server.autofix();
    }
}
