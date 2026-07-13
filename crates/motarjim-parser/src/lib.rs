#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! CSS parser for the Motarjim compiler.
//!
//! This crate provides:
//!
//! - **CSS parser** — Powered by Lightning CSS internally, produces
//!   Motarjim's own CSS AST types. Lightning CSS is an internal dependency;
//!   no Lightning CSS types are exposed in the public API.
//!
//! # CSS Parsing
//!
//! The CSS parser handles:
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

pub use css::CssParser;
