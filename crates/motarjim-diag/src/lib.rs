#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]

//! Diagnostic system for the Motarjim compiler.
//!
//! This crate provides a comprehensive diagnostic infrastructure by re-exporting
//! types from [`motarjim_errors`] and [`motarjim_span`]:
//!
//! - **Severity levels** — [`Severity`] distinguishes Error, Warning, Info, Hint, and Note.
//! - **Diagnostic codes** — [`DiagnosticCode`] pairs a numeric ID with a static message.
//! - **Diagnostic** — A single diagnostic with message, severity, code, source span,
//!   suggestions, and notes.
//! - **Source locations** — [`SourceLocation`] and [`SourceSpan`] track 1-based line/column
//!   positions with byte offsets.
//! - **Source files** — [`SourceFile`] holds path + content and provides line lookups and
//!   context snippets.
//! - **Diagnostic bag** — [`DiagnosticBag`] collects diagnostics during compilation.
//!
//! ## Feature flags
//!
//! - `color` — Enables `emitter::DiagnosticEmitter` for colored terminal output.
//! - `json` — Derives `serde::Serialize`/`Deserialize` on diagnostic types.

/// Predefined diagnostic codes for common compiler messages.
pub mod codes;
/// Colored terminal diagnostic emitter.
#[cfg(feature = "color")]
pub mod emitter;

pub use motarjim_errors::code::DiagnosticCode;
pub use motarjim_errors::diagnostic::{Diagnostic, DiagnosticBag};
pub use motarjim_errors::severity::Severity;
pub use motarjim_span::{SourceFile, SourceLocation, SourceSpan};
