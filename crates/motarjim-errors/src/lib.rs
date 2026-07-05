#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Structured error types for the Motarjim compiler.

/// Registered diagnostic codes.
pub mod code;
/// Diagnostic and diagnostic bag types.
pub mod diagnostic;
/// Diagnostic severity levels.
pub mod severity;
