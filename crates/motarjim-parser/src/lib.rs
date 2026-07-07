#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! HTML and CSS parsers for the Motarjim compiler.
//!
//! This crate provides:
//!
//! - **HTML parser** — Recursive descent parser that consumes tokens from
//!   `motarjim_lexer` and builds AST nodes from `motarjim_ast`.
//! - **CSS parser** — Powered by Lightning CSS internally, produces
//!   Motarjim's own CSS AST types. Lightning CSS is an internal dependency;
//!   no Lightning CSS types are exposed in the public API.
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
//! The CSS parser (available via the [`css`] module) handles:
//! - Style rules with selectors and declarations
//! - Simple selectors (type, class, id, attribute, pseudo-class, pseudo-element)
//! - Compound and grouped selectors with combinators
//! - At-rules (`@media`, `@import`, `@keyframes`, `@font-face`, `@supports`, etc.)
//! - CSS variables, gradients, calc(), transforms, animations
//! - Comprehensive error diagnostics
//! - Source span preservation

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]

/// CSS parser implementation (powered by Lightning CSS).
pub mod css;
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
