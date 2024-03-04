use serde::Deserialize;

use crate::uitls::anyhow;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct RuntimeConfig {
    #[serde(default, alias = "WorkerThreads")]
    pub worker_threads: u32,
}

impl RuntimeConfig {
    pub fn autofix(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
