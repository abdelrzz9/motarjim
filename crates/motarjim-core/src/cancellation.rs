//! Cancellation token support for long-running operations.
//!
//! Provides [`CancelToken`] and [`Cancelled`] for cooperative cancellation
//! throughout the compilation pipeline.
//!
//! This module re-exports from [`motarjim_session`] so that consumers using
//! `motarjim_core::cancellation::CancelToken` continue to work.

pub use motarjim_session::{CancelToken, Cancelled};

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
