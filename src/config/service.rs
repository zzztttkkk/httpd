use std::{collections::HashMap, fmt::format};

use serde::Deserialize;

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
    pub fn autofix(&mut self, name: &str) -> Option<String> {
        match self {
            Service::FileSystem { root } => {
                if root.is_empty() {
                    return Some(format!("fs service `{}` get an empty root path", name));
                }

                if !std::path::Path::new(root).exists() {
                    panic!(
                        "fs service `{}` get an non-exists root path `{}`",
                        name, root
                    );
                }
                None
            }
            Service::Forward { target_addr, rules } => None,
            Service::Upstream {
                target_addrs,
                rules,
            } => None,
            _ => None,
        }
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::None
    }
}
