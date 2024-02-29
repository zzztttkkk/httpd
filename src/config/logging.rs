use serde::Deserialize;
use std::str::FromStr;
use tracing::subscriber;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct LoggingConfig {
    #[serde(default, alias = "Disable")]
    disable: bool,

    #[serde(default, alias = "Debug")]
    debug: bool,

    #[serde(default, alias = "Level")]
    level: String,

    #[serde(default, alias = "Directory")]
    directory: String,

    #[serde(default, alias = "DailyRoration", alias = "Daily", alias = "daily")]
    daily_roration: bool,
}

struct EqualLevelFilter(tracing::Level);

impl<S> tracing_subscriber::layer::Filter<S> for EqualLevelFilter {
    fn enabled(
        &self,
        meta: &tracing_core::Metadata<'_>,
        _: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        meta.level() == &self.0
    }
}

impl LoggingConfig {
    pub(crate) fn autofix(&mut self) {
        if self.disable {
            return;
        }

        if self.directory.is_empty() {
            self.directory = "./log".to_string();
        }
    }

    pub fn init(&self) -> Option<Vec<tracing_appender::non_blocking::WorkerGuard>> {
        if self.disable {
            return None;
        }

        if self.debug {
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
                .with_file(true)
                .with_line_number(true)
                .finish();
            _ = tracing::subscriber::set_global_default(subscriber)
                .expect("failed to init logging");
            return None;
        }

        let levels = vec![
            tracing::Level::TRACE,
            tracing::Level::DEBUG,
            tracing::Level::INFO,
            tracing::Level::WARN,
            tracing::Level::ERROR,
        ];

        let min_level =
            tracing::Level::from_str(self.level.as_str()).map_or(tracing::Level::INFO, |v| v);

        let min_level_idx = levels
            .iter()
            .position(|v| v.as_str() == min_level.to_string())
            .unwrap();

        let levels = &levels[min_level_idx..];

        let mut guards = vec![];

        let mut layers = Vec::new();

        for level in levels {
            let appender;
            let name = format!("httpd.{}.log", level.as_str().to_lowercase());

            if self.daily_roration {
                appender =
                    tracing_appender::rolling::daily(self.directory.to_string(), name.as_str());
            } else {
                appender =
                    tracing_appender::rolling::never(self.directory.to_string(), name.as_str());
            }
            let (appender, guard) = tracing_appender::non_blocking(appender);
            guards.push(guard);

            let layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(appender)
                // .with_writer(std::io::stdout)
                .with_filter(EqualLevelFilter(level.clone()))
                .boxed();

            layers.push(layer);
        }

        tracing_subscriber::registry().with(layers).init();
        Some(guards)
    }
}

#[cfg(test)]
mod tests {
    use tracing::{debug, error, info, trace, warn};

    use super::LoggingConfig;

    #[test]
    fn test_logging() {
        let mut logging = LoggingConfig::default();
        logging.autofix();
        logging.level = "trace".to_string();

        let _guards = logging.init();

        trace!("trace");
        debug!("debug");
        info!("info");
        warn!("warn");
        error!("error");
    }
}
