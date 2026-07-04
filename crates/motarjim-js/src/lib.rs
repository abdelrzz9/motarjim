#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! JavaScript lexer, parser, AST, semantic analysis, and transforms for the
//! Motarjim compiler.
//!
//! This crate is Motarjim's JavaScript front end, following a modular
//! compiler pipeline design similar to SWC, Biome, and oxc.
//!
//! # Architecture
//!
//! ```text
//! Lexer → Parser → AST → Semantic Analysis → Transforms
//! ```
//!
//! Each stage is in its own module with a clear responsibility:
//!
//! - `token` — Token kinds, values, and keyword lookup
//! - `lexer` — Character-level tokenization
//! - `ast` — AST node types (expr, stmt, lit, pat, program)
//! - `parser` — Recursive-descent parser with error recovery
//! - `semantic` — Scope tracking, variable resolution, semantic checks
//! - `visitor` — Visitor, VisitorMut, and Fold traits for tree traversal
//! - `transform` — AST-to-AST transformation passes
//! - `events` — DOM event binding extraction
//! - `diagnostics` — Diagnostic codes and types

mod ast;
mod diagnostics;
mod events;
mod lexer;
mod parser;
mod semantic;
pub mod token;
mod transform;
pub mod visitor;

pub use ast::{
    ArrayElement, ArrayLit, ArrowBody, ArrowFunction, AssignExpr, AssignOp, BinaryExpr, BinaryOp,
    BlockStmt, BoolLit, CallExpr, CatchClause, ClassBody, ClassDecl, ClassMember, ClassMethod,
    ClassProperty, CondExpr, DoWhileStmt, ExportDefaultDecl, ExportDefaultKind, ExportNamedDecl,
    ExportSpecifier, ExprStmt, Expression, ForInStmt, ForInit, ForOfStmt, ForStmt, FunctionDecl,
    FunctionExpr, IfStmt, ImportDecl, ImportSpecifier, LogicalExpr, LogicalOp, MemberExpr,
    MemberProp, MethodKind, NewExpr, NumberLit, ObjectLit, ObjectPat, ObjectPatProp, ObjectProp,
    Param, Pattern, Program, PropKey, ReturnStmt, SequenceExpr, Statement, StringLit,
    SwitchCase, SwitchStmt, TemplateLiteral, ThrowStmt, TryStmt, UnaryExpr, UnaryOp, VarDecl,
    VarDeclarator, VarKind, WhileStmt, YieldExpr, SourceType,
};
pub use diagnostics::{JsDiagnostic, JsDiagnosticCode};
pub use events::{find_dom_event_bindings, DomEventBinding};
pub use lexer::JsLexer;
pub use parser::JsParser;
pub use semantic::SemanticAnalyzer;
pub use token::{keyword_from_str, is_reserved_word, JsToken, JsTokenKind, TokenValue};
pub use transform::passes::TemplateLiteralToConcat;
pub use transform::{run_transforms, Transform};
pub use visitor::{walk_expression, walk_program, walk_statement, Fold, Visitor, VisitorMut};
