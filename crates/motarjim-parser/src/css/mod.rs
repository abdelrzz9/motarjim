//! CSS parsing module powered by Lightning CSS.
//!
//! This module provides a complete CSS parsing pipeline that:
//! 1. Parses CSS source using Lightning CSS
//! 2. Converts the Lightning CSS AST into Motarjim's own AST types
//! 3. Validates the converted AST
//! 4. Returns structured diagnostics on failure
//!
//! The Lightning CSS dependency is fully encapsulated within this module.
//! No Lightning CSS types are exposed in the public API.

mod converter;
mod error;
mod parser;
#[cfg(test)]
mod tests;
mod validation;

pub use error::CssError;
pub use parser::parse_css;
pub use parser::CssParser;
