#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! CSS AST type definitions for the Motarjim compiler.
//!
//! This crate defines CSS stylesheet types (rules, declarations, media queries)
//! and selector types (simple selectors, combinators, pseudo-classes).

pub mod selector;
pub mod stylesheet;
pub mod value;

pub use selector::{
    AttributeOperator, Combinator, PseudoClass, PseudoElement, Selector, SimpleSelector,
};
pub use stylesheet::{
    AtRule, CharsetRule, CssRule, CssStylesheet, Declaration, FontFaceRule, ImportRule, Keyframe,
    KeyframesRule, MediaCondition, MediaQuery, MediaRule, NamespaceRule, PageRule, StyleRule,
    SupportsRule,
};
pub use value::{CssFunction, CssUnit, CssValue};
