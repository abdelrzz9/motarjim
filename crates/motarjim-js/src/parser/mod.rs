//! Recursive-descent parser with Pratt expression parsing and error recovery.

use motarjim_span::SourceSpan;

use crate::ast::program::{Program, SourceType};
use crate::ast::stmt::Statement;
use crate::lexer::JsLexer;
use crate::token::{JsToken, JsTokenKind};

mod expr;
mod stmt;

pub use expr::*;
pub use stmt::*;

use crate::diagnostics::JsDiagnostic;

#[derive(Debug)]
pub struct JsParser {
    tokens: Vec<JsToken>,
    pos: usize,
    errors: Vec<JsDiagnostic>,
    source: String,
}

impl JsParser {
    pub fn new(source: &str) -> Self {
        let mut lexer = JsLexer::new(source);
        let tokens = lexer.tokenize();
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            source: source.to_string(),
        }
    }

    pub fn parse(&mut self) -> Result<Program, Vec<JsDiagnostic>> {
        self.parse_with_type(SourceType::Script)
    }

    pub fn parse_module(&mut self) -> Result<Program, Vec<JsDiagnostic>> {
        self.parse_with_type(SourceType::Module)
    }

    fn parse_with_type(&mut self, source_type: SourceType) -> Result<Program, Vec<JsDiagnostic>> {
        let start = self.cur().span;
        let mut body = Vec::new();
        while !self.at(JsTokenKind::Eof) {
            body.push(self.parse_statement());
        }
        let span = self.span_from(start);
        if self.errors.is_empty() {
            Ok(Program {
                body,
                span,
                source_type,
            })
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    pub fn take_errors(&mut self) -> Vec<JsDiagnostic> {
        std::mem::take(&mut self.errors)
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    // ---- token stream helpers -------------------------------------------

    fn cur(&self) -> &JsToken {
        // Use get() instead of direct indexing to avoid panic on out-of-bounds.
        // tokenize() always produces at least one token (Eof).
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens
                .last()
                .expect("token stream should never be empty")
        })
    }

    fn kind(&self) -> JsTokenKind {
        self.cur().kind
    }

    fn at(&self, kind: JsTokenKind) -> bool {
        self.kind() == kind
    }

    fn at_any(&self, kinds: &[JsTokenKind]) -> bool {
        kinds.contains(&self.kind())
    }

    fn peek_kind_at(&self, offset: usize) -> JsTokenKind {
        self.tokens
            .get(self.pos + offset)
            .map_or(JsTokenKind::Eof, |t| t.kind)
    }

    fn advance(&mut self) -> JsToken {
        let token = self.cur().clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, kind: JsTokenKind, what: &str) -> JsToken {
        if self.at(kind) {
            self.advance()
        } else {
            self.error(format!("expected '{what}'"));
            self.cur().clone()
        }
    }

    fn expect_any(&mut self, kinds: &[JsTokenKind], what: &str) -> JsToken {
        if self.at_any(kinds) {
            self.advance()
        } else {
            self.error(format!("expected {what}"));
            self.cur().clone()
        }
    }

    fn eat(&mut self, kind: JsTokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_semicolon(&mut self) {
        if self.at(JsTokenKind::Semicolon) {
            self.advance();
        }
    }

    fn span_from(&self, start: SourceSpan) -> SourceSpan {
        let end = self.tokens[self.pos.saturating_sub(1)].span.end;
        SourceSpan {
            start: start.start,
            end,
        }
    }

    fn error(&mut self, message: impl Into<String>) {
        let msg = message.into();
        self.errors
            .push(JsDiagnostic::parse_error(msg, self.cur().span));
    }

    fn error_at(&mut self, span: SourceSpan, message: impl Into<String>) {
        let msg = message.into();
        self.errors.push(JsDiagnostic::parse_error(msg, span));
    }

    fn advance_if_semicolon(&mut self) {
        if self.at(JsTokenKind::Semicolon) {
            self.advance();
        }
    }
}
