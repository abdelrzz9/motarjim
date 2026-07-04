//! IR builder crate for the Motarjim compiler.
//!
//! Builds the platform-independent Intermediate Representation (IR) tree
//! from parsed HTML documents and computed CSS styles. The IR tree is the
//! central data structure consumed by all code generators.
//!
//! The builder orchestrates four inference passes:
//! - **Semantic inference** ([`SemanticAnalyzer`]): Maps HTML tag names and
//!   attributes to [`SemanticIr`] roles.
//! - **Layout inference** ([`LayoutInferrer`]): Converts CSS computed styles into [`LayoutIr`] strategies.
//! - **Responsive inference** ([`ResponsiveInferrer`]): Extracts responsive breakpoint information.
//! - **Accessibility inference** ([`AccessibilityInferrer`]): Extracts ARIA attributes and implicit roles.

#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

use std::collections::HashMap;

use motarjim_ast::ir::{HintType, IrNode, IrTree, LayoutIr, SemanticIr, TargetHint, TargetIr};
use motarjim_ast::layout::ResponsiveVariant;
use motarjim_ast::semantic::AccessibilityInfo;
use motarjim_ast::style::{ComputedStyle, DisplayType, FlexDirection, Overflow, PositionType};
use motarjim_ast::{Document, Element, HtmlNode, NodeId, NodeType};
use motarjim_diag::DiagnosticBag;
use smol_str::SmolStr;

mod builder;
pub use builder::*;
mod semantic;
pub use semantic::*;
mod layout;
pub use layout::*;
mod responsive;
pub use responsive::*;
mod accessibility;
pub use accessibility::*;
#[cfg(test)]
mod tests;
