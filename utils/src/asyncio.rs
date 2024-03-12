use std::future::Future;

use crate::anyhow;

pub fn sync<T>(fut: impl Future<Output = T>) -> anyhow::Result<T> {
    let mut builder = tokio::runtime::Builder::new_current_thread();
    let builder = &mut builder.max_blocking_threads(1).enable_all();
    let runtime = anyhow::result(builder.build())?;
    Ok(runtime.block_on(fut))
}
