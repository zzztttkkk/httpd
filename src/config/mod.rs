use std::collections::HashMap;

use serde::Deserialize;

use self::{http::HttpConfig, logging::LoggingConfig, service::ServiceInfo, tcp::TcpConfig};

mod bytes_size;
mod duration_in_millis;
mod http;
mod logging;
mod service;
mod split_uint;
mod tcp;
mod tls;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Config {
    #[serde(default, alias = "Logging", alias = "Log", alias = "log")]
    pub logging: LoggingConfig,

    #[serde(default, alias = "Tcp")]
    pub tcp: TcpConfig,

    #[serde(default, alias = "Http")]
    pub http: HttpConfig,

    #[serde(default, alias = "Services")]
    pub services: HashMap<String, ServiceInfo>,
}

impl Config {
    pub fn autofix(&mut self) {
        self.logging.autofix();
        self.tcp.autofix();
        self.http.autofix();
        for (name, service) in self.services.iter_mut() {
            service.autofix(&name);
        }
    }
}
