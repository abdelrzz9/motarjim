#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! JavaScript lexer, parser, AST, semantic analysis, and transforms for the
//! Motarjim compiler.
//!
//! This crate is Motarjim's JavaScript front end, following the same
//! zero-copy tokenize-then-parse design as the HTML and CSS front ends in
//! `motarjim-lexer` and `motarjim-parser`. It is intentionally standalone
//! (lexer, AST, parser, semantic analysis, and transforms all live here,
//! unlike HTML/CSS which are split across several crates) since JavaScript
//! support is new; splitting it further is a natural follow-up once it
//! grows past the size limits documented in `ARCHITECTURE_REVIEW.md`.
//!
//! # Supported syntax
//!
//! - `var`/`let`/`const` declarations
//! - Named and anonymous function declarations/expressions
//! - Arrow functions, including concise (expression) and block bodies
//! - Template literals, including nested interpolations and nested templates
//! - `if`/`else`, `for` (C-style, `for...of`, `for...in` — declaration form
//!   only), `while`, `do...while`
//! - `import`/`export` (default, named, namespace)
//! - The common expression grammar: binary/logical/unary/assignment
//!   operators, member access, calls, `new`, array/object literals,
//!   ternaries, comma sequences
//!
//! # Known gaps
//!
//! - No automatic semicolon insertion (semicolons are simply optional)
//! - No destructuring, spread/rest, classes, generators, or `async`/`await`
//! - No regular expression literals
//! - Bare (non-declaration) `for (x of xs)` / `for (x in xs)` loops
//!
//! See [`parser`] module docs for the full list.
//!
//! # Example
//!
//! ```rust
//! use motarjim_js::{find_dom_event_bindings, JsParser, SemanticAnalyzer};
//!
//! let source = r#"
//!     const button = document.getElementById("go");
//!     button.addEventListener("click", () => {
//!         console.log(`Clicked ${count} times`);
//!     });
//! "#;
//!
//! let mut parser = JsParser::new(source);
//! let program = parser.parse().expect("valid syntax");
//!
//! let diagnostics = SemanticAnalyzer::new().analyze(&program);
//! assert!(diagnostics.iter().any(|d| d.message().contains("count")));
//!
//! let bindings = find_dom_event_bindings(&program);
//! assert_eq!(bindings[0].event_name, "click");
//! ```

mod ast;
mod events;
mod lexer;
mod parser;
mod semantic;
mod token;
mod transform;
mod visitor;

pub use ast::{
    ArrayLit, ArrowBody, ArrowFunction, AssignExpr, AssignOp, BinaryExpr, BinaryOp, BlockStmt,
    BoolLit, CallExpr, CondExpr, DoWhileStmt, ExportDefaultDecl, ExportNamedDecl, ExprStmt,
    Expression, ForInStmt, ForInit, ForOfStmt, ForStmt, FunctionDecl, FunctionExpr, Ident, IfStmt,
    ImportDecl, ImportSpecifier, LogicalExpr, LogicalOp, MemberExpr, MemberProp, NewExpr,
    NumberLit, ObjectLit, ObjectProp, Param, Program, PropKey, ReturnStmt, SequenceExpr, Statement,
    StringLit, TemplateLiteral, UnaryExpr, UnaryOp, VarDecl, VarDeclarator, VarKind, WhileStmt,
};
pub use events::{find_dom_event_bindings, DomEventBinding};
pub use lexer::JsLexer;
pub use parser::JsParser;
pub use semantic::SemanticAnalyzer;
pub use token::{keyword_from_str, JsTokenKind};
pub use transform::{run_transforms, TemplateLiteralToConcat, Transform};
pub use visitor::{walk_expression, walk_program, walk_statement, Visitor};
