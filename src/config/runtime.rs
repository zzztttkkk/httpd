use serde::Deserialize;

use crate::uitls::anyhow;

use super::duration_in_millis::DurationInMillis;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct RuntimeConfig {
    #[serde(default, alias = "WorkerThreads")]
    pub worker_threads: u32,

    #[serde(default, alias = "ShutdownTimeout")]
    pub shutdown_timeout: DurationInMillis,
}

impl RuntimeConfig {
    pub fn autofix(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
