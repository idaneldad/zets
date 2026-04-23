//! Cooperative cancellation token for parallel walks.
//!
//! When one walk finds an answer, the others should stop. This uses an
//! atomic flag checked at each step.
//!
//! Cheaper than tokio's CancellationToken for our tight walk loop.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Shared flag; walks check `is_cancelled()` each step and return early.
#[derive(Clone, Debug)]
pub struct CancelToken {
    inner: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self { inner: Arc::new(AtomicBool::new(false)) }
    }

    /// Signal all holders to stop. Idempotent.
    pub fn cancel(&self) {
        self.inner.store(true, Ordering::Release);
    }

    /// Returns true once any holder has called `cancel()`.
    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::Acquire)
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_token_not_cancelled() {
        let t = CancelToken::new();
        assert!(!t.is_cancelled());
    }

    #[test]
    fn cancel_flips_flag() {
        let t = CancelToken::new();
        t.cancel();
        assert!(t.is_cancelled());
    }

    #[test]
    fn cancel_propagates_through_clones() {
        let t = CancelToken::new();
        let t2 = t.clone();
        let t3 = t.clone();
        t2.cancel();
        assert!(t.is_cancelled());
        assert!(t3.is_cancelled());
    }

    #[test]
    fn cancel_is_idempotent() {
        let t = CancelToken::new();
        t.cancel();
        t.cancel();
        t.cancel();
        assert!(t.is_cancelled());
    }
}
