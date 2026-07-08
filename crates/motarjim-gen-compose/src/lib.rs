#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Jetpack Compose/Kotlin code generator for the Motarjim compiler.
//!
//! Maps [`IrTree`] nodes to Compose composable functions
//! using the [`CodeWriter`] for indented Kotlin output.

use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr};
use motarjim_ast::NodeId;
use motarjim_ast_html::EdgeValues;
use motarjim_formatter::CodeWriter;

/// The Kotlin/Jetpack Compose code generator implementation.
mod generator;
pub use generator::*;

#[cfg(test)]
mod tests;
