use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ProxyRule {}

#[derive(Deserialize, Clone, Debug)]
pub enum ServiceInfo {
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

impl ServiceInfo {
    pub fn autofix(&mut self, name: &str) {
        match self {
            ServiceInfo::FileSystem { root } => {
                if root.is_empty() {
                    panic!("fs service `{}` get an empty root path", name);
                }

                if !std::path::Path::new(root).exists() {
                    panic!(
                        "fs service `{}` get an non-exists root path `{}`",
                        name, root
                    );
                }
            }
            ServiceInfo::Forward { target_addr, rules } => {}
            ServiceInfo::Upstream {
                target_addrs,
                rules,
            } => {}
            _ => {}
        }
    }
}

impl Default for ServiceInfo {
    fn default() -> Self {
        Self::None
    }
}
