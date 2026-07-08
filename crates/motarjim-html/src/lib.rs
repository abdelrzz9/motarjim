#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! HTML5 parser for the Motarjim compiler.
//!
//! This crate wraps Servo's [html5ever] — a browser-grade, spec-compliant
//! HTML parser — and converts its output into Motarjim's internal HTML AST.
//!
//! # Architecture
//!
//! The crate is organised into several modules:
//!
//! - [`ast`] — Motarjim-owned HTML AST types (independent of html5ever)
//! - [`span`] — Source location span types
//! - [`diagnostics`] — Structured parse errors and diagnostics
//! - `converter` — RcDom → Motarjim AST conversion (the only module
//!   that directly uses html5ever types)
//! - [`parser`] — The public parser API using html5ever internally
//!
//! # Why wrap html5ever instead of exposing it directly?
//!
//! 1. **Ownership**: Motarjim owns its AST. The parser backend can be
//!    replaced (e.g., with a streaming parser or LALRPOP parser) without
//!    affecting downstream crates.
//! 2. **Stability**: html5ever's types (RcDom, Handle, NodeData) are
//!    designed for browser-engine use. They expose reference counting,
//!    interior mutability, and tendril string types that are unidiomatic
//!    in a traditional compiler pipeline.
//! 3. **API surface**: A clean `parse(&str) -> Result<Document, ParseError>`
//!    is simpler and more maintainable than exposing the full RcDom API.
//!
//! # Parsing flow
//!
//! ```text
//! HTML source
//!     │
//!     ▼
//! html5ever (spec-compliant parser)
//!     │
//!     ▼
//! RcDom (reference-counted DOM tree)
//!     │
//!     ▼
//! converter (recursive walk + conversion)
//!     │
//!     ▼
//! Motarjim AST (tree-based, owned types)
//! ```
//!
//! # Ownership model
//!
//! The converter takes ownership of the RcDom and recursively walks its
//! tree, building Motarjim-owned nodes. Each Motarjim [`Node`] fully owns
//! its children (unlike RcDom's shared-reference model), which means the
//! RcDom can be dropped after conversion.
//!
//! This also means that the resulting Motarjim AST is:
//! - **Send + Sync** (no Rc/RefCell)
//! - **'static** (no borrowed references to the input)
//! - **Cloneable** cheaply or expensively depending on tree size
//!
//! # Future compatibility
//!
//! Only the `converter` module depends on html5ever. If the parser
//! backend were to be replaced (e.g., with a streaming parser, a
//! handwritten tokenizer, or an LALRPOP-based parser), only this module
//! would need to change. The rest of the crate and all downstream crates
//! would remain unaffected.

/// Motarjim-owned HTML AST types.
///
/// These types are completely independent of html5ever and represent the
/// compiler's internal view of an HTML document as a tree of nodes.
pub mod ast;

/// Source location span types.
pub mod span;

/// Structured parse errors and diagnostics.
pub mod diagnostics;

/// RcDom → Motarjim AST conversion.
///
/// This module is the only one that imports html5ever/markup5ever_rcdom
/// types. It is kept deliberately separate to isolate the parser backend.
mod converter;

/// The public HTML5 parser API.
pub mod parser;

// Re-export the most important types at the crate root for convenience.
pub use ast::{
    Attribute, CommentData, DoctypeData, Document, ElementData, Fragment, Node, NodeId,
    NodeIdGenerator, NodeKind, ProcessingInstructionData, TextData,
};
pub use diagnostics::{DiagnosticBag, ParseError, ParseErrorKind, Severity};
pub use parser::HtmlParser;
pub use span::{BytePos, SourceSpan};
