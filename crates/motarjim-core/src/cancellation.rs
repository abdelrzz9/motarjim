use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A token that signals cancellation of a long-running compilation.
///
/// Cloning shares the underlying cancellation state — cancelling any clone
/// cancels all of them.
#[derive(Clone)]
pub struct CancelToken {
    /// Shared atomic flag; `true` once [`cancel`](Self::cancel) is called.
    cancelled: Arc<AtomicBool>,
}

impl CancelToken {
    /// Creates a new cancellation token that is **not** cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Requests cancellation. All clones of this token will now report
    /// [`is_cancelled`](Self::is_cancelled) as `true` and [`check`](Self::check)
    /// will return `Err`.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns `true` if [`cancel`](Self::cancel) has been called on any clone.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Returns `Ok(())` if not cancelled, or `Err(Cancelled)` if cancelled.
    ///
    /// # Errors
    ///
    /// Returns [`Cancelled`] when [`cancel`](Self::cancel) has been called.
    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled {
                message: String::from("compilation cancelled"),
            })
        } else {
            Ok(())
        }
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned when a cancelled operation is detected.
#[derive(Debug, Clone)]
pub struct Cancelled {
    /// Human-readable reason for the cancellation.
    pub message: String,
}

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Cancelled {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_starts_not_cancelled() {
        let token = CancelToken::new();
        assert!(!token.is_cancelled());
        assert!(token.check().is_ok());
    }

    #[test]
    fn cancel_sets_flag() {
        let token = CancelToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn check_returns_ok_before_cancel_err_after() {
        let token = CancelToken::new();
        assert!(token.check().is_ok());
        token.cancel();
        let err = token.check().unwrap_err();
        assert_eq!(err.message, "compilation cancelled");
    }

    #[test]
    fn clone_shares_state() {
        let token = CancelToken::new();
        let cloned = token.clone();
        token.cancel();
        assert!(cloned.is_cancelled());
        assert!(cloned.check().is_err());
    }

    #[test]
    fn multiple_cancels_do_not_error() {
        let token = CancelToken::new();
        token.cancel();
        token.cancel();
        token.cancel();
        assert!(token.is_cancelled());
    }
}
