use std::collections::HashMap;

use serde::Deserialize;

use self::{
    http::HttpConfig, logging::LoggingConfig, runtime::RuntimeConfig, service::Service,
    tcp::TcpConfig,
};

pub mod bytes_size;
pub mod duration_in_millis;
pub mod http;
pub mod logging;
mod runtime;
pub mod service;
pub mod split_uint;
pub mod tcp;
pub mod tls;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Config {
    #[serde(default, alias = "Runtime")]
    pub runtime: RuntimeConfig,

    #[serde(default, alias = "Logging", alias = "Log", alias = "log")]
    pub logging: LoggingConfig,

    #[serde(default, alias = "Tcp")]
    pub tcp: TcpConfig,

    #[serde(default, alias = "Http")]
    pub http: HttpConfig,

    #[serde(default, alias = "Services")]
    pub services: HashMap<String, Service>,
}

impl Config {
    pub fn autofix(&mut self) -> Option<String> {
        match self.runtime.autofix() {
            Some(e) => {
                return Some(e);
            }
            _ => {}
        }

        match self.logging.autofix() {
            Some(e) => {
                return Some(e);
            }
            _ => {}
        }

        match self.tcp.autofix() {
            Some(e) => {
                return Some(e);
            }
            _ => {}
        };

        match self.http.autofix() {
            Some(e) => {
                return Some(e);
            }
            _ => {}
        };
        for (name, service) in self.services.iter_mut() {
            match service.autofix(&name) {
                Some(e) => {
                    return Some(e);
                }
                _ => {}
            };
        }
        None
    }
}
