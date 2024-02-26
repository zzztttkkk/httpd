use serde::Deserialize;
pub(crate) use crate::config::server::ServerConfig;

mod tls;
mod server;
mod duration;
mod split_uint;

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
