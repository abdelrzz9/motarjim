#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]

//! Source span/location types for Motarjim.
//!
//! This crate provides the fundamental types used throughout the
//! Motarjim compiler to represent positions and ranges in source files.

/// Source file representation (path, content, line starts).
mod source_file;
/// 1-based line/column position within a source file.
mod source_location;
/// A range within a source file, defined by start and end [`SourceLocation`]s.
mod source_span;

pub use source_file::SourceFile;
pub use source_location::SourceLocation;
pub use source_span::SourceSpan;
