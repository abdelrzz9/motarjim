#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! AST type definitions for the Motarjim compiler.
//!
//! This crate defines all the AST types used across the compiler pipeline,
//! including HTML nodes, CSS stylesheets, selectors, intermediate representation,
//! layout strategies, computed styles, and semantic roles.
//!
//! These types are pure data with no logic — they are shared by the parser,
//! CSS engine, IR builder, optimizer, and generators.

pub mod css;
mod html;
pub mod ir;
pub mod layout;
pub mod selector;
pub mod semantic;
pub mod style;

pub use css::{
    AtRule, CharsetRule, CssRule, CssStylesheet, Declaration, FontFaceRule, ImportRule, Keyframe,
    KeyframesRule, MediaCondition, MediaQuery, MediaRule, NamespaceRule, PageRule, StyleRule,
    SupportsRule,
};
pub use html::{
    Attribute, Document, DocumentTypeNode, Element, HtmlNode, NodeId, NodeType, SemanticDocument,
    StyledDocument, StyledNode,
};
pub use ir::{HintType, IrNode, IrTree, LayoutIr, SemanticIr, TargetHint, TargetIr};
pub use layout::{Breakpoint, LayoutConstraints, LayoutStrategy, ResponsiveVariant};
pub use selector::{
    AttributeOperator, Combinator, PseudoClass, PseudoElement, Selector, SimpleSelector,
};
pub use semantic::{A11yViolation, AccessibilityInfo, SemanticRole};
pub use style::{
    AlignContent, AlignItems, Background, Border, ComputedStyle, DisplayType, EdgeValues,
    FlexDirection, FlexWrap, FontWeight, JustifyContent, Overflow, PositionType, TextAlign,
};
