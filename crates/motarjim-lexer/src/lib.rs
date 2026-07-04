#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! HTML and CSS lexer/tokenizer for the Motarjim compiler.
//!
//! Provides zero-copy tokenization for HTML and CSS source text with
//! position tracking, unicode support, and error recovery.
//!
//! # Example
//!
//! ```rust
//! use motarjim_lexer::html::HtmlTokenizer;
//!
//! let mut tokenizer = HtmlTokenizer::new("<div>hello</div>");
//! let tokens = tokenizer.tokenize();
//! assert!(!tokens.is_empty());
//! ```

/// Character cursor for zero-copy tokenization with position tracking.
mod cursor;
/// Generic token type used by both HTML and CSS tokenizers.
mod token;

/// CSS tokenizer module with `CssTokenKind` and `CssTokenizer`.
pub mod css;
/// HTML tokenizer module with `HtmlTokenKind` and `HtmlTokenizer`.
pub mod html;

#[cfg(test)]
mod proptests;

pub use cursor::Cursor;
pub use token::Token;
