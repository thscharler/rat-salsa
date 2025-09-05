use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Liveness check for background tasks.
/// Used by both the thread-pool and by the tokio async runner.
#[derive(Debug, Default, Clone)]
pub struct Liveness(Arc<AtomicBool>);

impl Liveness {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn is_alive(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn born(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn dead(&self) {
        self.0.store(false, Ordering::Release);
    }
}
