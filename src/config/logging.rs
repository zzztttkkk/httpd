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

    #[serde(default, alias = "Lossy")]
    pub lossy: Option<bool>,

    #[serde(default, alias = "BufLines")]
    pub buf_lines: Option<usize>,
}

impl LoggingConfig {
    pub(crate) fn autofix(&mut self, name: &str, root: Option<&Self>) -> anyhow::Result<()> {
        self.service_name = name.to_string();

        match root {
            Some(root) => {
                if root.disable.is_some() && root.disable.unwrap() {
                    self.disable = Some(true);
                }
                if self.directory.is_empty() {
                    self.directory = format!("{}/{}/", root.directory, self.service_name);
                }
            }
            None => {}
        }

        if self.directory.is_empty() && name == "" {
            self.directory = "./log".to_string();
        }

        if self.buf_lines.is_some() && self.buf_lines.unwrap() < 10240 {
            self.buf_lines = Some(10240);
        }
        Ok(())
    }

    pub fn init(
        &self,
    ) -> Option<(
        Box<dyn tracing::Subscriber + Send + Sync>,
        Option<tracing_appender::non_blocking::WorkerGuard>,
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
            return Some((Box::new(subscriber), None));
        }

        let level =
            tracing::Level::from_str(self.level.as_str()).map_or(tracing::Level::INFO, |v| v);

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

        let mut builder = tracing_appender::non_blocking::NonBlockingBuilder::default();
        if self.lossy.is_some() {
            builder = builder.lossy(self.lossy.unwrap());
        } else {
            builder = builder.lossy(false);
        }
        if self.buf_lines.is_some() {
            builder = builder.buffered_lines_limit(self.buf_lines.unwrap());
        }

        let (appender, guard) = builder.finish(appender);

        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_ansi(false)
            .with_target(false)
            .with_writer(appender)
            .with_max_level(level)
            .with_file(true)
            .with_line_number(true)
            .finish();

        Some((Box::new(subscriber), Some(guard)))
    }
}
