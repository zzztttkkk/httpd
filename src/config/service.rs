use std::collections::HashMap;

use serde::Deserialize;

use crate::uitls::anyhow;

use super::{http::HttpConfig, logging::LoggingConfig, tcp::TcpConfig};

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ProxyRule {}

#[derive(Deserialize, Clone, Debug)]
pub enum Service {
    #[serde(alias = "none")]
    None,
    #[serde(alias = "helloworld")]
    HelloWorld,
    #[serde(alias = "fs")]
    FileSystem {
        #[serde(default, alias = "Root")]
        root: String,
    },
    #[serde(alias = "forward")]
    Forward {
        #[serde(default, alias = "target", alias = "Target", alias = "TargetAddress")]
        target_addr: String,

        #[serde(default, alias = "Rules")]
        rules: HashMap<String, ProxyRule>,
    },
    #[serde(alias = "upstream")]
    Upstream {
        #[serde(
            default,
            alias = "targets",
            alias = "target_addresses",
            alias = "Targets",
            alias = "TargetAddrs",
            alias = "TargetAddresses"
        )]
        target_addrs: Vec<String>, // ip[? :port][? #weights]

        #[serde(default, alias = "Rules")]
        rules: HashMap<String, ProxyRule>,
    },
}

impl Service {
    pub fn autofix(&mut self, name: &str) -> anyhow::Result<()> {
        match self {
            Service::FileSystem { root } => {
                if root.is_empty() {
                    return Err(anyhow::Error(format!(
                        "fs service `{}` get an empty root path",
                        name
                    )));
                }

                if !std::path::Path::new(root).exists() {
                    return Err(anyhow::Error(format!(
                        "fs service `{}` get an non-exists root path `{}`",
                        name, root
                    )));
                }
                Ok(())
            }
            Service::Forward { target_addr, rules } => Ok(()),
            Service::Upstream {
                target_addrs,
                rules,
            } => Ok(()),
            _ => Ok(()),
        }
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ServiceConfig {
    #[serde(default, alias = "Name")]
    pub name: String,

    #[serde(default, alias = "Host")]
    pub host: String,

    #[serde(default, alias = "Service")]
    pub service: Service,

    #[serde(default, alias = "Logging", alias = "Log", alias = "log")]
    pub logging: LoggingConfig,

    #[serde(default, alias = "Tcp")]
    pub tcp: TcpConfig,

    #[serde(default, alias = "Http")]
    pub http: HttpConfig,
}

impl ServiceConfig {
    pub fn autofix(
        &mut self,
        name: &str,
        rlog: &LoggingConfig,
        rtcp: &TcpConfig,
        rhttp: &HttpConfig,
    ) -> anyhow::Result<()> {
        self.logging.autofix(name, Some(rlog))?;
        self.tcp.autofix(Some(rtcp))?;
        self.http.autofix(Some(rhttp))?;
        self.service.autofix(name)?;
        Ok(())
    }
}
