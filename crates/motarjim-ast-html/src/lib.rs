#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! HTML AST type definitions for the Motarjim compiler.
//!
//! This crate defines HTML node types, computed styles, and semantic
//! accessibility types.  These are pure data types shared by the parser,
//! CSS engine, IR builder, and generators.

pub mod node;
pub mod semantic;
pub mod style;
pub mod grid;

pub use node::{
    Attribute, Document, DocumentTypeNode, Element, HtmlNode, NodeId, NodeType, SemanticDocument,
    StyledDocument, StyledNode,
};
pub use semantic::{A11yViolation, AccessibilityInfo, SemanticRole};
pub use style::{
    AlignContent, AlignItems, Background, Border, ComputedStyle, DisplayType, EdgeValues,
    FlexDirection, FlexWrap, FontWeight, JustifyContent, Overflow, PositionType, TextAlign,
};
