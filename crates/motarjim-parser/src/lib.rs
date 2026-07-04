#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Recursive descent HTML parser for the Motarjim compiler.
//!
//! This crate provides an [`HtmlParser`] and a [`CssParser`] that consume
//! tokens from `motarjim_lexer` and build AST nodes from `motarjim_ast`.
//!
//! # HTML Parsing
//!
//! The HTML parser handles:
//! - Elements with attributes (id, classes extracted automatically)
//! - Nested elements
//! - Text nodes
//! - HTML comments
//! - Self-closing and void elements
//! - DOCTYPE declarations
//! - Error recovery with diagnostics
//!
//! # CSS Parsing
//!
//! The CSS parser handles:
//! - Style rules with selectors and declarations
//! - Simple selectors (type, class, id, attribute selectors)
//! - Compound and grouped selectors
//! - At-rules (`@media`, `@import`, etc.)
//! - Error recovery with diagnostics

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]

/// CSS parser implementation.
mod css;
/// HTML parser implementation.
mod html;
/// Shared utility functions.
mod util;

#[cfg(test)]
/// Property-based tests.
mod proptests;

pub use css::CssParser;
pub use html::HtmlParser;
pub use util::{extract_tag_name, is_void_element, parse_attributes_from_str};
