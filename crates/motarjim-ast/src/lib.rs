#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]

//! AST type definitions for the Motarjim compiler.
//!
//! This crate re-exports all AST types from domain-specific sub-crates:
//!
//! - [`motarjim_ast_html`] — HTML node types, computed styles, semantic roles
//! - [`motarjim_ast_css`] — CSS stylesheet and selector types
//! - [`motarjim_ast_ir`] — Intermediate representation types
//! - [`motarjim_span`] — Source location and span types

pub use motarjim_ast_css as css;
pub use motarjim_ast_html as html;
pub use motarjim_ast_ir as ir;

// Re-export all public types from sub-crates for convenience.
pub use motarjim_ast_css::{
    AtRule, CharsetRule, CssFunction, CssNumber, CssRule, CssStylesheet, CssUnit, CssValue,
    Declaration, FontFaceRule, ImportRule, Keyframe, KeyframesRule, MediaCondition, MediaQuery,
    MediaRule, NamespaceRule, PageRule, StyleRule, SupportsRule,
};
pub use motarjim_ast_css::{
    AttributeOperator, Combinator, PseudoClass, PseudoElement, Selector, SimpleSelector,
};

pub use motarjim_ast_html::{
    A11yViolation, AccessibilityInfo, Attribute, ComputedStyle, DisplayType, Document,
    DocumentTypeNode, Element, HtmlNode, NodeId, NodeType, SemanticDocument, SemanticRole,
    StyledDocument, StyledNode,
};
pub use motarjim_ast_html::{
    AlignContent, AlignItems, Background, Border, EdgeValues, FlexDirection, FlexWrap, FontWeight,
    JustifyContent, Overflow, PositionType, TextAlign,
};

pub use motarjim_ast_ir::{Breakpoint, LayoutConstraints, ResponsiveVariant};
pub use motarjim_ast_ir::{HintType, IrNode, IrTree, LayoutIr, SemanticIr, TargetHint, TargetIr};
