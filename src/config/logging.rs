use std::{collections::HashSet, process::id, str::FromStr};

use serde::Deserialize;

use utils::anyhow;

use super::bytes_size::BytesSize;

#[derive(Deserialize, Clone, Debug)]
pub enum Appender {
    #[serde(alias = "console")]
    Console {
        #[serde(default, alias = "Level")]
        level: Option<String>,
    },

    #[serde(alias = "file")]
    File {
        #[serde(default, alias = "Level")]
        level: Option<String>,

        #[serde(default, alias = "Equal")]
        equal: Option<bool>,

        #[serde(default, alias = "Daily")]
        daily: bool,

        #[serde(default, alias = "BufferSize")]
        bufsize: BytesSize,
    },
}

impl Default for Appender {
    fn default() -> Self {
        Self::Console {
            level: Some("trace".to_string()),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LoggingConfig {
    #[serde(skip)]
    service_name: String,

    #[serde(skip)]
    service_idx: usize,

    #[serde(default, alias = "Disable")]
    pub disable: Option<bool>,

    #[serde(default, alias = "TimeLayout", alias = "time_layout")]
    pub timelayout: String,

    #[serde(default, alias = "Appender")]
    pub appenders: Vec<Appender>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            service_name: Default::default(),
            service_idx: Default::default(),
            disable: Default::default(),
            timelayout: Default::default(),
            appenders: vec![Default::default()],
        }
    }
}

#[derive(Clone)]
struct FilterConifg {
    level: log::Level,
    level_equal: bool,
}

impl logging::Filter for FilterConifg {
    fn filter(&self, item: &logging::Item) -> bool {
        if self.level_equal {
            item.level == self.level
        } else {
            item.level <= self.level
        }
    }
}

impl LoggingConfig {
    pub(crate) fn autofix(&mut self, name: &str, idx: usize) -> anyhow::Result<()> {
        self.service_name = name.to_string();
        self.service_idx = idx;
        Ok(())
    }

    pub fn init(&self, logpath: &str) -> anyhow::Result<Option<Vec<Box<dyn logging::Appender>>>> {
        if self.disable.is_some() && self.disable.unwrap() {
            return Ok(None);
        }

        let mut appenders: Vec<Box<dyn logging::Appender>> = vec![];
        let mut fps: HashSet<String> = HashSet::default();

        for (idx, cfg) in self.appenders.iter().enumerate() {
            match cfg {
                Appender::Console { level } => {
                    let level =
                        log::Level::from_str(&(level.clone()).unwrap_or("trace".to_string()))
                            .unwrap_or(log::Level::Trace);

                    appenders.push(Box::new(logging::ConsoleAppender::new(
                        self.service_idx,
                        "colored",
                        Box::new(FilterConifg {
                            level,
                            level_equal: false,
                        }),
                    )));
                }
                Appender::File {
                    level,
                    equal,
                    daily,
                    bufsize,
                } => {
                    let level =
                        log::Level::from_str(&(level.clone()).unwrap_or("trace".to_string()))
                            .unwrap_or(log::Level::Trace);
                    let equal = equal.unwrap_or(false);

                    let mut path = {
                        if self.service_name.is_empty() {
                            format!("{}/httpd.{}.log", logpath, level)
                        } else {
                            format!("{}/{}/{}.log", logpath, self.service_name, level)
                        }
                    };
                    if fps.contains(&path) {
                        path = {
                            if self.service_name.is_empty() {
                                format!("{}/httpd.{}.{}.log", logpath, level, idx)
                            } else {
                                format!("{}/{}/{}.{}.log", logpath, self.service_name, level, idx)
                            }
                        };
                    }
                    fps.insert(path.clone());

                    let filter = FilterConifg {
                        level,
                        level_equal: equal,
                    };

                    if *daily {
                        appenders.push(Box::new(logging::RotationFileAppender::daily(
                            self.service_idx,
                            &path,
                            bufsize.0,
                            "json",
                            Box::new(filter),
                        )?));
                    } else {
                        appenders.push(Box::new(logging::FileAppender::new(
                            self.service_idx,
                            &path,
                            bufsize.0,
                            "json",
                            Box::new(filter),
                        )?));
                    }
                }
            }
        }
        Ok(Some(appenders))
    }
}
