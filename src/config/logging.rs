use std::{collections::HashSet, str::FromStr};

use serde::Deserialize;

use utils::anyhow;

use super::bytes_size::BytesSize;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct LoggingConfig {
    #[serde(skip)]
    service_name: String,

    #[serde(default, alias = "Disable")]
    pub disable: Option<bool>,

    #[serde(default, alias = "Debug")]
    pub debug: Option<bool>,

    #[serde(default, alias = "Level")]
    pub level: Option<String>,

    #[serde(default, alias = "EachLevel")]
    pub each_level: Option<bool>,

    #[serde(default, alias = "Directory")]
    pub directory: String,

    #[serde(default, alias = "DailyRoration", alias = "Daily", alias = "daily")]
    pub daily_roration: bool,

    #[serde(default, alias = "BufferSize")]
    pub buffer_size: BytesSize,

    #[serde(default, alias = "RendererName")]
    pub renderer_name: String,
}

#[derive(Clone, Default)]
struct FilterConifg {
    service_name: String,
    level: Option<log::Level>,
    level_equal: Option<bool>,
}

impl FilterConifg {
    fn by_level(&self, item: &logging::Item) -> bool {
        match self.level {
            Some(level) => {
                if self.level_equal.is_some() && self.level_equal.unwrap() {
                    return item.level == level;
                }
                return item.level >= level;
            }
            None => {}
        }
        true
    }
}

impl logging::Filter for FilterConifg {
    fn filter(&self, item: &logging::Item) -> bool {
        if !self.by_level(item) {
            return false;
        }
        return item.service.eq(self.service_name.as_str());
    }
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
                    self.directory = root.directory.clone();
                }
                if self.buffer_size.0 < 8092 {
                    self.buffer_size = root.buffer_size.clone();
                }
            }
            None => {}
        }

        if self.directory.is_empty() && name == "" {
            self.directory = "./log".to_string();
        }

        if self.buffer_size.0 < 8092 {
            self.buffer_size = BytesSize(8092);
        }
        Ok(())
    }

    fn get_renderer_name(&self, names: &mut HashSet<String>, d: &'static str) -> &str {
        if self.renderer_name.is_empty() {
            names.insert(d.to_ascii_lowercase());
            return d;
        }
        names.insert(self.renderer_name.to_lowercase());
        return self.renderer_name.as_str();
    }

    pub fn init(
        &self,
    ) -> anyhow::Result<Option<(Vec<Box<dyn logging::Appender>>, HashSet<String>)>> {
        if self.disable.is_some() && self.disable.unwrap() {
            return Ok(None);
        }

        let mut renderer_names = HashSet::new();

        let mut filter = FilterConifg::default();
        filter.service_name = self.service_name.clone();

        match self.level.as_ref() {
            Some(level) => match log::Level::from_str(&level) {
                Ok(level) => {
                    filter.level = Some(level);
                }
                Err(_) => {}
            },
            None => {}
        }

        let mut appenders: Vec<Box<dyn logging::Appender>> = vec![];

        if self.debug.is_some() && self.debug.unwrap() {
            appenders.push(Box::new(logging::ConsoleAppender::new(
                self.service_name.as_str(),
                self.get_renderer_name(&mut renderer_names, "colored"),
                Box::new(filter.clone()),
            )));
        }

        let mut filename = format!("{}.log", self.service_name.as_str());
        if self.service_name.is_empty() {
            filename = "httpd.log".to_string();
        }

        macro_rules! make_appenders {
            ($self:ident, $renderer_name:ident, $filter:ident, $appenders:ident, $cls:ident, $method:ident) => {
                if $self.each_level.is_some() && $self.each_level.unwrap() {
                    for v in vec![
                        log::Level::Trace,
                        log::Level::Debug,
                        log::Level::Info,
                        log::Level::Warn,
                        log::Level::Error,
                    ] {
                        let mut filter = $filter.clone();
                        filter.level = Some(v);
                        filter.level_equal = Some(true);
                        let appender = logging::$cls::$method(
                            $self.service_name.as_str(),
                            &filename,
                            $self.buffer_size.0,
                            $self.get_renderer_name(&mut $renderer_name, "json"),
                            Box::new(filter),
                        )?;
                        $appenders.push(Box::new(appender));
                    }
                } else {
                    let filter = filter.clone();
                    let appender = logging::$cls::$method(
                        $self.service_name.as_str(),
                        &filename,
                        self.buffer_size.0,
                        self.get_renderer_name(&mut $renderer_name, "json"),
                        Box::new(filter),
                    )?;
                    appenders.push(Box::new(appender));
                }
            };
        }

        if self.daily_roration {
            make_appenders!(
                self,
                renderer_names,
                filter,
                appenders,
                RotationFileAppender,
                daily
            );
        } else {
            make_appenders!(self, renderer_names, filter, appenders, FileAppender, new);
        }
        Ok(Some((appenders, renderer_names)))
    }
}
