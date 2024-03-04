use std::collections::HashMap;

use serde::Deserialize;

use crate::uitls::anyhow;

use self::{
    http::HttpConfig,
    logging::LoggingConfig,
    runtime::RuntimeConfig,
    service::{Service, ServiceConfig},
    tcp::TcpConfig,
};

pub mod bytes_size;
pub mod duration_in_millis;
pub mod http;
pub mod logging;
pub mod runtime;
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

    #[serde(default, alias = "Include")]
    pub include: String,

    #[serde(default, alias = "Services")]
    pub services: HashMap<String, ServiceConfig>,
}

impl Config {
    pub fn load(fp: &str) -> anyhow::Result<Self> {
        let txt = anyhow::result(std::fs::read_to_string(fp))?;
        let mut config = anyhow::result(toml::from_str::<Self>(txt.as_str()))?;
        if !config.include.is_empty() {
            for entry in anyhow::result(glob::glob(config.include.as_str()))? {
                match entry {
                    Ok(entry) => {
                        if entry.is_file() {
                            let txt = anyhow::result(std::fs::read_to_string(&entry))?;
                            match toml::from_str::<ServiceConfig>(&txt) {
                                Ok(mut service) => {
                                    service.name = format!("{:?}:{}", &entry, &service.name);
                                    if config.services.contains_key(&service.name) {
                                        return Err(anyhow::Error(format!(
                                            "service name `{}` is exists",
                                            &service.name
                                        )));
                                    }
                                    config.services.insert(service.name.clone(), service);
                                }
                                Err(e) => {
                                    return Err(anyhow::Error(format!(
                                        "load service failed, from `{:?}`, {:?}",
                                        &entry, e
                                    )));
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        if config.services.len() < 1 {
            let mut service = ServiceConfig::default();
            service.name = "helloworld".to_string();
            service.logging.debug = Some(true);
            service.tcp.addr = "0.0.0.0:8080".to_string();
            service.service = Service::HelloWorld;
            service.host = "*".to_string();
            config.services.insert("helloworld".to_string(), service);
        }

        config.autofix()?;

        Ok(config)
    }

    pub fn autofix(&mut self) -> anyhow::Result<()> {
        self.runtime.autofix()?;

        self.logging.autofix("", None)?;

        self.tcp.autofix(None)?;

        self.http.autofix(None)?;

        for (name, service) in self.services.iter_mut() {
            let name = name.to_string();
            let name = name.trim();
            if name.is_empty() {
                return Err(anyhow::Error(format!("empty service name")));
            }
            service.autofix(name, &self.logging, &self.tcp, &self.http)?
        }

        Ok(())
    }
}
