use serde::Deserialize;

use utils::anyhow;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct RuntimeConfig {
    #[serde(default, alias = "WorkerThreads")]
    pub worker_threads: u32,

    #[serde(default, alias = "PerCore")]
    pub per_core: Option<bool>,
}

impl RuntimeConfig {
    pub fn autofix(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
