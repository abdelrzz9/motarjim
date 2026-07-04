#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Jetpack Compose/Kotlin code generator for the Motarjim compiler.
//!
//! Maps [`IrTree`](motarjim_ast::ir::IrTree) nodes to Compose composable functions
//! using the [`CodeWriter`](motarjim_formatter::CodeWriter) for indented Kotlin output.

use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr};
use motarjim_ast::style::EdgeValues;
use motarjim_ast::NodeId;
use motarjim_formatter::CodeWriter;

/// The Kotlin/Jetpack Compose code generator implementation.
mod generator;
pub use generator::*;

#[cfg(test)]
mod tests;
