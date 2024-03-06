use serde::Deserialize;
use std::str::FromStr;

use crate::uitls::anyhow;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct LoggingConfig {
    #[serde(skip)]
    service_name: String,

    #[serde(default, alias = "Disable")]
    pub disable: Option<bool>,

    #[serde(default, alias = "Debug")]
    pub debug: Option<bool>,

    #[serde(default, alias = "Level")]
    pub level: String,

    #[serde(default, alias = "Directory")]
    pub directory: String,

    #[serde(default, alias = "DailyRoration", alias = "Daily", alias = "daily")]
    pub daily_roration: bool,
}

impl LoggingConfig {
    pub(crate) fn autofix(&mut self, name: &str, root: Option<&Self>) -> anyhow::Result<()> {
        self.service_name = name.to_string();

        match root {
            Some(root) => {
                if self.directory.is_empty() {
                    self.directory = format!("{}/{}/", root.directory, self.service_name);
                }
            }
            None => {}
        }

        if self.directory.is_empty() && name == "" {
            self.directory = "./log".to_string();
        }

        Ok(())
    }

    pub fn init(
        &self,
    ) -> Option<(
        Vec<tracing_appender::non_blocking::WorkerGuard>,
        Option<tracing::subscriber::DefaultGuard>,
    )> {
        if self.disable.is_some() && *self.disable.as_ref().unwrap() {
            return None;
        }

        if self.debug.is_some() && *self.debug.as_ref().unwrap() {
            let subscriber = tracing_subscriber::fmt()
                .with_ansi(true)
                .with_target(false)
                .with_max_level(tracing::Level::TRACE)
                .with_file(true)
                .with_line_number(true)
                .finish();

            if !self.service_name.is_empty() {
                let dg = tracing::subscriber::set_default(subscriber);
                return Some((vec![], Some(dg)));
            }

            tracing::subscriber::set_global_default(subscriber).expect("failed to init logging");
            return None;
        }

        let level =
            tracing::Level::from_str(self.level.as_str()).map_or(tracing::Level::INFO, |v| v);

        let mut guards = vec![];

        let appender;
        let name;
        if self.service_name.is_empty() {
            name = "httpd.log".to_string();
        } else {
            name = format!("{}.log", self.service_name);
        }

        if self.daily_roration {
            appender = tracing_appender::rolling::daily(self.directory.to_string(), name.as_str());
        } else {
            appender = tracing_appender::rolling::never(self.directory.to_string(), name.as_str());
        }

        let (appender, guard) = tracing_appender::non_blocking(appender);
        guards.push(guard);

        let builder = tracing_subscriber::fmt()
            .json()
            .with_target(false)
            .with_writer(appender)
            .with_max_level(level)
            .with_file(true)
            .with_line_number(true);

        if self.service_name.is_empty() {
            builder.init();
            Some((guards, None))
        } else {
            let dg = tracing::subscriber::set_default(builder.finish());
            Some((guards, Some(dg)))
        }
    }
}

#[cfg(test)]
mod tests {
    use tracing::{debug, error, info, trace, warn};

    use super::LoggingConfig;

    #[test]
    fn test_logging() {
        let mut logging = LoggingConfig::default();
        logging.autofix("", None).unwrap();
        logging.level = "trace".to_string();

        let _guards = logging.init();

        trace!("trace");
        debug!("debug");
        info!("info");
        warn!("warn");
        error!("error");
    }

    #[test]
    fn test_option() {
        let v = toml::from_str::<LoggingConfig>(
            r"
disable = false
debug = true
        ",
        )
        .unwrap();
        println!("{:?}", v);
    }
}
