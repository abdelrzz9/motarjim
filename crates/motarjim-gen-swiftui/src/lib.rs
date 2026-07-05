#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! `SwiftUI` code generator for the Motarjim compiler.
//!
//! Maps [`IrTree`](motarjim_ast::ir::IrTree) nodes to `SwiftUI` View code
//! using the [`CodeWriter`](motarjim_formatter::CodeWriter) for indented Swift output.

use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr};
use motarjim_ast::NodeId;
use motarjim_ast_html::EdgeValues;
use motarjim_formatter::CodeWriter;

/// The Swift/SwiftUI code generator implementation.
mod generator;
pub use generator::*;

#[cfg(test)]
mod tests;
