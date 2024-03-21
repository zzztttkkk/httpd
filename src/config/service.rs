use std::collections::HashMap;

use serde::Deserialize;

use utils::anyhow;

use super::{http::HttpConfig, logging::LoggingConfig, tcp::TcpConfig};

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Match {
    #[serde(
        default,
        alias = "Pattern",
        alias = "regexp",
        alias = "Regexp",
        alias = "regex",
        alias = "Regex"
    )]
    pattern: Option<String>,

    #[serde(default, alias = "Headers")]
    headers: HashMap<String, String>, //
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Rewrite {}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct EarlyReturn {}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Rule {
    #[serde(default, alias = "Match")]
    r#match: Match,

    #[serde(default, alias = "InputRewrites", alias = "input_rewrites")]
    inputrewrites: Option<Rewrite>,

    #[serde(default, alias = "OnputRewrites", alias = "onput_rewrites")]
    outputrewrites: Option<Rewrite>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum Service {
    #[serde(alias = "helloworld")]
    HelloWorld,
    #[serde(alias = "fs")]
    FileSystem {
        #[serde(default, alias = "Root")]
        root: String,

        #[serde(default, alias = "Rewrites")]
        rewrites: HashMap<String, Rewrite>,
    },
    #[serde(alias = "forward")]
    Forward {
        #[serde(default, alias = "target", alias = "Target", alias = "TargetAddress")]
        target_addr: String,

        #[serde(default, alias = "Rules")]
        rules: HashMap<String, Rule>,
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
        rules: HashMap<String, Rule>,
    },
}

impl Service {
    pub fn autofix(&mut self, name: &str) -> anyhow::Result<()> {
        match self {
            Service::FileSystem { root, rewrites } => {
                if root.is_empty() {
                    return anyhow::error(&format!("fs service `{}` get an empty root path", name));
                }

                if !std::path::Path::new(root).exists() {
                    return anyhow::error(&format!(
                        "fs service `{}` get an non-exists root path `{}`",
                        name, root
                    ));
                }
                Ok(())
            }
            Service::Forward {
                target_addr: _,
                rules: _,
            } => Ok(()),
            Service::Upstream {
                target_addrs: _,
                rules: _,
            } => Ok(()),
            _ => Ok(()),
        }
    }

    pub fn kind(&self) -> String {
        match self {
            Service::HelloWorld => "Hello world".to_string(),
            Service::FileSystem { root, rewrites: _ } => format!("Fs{{{}}}", root),
            Service::Forward {
                target_addr: _,
                rules: _,
            } => format!("Forward"),
            Service::Upstream {
                target_addrs: _,
                rules: _,
            } => format!("Upstream"),
        }
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::HelloWorld
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ServiceConfig {
    #[serde(skip)]
    pub(crate) idx: usize,

    #[serde(default, alias = "Name")]
    pub(crate) name: String,

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

    #[serde(skip)]
    pub src: String,
}

impl ServiceConfig {
    pub fn autofix(
        &mut self,
        rlog: &LoggingConfig,
        rtcp: &TcpConfig,
        rhttp: &HttpConfig,
    ) -> anyhow::Result<()> {
        self.logging.autofix(&self.name, self.idx)?;
        self.tcp.autofix(Some(rtcp))?;
        self.http.autofix(Some(rhttp))?;
        self.service.autofix(&self.name)?;
        Ok(())
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}
