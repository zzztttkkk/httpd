use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    vec,
};

use crate::logging::{
    init as logging_init, ColorfulLineRendererBuilder, JsonLineRenderer, Renderer, ShutdownGuard,
};
use serde::Deserialize;

use crate::utils::{anyhow, paths};
use slab::Slab;

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
mod matchs;
pub mod runtime;
pub mod service;
pub mod split_uint;
pub mod tcp;
pub mod tls;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Config {
    #[serde(default, alias = "Workdir")]
    pub workdir: String,

    #[serde(default, alias = "Runtime")]
    pub runtime: RuntimeConfig,

    #[serde(default, alias = "Logging", alias = "Log", alias = "log")]
    pub logging: LoggingConfig,

    #[serde(default, alias = "LogPath", alias = "log_path")]
    pub logpath: String,

    #[serde(default, alias = "LogLevel", alias = "log_level")]
    pub loglevel: Option<String>,

    #[serde(default, alias = "Tcp")]
    pub tcp: TcpConfig,

    #[serde(default, alias = "Http")]
    pub http: HttpConfig,

    #[serde(default, alias = "Include")]
    pub include: String, // glob pattern for service config toml files

    #[serde(default, alias = "Includes")]
    pub includes: Vec<String>, // glob patterns for service config toml files

    #[serde(default, alias = "Services")]
    pub services: HashMap<String, ServiceConfig>,

    #[serde(skip)]
    service_logging_appender_map: Slab<Vec<usize>>,
}

impl Config {
    fn include(&mut self, pattern: &str) -> anyhow::Result<()> {
        for entry in anyhow::result(glob::glob(pattern))? {
            match entry {
                Ok(entry) => {
                    if !entry.is_file() || entry.as_path().file_name().is_none() {
                        continue;
                    }

                    let _path = entry.as_path().to_string_lossy().to_string();
                    let basename = entry
                        .as_path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    if basename.starts_with(".") {
                        continue;
                    }

                    let txt = anyhow::result(std::fs::read_to_string(&entry))?;
                    match toml::from_str::<ServiceConfig>(&txt) {
                        Ok(mut service) => {
                            service.name = service.name.trim().to_string();
                            if service.name.is_empty() {
                                service.name = paths::namewithoutext(&_path)?;
                            }

                            if self.services.contains_key(&service.name) {
                                return anyhow::error(&format!(
                                    "service name `{}` is exists, file: {}",
                                    &service.name, &_path
                                ));
                            }
                            self.services.insert(service.name.clone(), service);
                        }
                        Err(e) => {
                            return anyhow::error(&format!(
                                "load service failed, from `{:?}`, {:?}",
                                &entry, e
                            ));
                        }
                    }
                }
                Err(_) => {}
            }
        }
        Ok(())
    }

    pub fn load(fp: &str) -> anyhow::Result<Self> {
        let txt = anyhow::result(std::fs::read_to_string(fp))?;
        let mut config = anyhow::result(toml::from_str::<Self>(txt.as_str()))?;
        if !config.workdir.is_empty() {
            anyhow::result(std::env::set_current_dir(&config.workdir))?;
        }

        if !config.include.is_empty() {
            config.include(config.include.clone().as_str())?;
        }
        for pattern in config.includes.clone() {
            config.include(pattern.as_str())?;
        }

        if config.services.len() < 1 {
            let mut service = ServiceConfig::default();
            service.name = "helloworld".to_string();
            service.tcp.addr = "0.0.0.0:8080".to_string();
            service.service = Service::HelloWorld { early: None };
            service.host = "*".to_string();
            config.services.insert("helloworld".to_string(), service);
        }

        config.autofix()?;

        Ok(config)
    }

    pub fn autofix(&mut self) -> anyhow::Result<()> {
        self.runtime.autofix()?;

        self.logging.autofix("", 0)?;
        if self.logpath.is_empty() {
            self.logpath = "./logs/".to_string();
        }

        self.tcp.autofix(None)?;

        self.http.autofix(None)?;

        let fidx = self.service_logging_appender_map.insert(vec![]);

        for (name, service) in self.services.iter_mut() {
            let name = name.to_string();
            let name = name.trim();
            if name.is_empty() {
                return anyhow::error(&format!("empty service name"));
            }
            service.name = name.to_string();
            service.idx = self.service_logging_appender_map.insert(vec![]);
            service.autofix(&self.logging, &self.tcp, &self.http)?
        }

        Ok(())
    }

    pub fn logging(&mut self) -> anyhow::Result<ShutdownGuard> {
        let mut appenders = vec![];
        let mut renderer_names = HashSet::<String>::new();

        match self.logging.init(&self.logpath)? {
            Some(aps) => {
                appenders.extend(aps);
            }
            None => {}
        }

        for service in self.services.values() {
            match service.logging.init(&self.logpath)? {
                Some(aps) => {
                    appenders.extend(aps);
                }
                None => {}
            }
        }

        for (idx, appender) in appenders.iter().enumerate() {
            renderer_names.insert(appender.renderer().to_string());

            let idxes = self
                .service_logging_appender_map
                .get_mut(appender.service())
                .unwrap();

            idxes.push(idx);
        }

        let mut renderers: Vec<Box<dyn Renderer>> = vec![];

        for name in renderer_names {
            match name.as_str().to_lowercase().trim() {
                "color" | "colored" | "colorful" | "colorfullinerenderer" => {
                    let mut builder = ColorfulLineRendererBuilder::new();
                    builder
                        .with_name(&name)
                        .with_timelayout(&self.logging.timelayout);
                    renderers.push(Box::new(builder.finish()));
                }
                "" | "json" | "jsonlinerenderer" => {
                    renderers.push(Box::new(JsonLineRenderer::new(
                        &name,
                        &self.logging.timelayout,
                    )));
                }
                _ => {
                    return anyhow::error(&format!(
                        "unknown log line renderer name(`json`, `colored`), `{}`",
                        &name
                    ));
                }
            }
        }

        let level = match self.loglevel.as_ref() {
            Some(level) => match log::Level::from_str(level) {
                Ok(v) => v,
                Err(_) => log::Level::Trace,
            },
            None => log::Level::Trace,
        };

        logging_init(
            level,
            appenders,
            renderers,
            self.service_logging_appender_map.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_glob() {
        for ele in glob::glob("./src/**/*.rs").unwrap() {
            let ele = ele.unwrap();
            let path = ele.as_path().to_string_lossy().to_string();
            let filename = ele
                .as_path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            println!("{} {}", path, filename);
        }
    }
}
