use std::collections::HashSet;

use crate::config::tls::TlsConfig;
use serde::Deserialize;

#[derive(Deserialize, Clone, Default, Debug)]
pub(crate) struct ServerConfig {
    #[serde(default)]
    pub hostnames: Vec<String>,

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

        let mut set = HashSet::new();
        self.hostnames.iter().for_each(|e| {
            set.insert(e.clone());
        });
        self.hostnames.clear();
        set.iter().for_each(|e| {
            self.hostnames.push(e.clone());
        });
    }
}
