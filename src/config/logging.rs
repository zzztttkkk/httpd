use std::str::FromStr;

use serde::Deserialize;

use utils::anyhow;

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
            }
            None => {}
        }

        if self.directory.is_empty() && name == "" {
            self.directory = "./log".to_string();
        }
        Ok(())
    }

    pub fn init(&self) -> Option<Vec<Box<dyn logging::Appender>>> {
        if self.disable.is_some() && self.disable.unwrap() {
            return None;
        }

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

        if self.each_level.is_some() && self.each_level.unwrap() {}

        if self.debug.is_some() && self.debug.unwrap() {
            appenders.push(Box::new(logging::ConsoleAppender::new(
                "",
                Box::new(filter.clone()),
            )));
        }

        let mut filename = format!("{}.log", self.service_name.as_str());
        if self.service_name.is_empty() {
            filename = "httpd.log".to_string();
        }

        Some(appenders)
    }
}
