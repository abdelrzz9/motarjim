#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Intermediate representation (IR) AST types for the Motarjim compiler.
//!
//! This crate defines the IR node types (semantic, layout, and target hints)
//! and layout strategy types that drive code generation.

pub mod ir;
pub mod layout;

pub use ir::{HintType, IrNode, IrTree, LayoutIr, SemanticIr, TargetHint, TargetIr};
pub use layout::{Breakpoint, ResponsiveVariant};
