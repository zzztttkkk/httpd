use serde::Deserialize;
use crate::config::tls::TlsConfig;

#[derive(Deserialize, Clone, Default, Debug)]
pub(crate) struct ServerConfig {
    #[serde(default)]
    pub hostname: String,

    #[serde(default)]
    pub addr: String,

    #[serde(default)]
    pub tls: TlsConfig,
}

impl ServerConfig {
    pub(crate) fn autofix(&mut self) {
        if self.addr.is_empty() {
            self.addr = "127.0.0.1:8080".to_string();
        }
        self.tls.autofix();
    }
}
