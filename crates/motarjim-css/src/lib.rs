#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! CSS engine for the Motarjim compiler.
//!
//! Provides cascade resolution, computed style calculation, CSS value parsing,
//! and parallel selector matching.
//!
//! # Overview
//!
//! The engine is built around three core types:
//!
//! - [`Cascade`] — Collects and sorts declarations by specificity and source order.
//! - [`ComputedValues`] — Holds the final resolved [`ComputedStyle`] for an element.
//! - [`StyleResolver`] — Accepts stylesheets, matches selectors, computes styles.
//!
//! Selector matching against DOM elements is parallelised via `rayon` when
//! [`StyleResolver::resolve_parallel`] is used.

use std::collections::HashMap;

use motarjim_ast::css::{CssRule, CssStylesheet, Declaration, StyleRule};
use motarjim_ast_css::{AttributeOperator, Selector, SimpleSelector};
use motarjim_ast_html::{
    AlignContent, AlignItems, Background, Border, ComputedStyle, DisplayType, EdgeValues,
    FlexDirection, FlexWrap, FontWeight, JustifyContent, Overflow, PositionType, TextAlign,
};
use motarjim_ast::Element;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// CSS value types
// ---------------------------------------------------------------------------

/// Typed CSS value parsing (colors, lengths, spacing, etc.).
mod value;
pub use value::*;
/// Cascade resolution: sorting and merging matched declarations by specificity.
mod cascade;
pub use cascade::*;
/// Known CSS property definitions and lookups.
mod properties;
pub use properties::*;
/// Selector-to-node matching against a styled document.
mod matching;
/// Resolves a stylesheet and document into computed styles per node.
mod resolver;
pub use resolver::*;
#[cfg(test)]
mod tests;
