#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]

//! Source span/location types for Motarjim.
//!
//! This crate provides the fundamental types used throughout the
//! Motarjim compiler to represent positions and ranges in source files.

mod source_file;
mod source_location;
mod source_span;

pub use source_file::SourceFile;
pub use source_location::SourceLocation;
pub use source_span::SourceSpan;
