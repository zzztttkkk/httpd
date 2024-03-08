use serde::Deserialize;

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

    pub fn init(&self) -> Option<()> {
        None
    }
}
