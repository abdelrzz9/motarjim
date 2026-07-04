#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Optimization pass manager and passes for the Motarjim compiler IR.
//!
//! This crate provides a [`PassManager`] that registers and runs optimization
//! passes on an [`IrTree`]. Each pass implements the [`Pass`] trait and
//! reports results via [`PassResult`].

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use motarjim_ast::ir::{IrNode, IrTree, SemanticIr, TargetIr};
use motarjim_ast_html::{Border, ComputedStyle, DisplayType, EdgeValues};
use motarjim_ast::NodeId;
use motarjim_diag::Diagnostic;
use smallvec::SmallVec;

/// The [`PassManager`] and pass scheduling/registration types.
mod pass_manager;
pub use pass_manager::*;
/// Shared helper functions used by multiple passes.
mod helpers;
/// Individual optimization pass implementations.
mod passes;
pub use passes::*;

#[cfg(test)]
mod tests;
