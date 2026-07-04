#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! CSS selector parsing and matching engine for the Motarjim compiler.
//!
//! Parses CSS selector strings into structured representations and provides
//! matching against HTML elements. Supports combinators (descendant, child,
//! sibling), pseudo-classes, pseudo-elements, and attribute selectors.
//!
//! # Example
//!
//! ```rust
//! use motarjim_selectors::{Selector, parse_selector};
//!
//! let selector = parse_selector("div.container").unwrap();
//! assert!(matches!(selector, Selector::Compound(_)));
//! ```

/// CSS selector types (Selector, SimpleSelector, etc.).
mod types;
/// Display implementations for selector types.
mod display;
/// Selector parse error type.
mod error;
/// Selector parsing logic.
mod parser;
/// Specificity and matched selector.
mod specificity;

#[cfg(test)]
mod proptests;

pub use types::*;
pub use error::SelectorParseError;
pub use parser::parse_selector;
pub use specificity::{MatchedSelector, Specificity};
