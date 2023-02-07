use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

pub struct AutoCounter(Arc<AtomicI64>);

impl AutoCounter {
    pub fn new(v: Arc<AtomicI64>) -> Self {
        v.fetch_add(1, Ordering::AcqRel);
        Self(v)
    }
}

impl Drop for AutoCounter {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}
