use serde::Deserialize;
pub(crate) use crate::config::server::ServerConfig;

mod tls;
mod server;

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
