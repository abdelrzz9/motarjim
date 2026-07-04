#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Diagnostic system for the Motarjim compiler.
//!
//! This crate provides a comprehensive diagnostic infrastructure:
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
//! - `color` — Enables [`emitter::DiagnosticEmitter`] for colored terminal output.
//! - `json` — Derives `serde::Serialize` and `serde::Deserialize` on all diagnostic types.

/// Internal diagnostic bag implementation.
mod bag;
pub mod codes;
/// Internal diagnostic types.
mod diagnostic;
#[cfg(feature = "color")]
#[cfg_attr(feature = "color", doc = "Colored terminal diagnostic output (requires the `color` feature).")]
pub mod emitter;
/// Internal source span types.
mod span;

pub use bag::DiagnosticBag;
pub use diagnostic::{Diagnostic, DiagnosticCode, Severity};
pub use span::{SourceFile, SourceLocation, SourceSpan};
