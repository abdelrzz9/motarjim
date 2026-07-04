//! Recursive-descent parser for the supported JavaScript subset.
//!
//! The parser fully tokenizes its input up front (mirroring
//! `motarjim-parser`'s HTML and CSS parsers), then walks the token stream
//! with a hand-written recursive-descent parser for statements and a
//! precedence-climbing (Pratt) parser for expressions.
//!
//! # Error recovery
//!
//! Every parse function that can fail is guaranteed to still make forward
//! progress: an unrecognized token is reported as a diagnostic and skipped
//! rather than aborting the whole parse, so a single syntax error does not
//! prevent the rest of the file from being analyzed.
//!
//! # Known limitations
//!
//! - No automatic semicolon insertion: semicolons are simply optional.
//! - `for (x of xs)` / `for (x in xs)` without a `let`/`const`/`var` binding
//!   are not supported; use `for (const x of xs)` instead.
//! - No destructuring, spread/rest parameters, classes, generators, or
//!   `async`/`await`.
//! - Regular expression literals are not tokenized.

use motarjim_diag::{codes, Diagnostic, DiagnosticBag, Severity};
use motarjim_lexer::Token;

use crate::ast::{
    ArrayLit, ArrowBody, ArrowFunction, AssignExpr, AssignOp, BinaryExpr, BinaryOp, BlockStmt,
    BoolLit, CallExpr, CondExpr, DoWhileStmt, ExportDefaultDecl, ExportNamedDecl, ExprStmt,
    Expression, ForInStmt, ForInit, ForOfStmt, ForStmt, FunctionDecl, FunctionExpr, Ident, IfStmt,
    ImportDecl, ImportSpecifier, LogicalExpr, LogicalOp, MemberExpr, MemberProp, NewExpr,
    NumberLit, ObjectLit, ObjectProp, Param, Program, PropKey, ReturnStmt, SequenceExpr, Statement,
    StringLit, TemplateLiteral, UnaryExpr, UnaryOp, VarDecl, VarDeclarator, VarKind, WhileStmt,
};
use crate::lexer::JsLexer;
use crate::token::JsTokenKind;

/// Parses JavaScript source text into a [`Program`].
///
/// # Example
///
/// ```rust
/// use motarjim_js::JsParser;
///
/// let mut parser = JsParser::new("const greet = name => `Hi, ${name}!`;");
/// let program = parser.parse();
/// assert!(program.is_ok());
/// ```
#[derive(Debug)]
pub struct JsParser {
    /// The fully tokenized input.
    tokens: Vec<Token<JsTokenKind>>,
    /// The index of the current token in `tokens`.
    pos: usize,
    /// Diagnostics collected while parsing.
    diagnostics: DiagnosticBag,
}

impl JsParser {
    /// Creates a new parser over the given source text.
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut lexer = JsLexer::new(source);
        let tokens = lexer.tokenize();
        Self {
            tokens,
            pos: 0,
            diagnostics: DiagnosticBag::new(),
        }
    }

    /// Parses the source as a full program.
    ///
    /// # Errors
    ///
    /// Returns the collected diagnostics if any parse error occurred.
    pub fn parse(&mut self) -> Result<Program, Vec<Diagnostic>> {
        let start = self.cur().span;
        let mut body = Vec::new();
        while !self.at(JsTokenKind::Eof) {
            body.push(self.parse_statement());
        }
        let span = self.span_from(start);
        if self.diagnostics.has_errors() {
            Err(self.diagnostics.clone().into_diagnostics())
        } else {
            Ok(Program { body, span })
        }
    }

    /// Returns diagnostics collected so far, leaving the bag empty.
    pub fn take_diagnostics(&mut self) -> DiagnosticBag {
        std::mem::take(&mut self.diagnostics)
    }

    // ---- token stream helpers -------------------------------------------

    /// Returns the current token. Always valid: the token stream always ends
    /// with `Eof`, and the cursor never advances past it.
    fn cur(&self) -> &Token<JsTokenKind> {
        &self.tokens[self.pos]
    }

    /// Returns the current token's kind.
    fn kind(&self) -> JsTokenKind {
        self.cur().kind
    }

    /// Returns `true` if the current token has the given kind.
    fn at(&self, kind: JsTokenKind) -> bool {
        self.kind() == kind
    }

    /// Returns the kind of the token `offset` positions ahead, or `Eof` if
    /// that would run past the end of the stream.
    fn peek_kind_at(&self, offset: usize) -> JsTokenKind {
        self.tokens
            .get(self.pos + offset)
            .map_or(JsTokenKind::Eof, |t| t.kind)
    }

    /// Consumes and returns the current token, advancing the cursor (except
    /// at end of input, where the cursor stays parked on `Eof`).
    fn advance(&mut self) -> Token<JsTokenKind> {
        let token = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    /// Combines `start`'s beginning with the end of the most recently
    /// consumed token, producing the span of everything parsed since `start`
    /// was captured.
    fn span_from(&self, start: motarjim_diag::SourceSpan) -> motarjim_diag::SourceSpan {
        let end = self.tokens[self.pos.saturating_sub(1)].span.end;
        motarjim_diag::SourceSpan {
            start: start.start,
            end,
        }
    }

    /// Records an error diagnostic anchored at the current token.
    fn error_here(&mut self, code: motarjim_diag::DiagnosticCode, message: impl Into<String>) {
        let diag = Diagnostic::new(Severity::Error, code, message).with_span(self.cur().span);
        self.diagnostics.push(diag);
    }

    /// Consumes the current token if it matches `kind`; otherwise reports an
    /// "expected `what`" diagnostic without consuming anything, leaving the
    /// mismatched token for the caller's recovery strategy.
    fn expect(&mut self, kind: JsTokenKind, what: &str) -> Token<JsTokenKind> {
        if self.at(kind) {
            self.advance()
        } else {
            self.error_here(codes::JS_UNEXPECTED_TOKEN, format!("expected '{what}'"));
            self.cur().clone()
        }
    }

    /// Consumes a trailing `;` if present. Semicolons are always optional in
    /// this parser (see module docs on automatic semicolon insertion).
    fn eat_semicolon(&mut self) {
        if self.at(JsTokenKind::Semicolon) {
            self.advance();
        }
    }

    /// Parses a binding identifier (a variable, parameter, or function name).
    fn parse_binding_ident(&mut self) -> String {
        if self.at(JsTokenKind::Identifier) {
            self.advance().raw
        } else {
            self.error_here(codes::JS_UNEXPECTED_TOKEN, "expected an identifier");
            String::new()
        }
    }

    // ---- statements -------------------------------------------------------

    /// Parses a single statement. Always advances at least one token, even
    /// on error, so callers can loop safely.
    fn parse_statement(&mut self) -> Statement {
        match self.kind() {
            JsTokenKind::Var | JsTokenKind::Let | JsTokenKind::Const => self.parse_var_decl_stmt(),
            JsTokenKind::Function => self.parse_function_decl(),
            JsTokenKind::Return => self.parse_return(),
            JsTokenKind::If => self.parse_if(),
            JsTokenKind::For => self.parse_for(),
            JsTokenKind::While => self.parse_while(),
            JsTokenKind::Do => self.parse_do_while(),
            JsTokenKind::Break => {
                let span = self.advance().span;
                self.eat_semicolon();
                Statement::Break(span)
            }
            JsTokenKind::Continue => {
                let span = self.advance().span;
                self.eat_semicolon();
                Statement::Continue(span)
            }
            JsTokenKind::LBrace => Statement::Block(self.parse_block()),
            JsTokenKind::Semicolon => Statement::Empty(self.advance().span),
            JsTokenKind::Import => self.parse_import(),
            JsTokenKind::Export => self.parse_export(),
            _ => self.parse_expr_stmt(),
        }
    }

    /// Parses a `{ ... }` block of statements.
    fn parse_block(&mut self) -> BlockStmt {
        let start = self.cur().span;
        self.expect(JsTokenKind::LBrace, "{");
        let mut body = Vec::new();
        while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
            body.push(self.parse_statement());
        }
        self.expect(JsTokenKind::RBrace, "}");
        let span = self.span_from(start);
        BlockStmt { body, span }
    }

    /// Parses a `var`/`let`/`const` declaration (without the trailing `;`).
    fn parse_var_decl(&mut self) -> VarDecl {
        let start = self.cur().span;
        let kind = match self.advance().kind {
            JsTokenKind::Var => VarKind::Var,
            JsTokenKind::Let => VarKind::Let,
            _ => VarKind::Const,
        };
        let mut declarators = Vec::new();
        loop {
            let decl_start = self.cur().span;
            let name = self.parse_binding_ident();
            let init = if self.at(JsTokenKind::Assign) {
                self.advance();
                Some(self.parse_assignment_expr())
            } else {
                None
            };
            let span = self.span_from(decl_start);
            declarators.push(VarDeclarator { name, init, span });
            if self.at(JsTokenKind::Comma) {
                self.advance();
                continue;
            }
            break;
        }
        let span = self.span_from(start);
        VarDecl {
            kind,
            declarators,
            span,
        }
    }

    /// Parses a `var`/`let`/`const` declaration statement.
    fn parse_var_decl_stmt(&mut self) -> Statement {
        let decl = self.parse_var_decl();
        self.eat_semicolon();
        Statement::VarDecl(decl)
    }

    /// Parses a named function declaration.
    fn parse_function_decl(&mut self) -> Statement {
        let start = self.advance().span; // 'function'
        let name = self.parse_binding_ident();
        let params = self.parse_params();
        let body = self.parse_block();
        let span = self.span_from(start);
        Statement::FunctionDecl(FunctionDecl {
            name,
            params,
            body,
            span,
        })
    }

    /// Parses a parenthesized, comma-separated parameter list.
    fn parse_params(&mut self) -> Vec<Param> {
        self.expect(JsTokenKind::LParen, "(");
        let mut params = Vec::new();
        if !self.at(JsTokenKind::RParen) {
            loop {
                let start = self.cur().span;
                let name = self.parse_binding_ident();
                let default = if self.at(JsTokenKind::Assign) {
                    self.advance();
                    Some(self.parse_assignment_expr())
                } else {
                    None
                };
                let span = self.span_from(start);
                params.push(Param {
                    name,
                    default,
                    span,
                });
                if self.at(JsTokenKind::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        self.expect(JsTokenKind::RParen, ")");
        params
    }

    /// Parses a `return` statement.
    fn parse_return(&mut self) -> Statement {
        let start = self.advance().span; // 'return'
        let argument = if matches!(
            self.kind(),
            JsTokenKind::Semicolon | JsTokenKind::RBrace | JsTokenKind::Eof
        ) {
            None
        } else {
            Some(self.parse_expression())
        };
        self.eat_semicolon();
        let span = self.span_from(start);
        Statement::Return(ReturnStmt { argument, span })
    }

    /// Parses an `if`/`else` statement.
    fn parse_if(&mut self) -> Statement {
        let start = self.advance().span; // 'if'
        self.expect(JsTokenKind::LParen, "(");
        let test = self.parse_expression();
        self.expect(JsTokenKind::RParen, ")");
        let consequent = Box::new(self.parse_statement());
        let alternate = if self.at(JsTokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_statement()))
        } else {
            None
        };
        let span = self.span_from(start);
        Statement::If(IfStmt {
            test,
            consequent,
            alternate,
            span,
        })
    }

    /// Parses a `while` loop.
    fn parse_while(&mut self) -> Statement {
        let start = self.advance().span; // 'while'
        self.expect(JsTokenKind::LParen, "(");
        let test = self.parse_expression();
        self.expect(JsTokenKind::RParen, ")");
        let body = Box::new(self.parse_statement());
        let span = self.span_from(start);
        Statement::While(WhileStmt { test, body, span })
    }

    /// Parses a `do { ... } while (test);` loop.
    fn parse_do_while(&mut self) -> Statement {
        let start = self.advance().span; // 'do'
        let body = Box::new(self.parse_statement());
        self.expect(JsTokenKind::While, "while");
        self.expect(JsTokenKind::LParen, "(");
        let test = self.parse_expression();
        self.expect(JsTokenKind::RParen, ")");
        self.eat_semicolon();
        let span = self.span_from(start);
        Statement::DoWhile(DoWhileStmt { body, test, span })
    }

    /// Parses a `for` loop: C-style, `for...of`, or `for...in`. Bare
    /// (non-declaration) `for...of`/`for...in` loops are not supported; see
    /// the module docs.
    fn parse_for(&mut self) -> Statement {
        let start = self.advance().span; // 'for'
        self.expect(JsTokenKind::LParen, "(");

        let decl_kind = match self.kind() {
            JsTokenKind::Var => Some(VarKind::Var),
            JsTokenKind::Let => Some(VarKind::Let),
            JsTokenKind::Const => Some(VarKind::Const),
            _ => None,
        };

        if let Some(kind) = decl_kind {
            self.advance();
            let name_span = self.cur().span;
            let name = self.parse_binding_ident();

            if self.at(JsTokenKind::Of) {
                return self.finish_for_in_of(start, Some(kind), name, true);
            }
            if self.at(JsTokenKind::In) {
                return self.finish_for_in_of(start, Some(kind), name, false);
            }

            let init_expr = if self.at(JsTokenKind::Assign) {
                self.advance();
                Some(self.parse_assignment_expr())
            } else {
                None
            };
            let mut declarators = vec![VarDeclarator {
                name,
                init: init_expr,
                span: name_span,
            }];
            while self.at(JsTokenKind::Comma) {
                self.advance();
                let d_span = self.cur().span;
                let d_name = self.parse_binding_ident();
                let d_init = if self.at(JsTokenKind::Assign) {
                    self.advance();
                    Some(self.parse_assignment_expr())
                } else {
                    None
                };
                declarators.push(VarDeclarator {
                    name: d_name,
                    init: d_init,
                    span: d_span,
                });
            }
            let decl_span = self.span_from(name_span);
            let decl = VarDecl {
                kind,
                declarators,
                span: decl_span,
            };
            self.expect(JsTokenKind::Semicolon, ";");
            return self.parse_for_rest(start, Some(ForInit::VarDecl(decl)));
        }

        if self.at(JsTokenKind::Semicolon) {
            self.advance();
            return self.parse_for_rest(start, None);
        }

        let expr = self.parse_expression();
        self.expect(JsTokenKind::Semicolon, ";");
        self.parse_for_rest(start, Some(ForInit::Expr(expr)))
    }

    /// Finishes parsing a `for (let x of/in right) body` loop after the
    /// declaration keyword and binding name have already been consumed.
    fn finish_for_in_of(
        &mut self,
        start: motarjim_diag::SourceSpan,
        decl_kind: Option<VarKind>,
        left: String,
        is_of: bool,
    ) -> Statement {
        self.advance(); // 'of' or 'in'
        let right = if is_of {
            self.parse_assignment_expr()
        } else {
            self.parse_expression()
        };
        self.expect(JsTokenKind::RParen, ")");
        let body = Box::new(self.parse_statement());
        let span = self.span_from(start);
        if is_of {
            Statement::ForOf(ForOfStmt {
                decl_kind,
                left,
                right,
                body,
                span,
            })
        } else {
            Statement::ForIn(ForInStmt {
                decl_kind,
                left,
                right,
                body,
                span,
            })
        }
    }

    /// Parses the `test; update) body` tail shared by every `for` loop form.
    fn parse_for_rest(
        &mut self,
        start: motarjim_diag::SourceSpan,
        init: Option<ForInit>,
    ) -> Statement {
        let test = if self.at(JsTokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression())
        };
        self.expect(JsTokenKind::Semicolon, ";");
        let update = if self.at(JsTokenKind::RParen) {
            None
        } else {
            Some(self.parse_expression())
        };
        self.expect(JsTokenKind::RParen, ")");
        let body = Box::new(self.parse_statement());
        let span = self.span_from(start);
        Statement::For(Box::new(ForStmt {
            init,
            test,
            update,
            body,
            span,
        }))
    }

    /// Parses an `import` declaration.
    fn parse_import(&mut self) -> Statement {
        let start = self.advance().span; // 'import'
        let mut default = None;
        let mut namespace = None;
        let mut named = Vec::new();

        if self.at(JsTokenKind::Identifier) {
            default = Some(self.advance().raw);
            if self.at(JsTokenKind::Comma) {
                self.advance();
            }
        }

        if self.at(JsTokenKind::Star) {
            self.advance();
            self.expect(JsTokenKind::As, "as");
            namespace = Some(self.parse_binding_ident());
        } else if self.at(JsTokenKind::LBrace) {
            self.advance();
            while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
                let spec_start = self.cur().span;
                let imported = self.parse_binding_ident();
                let local = if self.at(JsTokenKind::As) {
                    self.advance();
                    self.parse_binding_ident()
                } else {
                    imported.clone()
                };
                let span = self.span_from(spec_start);
                named.push(ImportSpecifier {
                    imported,
                    local,
                    span,
                });
                if self.at(JsTokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(JsTokenKind::RBrace, "}");
        }

        let has_bindings = default.is_some() || namespace.is_some() || !named.is_empty();
        let source = if self.at(JsTokenKind::From) {
            self.advance();
            self.parse_string_literal_value()
        } else if !has_bindings {
            // Bare `import 'module';` with no bindings at all.
            self.parse_string_literal_value()
        } else {
            self.error_here(
                codes::JS_UNEXPECTED_TOKEN,
                "expected 'from' after import specifiers",
            );
            String::new()
        };

        self.eat_semicolon();
        let span = self.span_from(start);
        Statement::Import(ImportDecl {
            default,
            namespace,
            named,
            source,
            span,
        })
    }

    /// Parses a string literal token and returns its content.
    fn parse_string_literal_value(&mut self) -> String {
        if self.at(JsTokenKind::String) {
            self.advance().raw
        } else {
            self.error_here(codes::JS_UNEXPECTED_TOKEN, "expected a string literal");
            String::new()
        }
    }

    /// Parses an `export` declaration: `export default`, `export { ... }`,
    /// or `export` followed by a declaration.
    fn parse_export(&mut self) -> Statement {
        let start = self.advance().span; // 'export'

        if self.at(JsTokenKind::Default) {
            self.advance();
            let expr = self.parse_assignment_expr();
            self.eat_semicolon();
            let span = self.span_from(start);
            return Statement::ExportDefault(ExportDefaultDecl { expr, span });
        }

        if self.at(JsTokenKind::LBrace) {
            self.advance();
            let mut specifiers = Vec::new();
            while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
                specifiers.push(self.parse_binding_ident());
                if self.at(JsTokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(JsTokenKind::RBrace, "}");
            self.eat_semicolon();
            let span = self.span_from(start);
            return Statement::ExportNamed(ExportNamedDecl {
                declaration: None,
                specifiers,
                span,
            });
        }

        let declaration = Box::new(self.parse_statement());
        let span = self.span_from(start);
        Statement::ExportNamed(ExportNamedDecl {
            declaration: Some(declaration),
            specifiers: Vec::new(),
            span,
        })
    }

    /// Parses an expression statement.
    fn parse_expr_stmt(&mut self) -> Statement {
        let start = self.cur().span;
        let expr = self.parse_expression();
        self.eat_semicolon();
        let span = self.span_from(start);
        Statement::Expr(ExprStmt { expr, span })
    }

    // ---- expressions --------------------------------------------------

    /// Parses a full expression, including top-level comma sequences.
    fn parse_expression(&mut self) -> Expression {
        let start = self.cur().span;
        let first = self.parse_assignment_expr();
        if self.at(JsTokenKind::Comma) {
            let mut exprs = vec![first];
            while self.at(JsTokenKind::Comma) {
                self.advance();
                exprs.push(self.parse_assignment_expr());
            }
            let span = self.span_from(start);
            return Expression::Sequence(SequenceExpr { exprs, span });
        }
        first
    }

    /// Parses a single assignment-level expression (no top-level comma).
    fn parse_assignment_expr(&mut self) -> Expression {
        if let Some(arrow) = self.try_parse_arrow() {
            return arrow;
        }

        let start = self.cur().span;
        let left = self.parse_conditional();
        let op = match self.kind() {
            JsTokenKind::Assign => Some(AssignOp::Assign),
            JsTokenKind::PlusAssign => Some(AssignOp::AddAssign),
            JsTokenKind::MinusAssign => Some(AssignOp::SubAssign),
            JsTokenKind::StarAssign => Some(AssignOp::MulAssign),
            JsTokenKind::SlashAssign => Some(AssignOp::DivAssign),
            JsTokenKind::PercentAssign => Some(AssignOp::ModAssign),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let value = self.parse_assignment_expr();
            let span = self.span_from(start);
            return Expression::Assignment(AssignExpr {
                op,
                target: Box::new(left),
                value: Box::new(value),
                span,
            });
        }
        left
    }

    /// Attempts to parse an arrow function starting at the current position,
    /// using bounded lookahead to distinguish `(a, b) => ...` from a
    /// parenthesized or grouped expression. Returns `None` (without
    /// consuming any tokens) if the current position is not an arrow
    /// function.
    fn try_parse_arrow(&mut self) -> Option<Expression> {
        let start = self.cur().span;

        if self.at(JsTokenKind::Identifier) && self.peek_kind_at(1) == JsTokenKind::Arrow {
            let name = self.advance().raw;
            let param_span = self.span_from(start);
            self.advance(); // '=>'
            let params = vec![Param {
                name,
                default: None,
                span: param_span,
            }];
            let body = self.parse_arrow_body();
            let span = self.span_from(start);
            return Some(Expression::Arrow(ArrowFunction { params, body, span }));
        }

        if self.at(JsTokenKind::LParen) && self.arrow_params_follow() {
            let params = self.parse_params();
            self.expect(JsTokenKind::Arrow, "=>");
            let body = self.parse_arrow_body();
            let span = self.span_from(start);
            return Some(Expression::Arrow(ArrowFunction { params, body, span }));
        }

        None
    }

    /// Scans forward from the current `(` to find its matching `)`, and
    /// reports whether an `=>` immediately follows it.
    fn arrow_params_follow(&self) -> bool {
        let mut depth: i32 = 0;
        let mut i = self.pos;
        loop {
            match self.tokens.get(i).map(|t| t.kind) {
                Some(JsTokenKind::LParen) => depth += 1,
                Some(JsTokenKind::RParen) => {
                    depth -= 1;
                    if depth == 0 {
                        return self.peek_kind_at(i - self.pos + 1) == JsTokenKind::Arrow;
                    }
                }
                Some(JsTokenKind::Eof) | None => return false,
                _ => {}
            }
            i += 1;
        }
    }

    /// Parses an arrow function's body: a block or a concise expression.
    fn parse_arrow_body(&mut self) -> ArrowBody {
        if self.at(JsTokenKind::LBrace) {
            ArrowBody::Block(self.parse_block())
        } else {
            ArrowBody::Expr(Box::new(self.parse_assignment_expr()))
        }
    }

    /// Parses a ternary conditional expression: `test ? cons : alt`.
    fn parse_conditional(&mut self) -> Expression {
        let start = self.cur().span;
        let test = self.parse_nullish();
        if self.at(JsTokenKind::Question) {
            self.advance();
            let consequent = self.parse_assignment_expr();
            self.expect(JsTokenKind::Colon, ":");
            let alternate = self.parse_assignment_expr();
            let span = self.span_from(start);
            return Expression::Conditional(CondExpr {
                test: Box::new(test),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
                span,
            });
        }
        test
    }

    /// Parses a `??` nullish-coalescing chain.
    fn parse_nullish(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_logical_or();
        while self.at(JsTokenKind::Nullish) {
            self.advance();
            let right = self.parse_logical_or();
            let span = self.span_from(start);
            left = Expression::Logical(LogicalExpr {
                op: LogicalOp::NullishCoalesce,
                left: Box::new(left),
                right: Box::new(right),
                span,
            });
        }
        left
    }

    /// Parses a `||` chain.
    fn parse_logical_or(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_logical_and();
        while self.at(JsTokenKind::PipePipe) {
            self.advance();
            let right = self.parse_logical_and();
            let span = self.span_from(start);
            left = Expression::Logical(LogicalExpr {
                op: LogicalOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span,
            });
        }
        left
    }

    /// Parses a `&&` chain.
    fn parse_logical_and(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_equality();
        while self.at(JsTokenKind::AmpAmp) {
            self.advance();
            let right = self.parse_equality();
            let span = self.span_from(start);
            left = Expression::Logical(LogicalExpr {
                op: LogicalOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span,
            });
        }
        left
    }

    /// Parses an equality (`==`, `===`, `!=`, `!==`) chain.
    fn parse_equality(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_relational();
        loop {
            let op = match self.kind() {
                JsTokenKind::EqEq => BinaryOp::Eq,
                JsTokenKind::EqEqEq => BinaryOp::StrictEq,
                JsTokenKind::NotEq => BinaryOp::NotEq,
                JsTokenKind::NotEqEq => BinaryOp::StrictNotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_relational();
            let span = self.span_from(start);
            left = Expression::Binary(BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            });
        }
        left
    }

    /// Parses a relational (`<`, `>`, `<=`, `>=`, `in`, `instanceof`) chain.
    fn parse_relational(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_additive();
        loop {
            let op = match self.kind() {
                JsTokenKind::Lt => BinaryOp::Lt,
                JsTokenKind::Gt => BinaryOp::Gt,
                JsTokenKind::LtEq => BinaryOp::LtEq,
                JsTokenKind::GtEq => BinaryOp::GtEq,
                JsTokenKind::In => BinaryOp::In,
                JsTokenKind::Instanceof => BinaryOp::Instanceof,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive();
            let span = self.span_from(start);
            left = Expression::Binary(BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            });
        }
        left
    }

    /// Parses an additive (`+`, `-`) chain.
    fn parse_additive(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_multiplicative();
        loop {
            let op = match self.kind() {
                JsTokenKind::Plus => BinaryOp::Add,
                JsTokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative();
            let span = self.span_from(start);
            left = Expression::Binary(BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            });
        }
        left
    }

    /// Parses a multiplicative (`*`, `/`, `%`) chain.
    fn parse_multiplicative(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_unary();
        loop {
            let op = match self.kind() {
                JsTokenKind::Star => BinaryOp::Mul,
                JsTokenKind::Slash => BinaryOp::Div,
                JsTokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary();
            let span = self.span_from(start);
            left = Expression::Binary(BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            });
        }
        left
    }

    /// Parses a prefix unary expression, or falls through to postfix.
    fn parse_unary(&mut self) -> Expression {
        let start = self.cur().span;
        let op = match self.kind() {
            JsTokenKind::Minus => Some(UnaryOp::Neg),
            JsTokenKind::Plus => Some(UnaryOp::Plus),
            JsTokenKind::Bang => Some(UnaryOp::Not),
            JsTokenKind::Typeof => Some(UnaryOp::Typeof),
            JsTokenKind::Void => Some(UnaryOp::Void),
            JsTokenKind::Delete => Some(UnaryOp::Delete),
            JsTokenKind::Increment => Some(UnaryOp::Increment),
            JsTokenKind::Decrement => Some(UnaryOp::Decrement),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let argument = self.parse_unary();
            let span = self.span_from(start);
            return Expression::Unary(UnaryExpr {
                op,
                argument: Box::new(argument),
                prefix: true,
                span,
            });
        }
        self.parse_postfix()
    }

    /// Parses a postfix `++`/`--`, or falls through to call/member chains.
    fn parse_postfix(&mut self) -> Expression {
        let start = self.cur().span;
        let expr = self.parse_call_member();
        let op = match self.kind() {
            JsTokenKind::Increment => Some(UnaryOp::Increment),
            JsTokenKind::Decrement => Some(UnaryOp::Decrement),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let span = self.span_from(start);
            return Expression::Unary(UnaryExpr {
                op,
                argument: Box::new(expr),
                prefix: false,
                span,
            });
        }
        expr
    }

    /// Parses a `new` expression, or a primary expression followed by any
    /// chain of `.prop`, `[expr]`, and `(args)`.
    fn parse_call_member(&mut self) -> Expression {
        let start = self.cur().span;
        let mut expr = if self.at(JsTokenKind::New) {
            self.advance();
            let callee = self.parse_new_callee();
            let args = if self.at(JsTokenKind::LParen) {
                self.parse_arguments()
            } else {
                Vec::new()
            };
            let span = self.span_from(start);
            Expression::New(NewExpr {
                callee: Box::new(callee),
                args,
                span,
            })
        } else {
            self.parse_primary()
        };

        loop {
            match self.kind() {
                JsTokenKind::Dot => {
                    self.advance();
                    let name = self.parse_property_name();
                    let span = self.span_from(start);
                    expr = Expression::Member(MemberExpr {
                        object: Box::new(expr),
                        property: MemberProp::Ident(name),
                        span,
                    });
                }
                JsTokenKind::LBracket => {
                    self.advance();
                    let prop = self.parse_expression();
                    self.expect(JsTokenKind::RBracket, "]");
                    let span = self.span_from(start);
                    expr = Expression::Member(MemberExpr {
                        object: Box::new(expr),
                        property: MemberProp::Computed(Box::new(prop)),
                        span,
                    });
                }
                JsTokenKind::LParen => {
                    let args = self.parse_arguments();
                    let span = self.span_from(start);
                    expr = Expression::Call(CallExpr {
                        callee: Box::new(expr),
                        args,
                        span,
                    });
                }
                _ => break,
            }
        }
        expr
    }

    /// Parses the callee of a `new` expression: a primary expression
    /// followed only by `.prop`/`[expr]` accesses (call parens belong to
    /// `new`'s own argument list, not the callee).
    fn parse_new_callee(&mut self) -> Expression {
        let start = self.cur().span;
        let mut expr = self.parse_primary();
        loop {
            match self.kind() {
                JsTokenKind::Dot => {
                    self.advance();
                    let name = self.parse_property_name();
                    let span = self.span_from(start);
                    expr = Expression::Member(MemberExpr {
                        object: Box::new(expr),
                        property: MemberProp::Ident(name),
                        span,
                    });
                }
                JsTokenKind::LBracket => {
                    self.advance();
                    let prop = self.parse_expression();
                    self.expect(JsTokenKind::RBracket, "]");
                    let span = self.span_from(start);
                    expr = Expression::Member(MemberExpr {
                        object: Box::new(expr),
                        property: MemberProp::Computed(Box::new(prop)),
                        span,
                    });
                }
                _ => break,
            }
        }
        expr
    }

    /// Parses a property name after `.`. Accepts keywords as well as plain
    /// identifiers, since JavaScript allows keywords in property position
    /// (e.g. `x.default`, `x.class`).
    fn parse_property_name(&mut self) -> String {
        self.advance().raw
    }

    /// Parses a parenthesized, comma-separated call argument list.
    fn parse_arguments(&mut self) -> Vec<Expression> {
        self.expect(JsTokenKind::LParen, "(");
        let mut args = Vec::new();
        if !self.at(JsTokenKind::RParen) {
            loop {
                args.push(self.parse_assignment_expr());
                if self.at(JsTokenKind::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        self.expect(JsTokenKind::RParen, ")");
        args
    }

    /// Parses a primary expression: literals, identifiers, grouping,
    /// arrays, objects, and function expressions.
    fn parse_primary(&mut self) -> Expression {
        match self.kind() {
            JsTokenKind::Number => {
                let tok = self.advance();
                let value = parse_number_literal(&tok.raw);
                Expression::Number(NumberLit {
                    value,
                    raw: tok.raw,
                    span: tok.span,
                })
            }
            JsTokenKind::String => {
                let tok = self.advance();
                Expression::String(StringLit {
                    value: tok.raw,
                    span: tok.span,
                })
            }
            JsTokenKind::TemplateString => self.parse_template_literal(),
            JsTokenKind::True => {
                let tok = self.advance();
                Expression::Bool(BoolLit {
                    value: true,
                    span: tok.span,
                })
            }
            JsTokenKind::False => {
                let tok = self.advance();
                Expression::Bool(BoolLit {
                    value: false,
                    span: tok.span,
                })
            }
            JsTokenKind::Null => Expression::Null(self.advance().span),
            JsTokenKind::Undefined => Expression::Undefined(self.advance().span),
            JsTokenKind::This => Expression::This(self.advance().span),
            JsTokenKind::Identifier => {
                let tok = self.advance();
                Expression::Identifier(Ident {
                    name: tok.raw,
                    span: tok.span,
                })
            }
            JsTokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression();
                self.expect(JsTokenKind::RParen, ")");
                expr
            }
            JsTokenKind::LBracket => self.parse_array_literal(),
            JsTokenKind::LBrace => self.parse_object_literal(),
            JsTokenKind::Function => self.parse_function_expr(),
            _ => {
                self.error_here(
                    codes::JS_UNEXPECTED_TOKEN,
                    format!("unexpected token in expression: {:?}", self.kind()),
                );
                Expression::Undefined(self.advance().span)
            }
        }
    }

    /// Parses a `[ ... ]` array literal.
    fn parse_array_literal(&mut self) -> Expression {
        let start = self.advance().span; // '['
        let mut elements = Vec::new();
        while !self.at(JsTokenKind::RBracket) && !self.at(JsTokenKind::Eof) {
            elements.push(self.parse_assignment_expr());
            if self.at(JsTokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(JsTokenKind::RBracket, "]");
        let span = self.span_from(start);
        Expression::Array(ArrayLit { elements, span })
    }

    /// Parses a `{ ... }` object literal, including shorthand properties.
    fn parse_object_literal(&mut self) -> Expression {
        let start = self.advance().span; // '{'
        let mut props = Vec::new();
        while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
            let prop_start = self.cur().span;
            let key = if self.at(JsTokenKind::LBracket) {
                self.advance();
                let expr = self.parse_assignment_expr();
                self.expect(JsTokenKind::RBracket, "]");
                PropKey::Computed(Box::new(expr))
            } else if self.at(JsTokenKind::String) {
                PropKey::String(self.advance().raw)
            } else {
                PropKey::Ident(self.advance().raw)
            };
            let value = if self.at(JsTokenKind::Colon) {
                self.advance();
                self.parse_assignment_expr()
            } else if let PropKey::Ident(name) = &key {
                Expression::Identifier(Ident {
                    name: name.clone(),
                    span: prop_start,
                })
            } else {
                self.error_here(codes::JS_UNEXPECTED_TOKEN, "expected ':' in object literal");
                Expression::Undefined(prop_start)
            };
            let span = self.span_from(prop_start);
            props.push(ObjectProp { key, value, span });
            if self.at(JsTokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(JsTokenKind::RBrace, "}");
        let span = self.span_from(start);
        Expression::Object(ObjectLit { props, span })
    }

    /// Parses an unnamed or named function expression.
    fn parse_function_expr(&mut self) -> Expression {
        let start = self.advance().span; // 'function'
        let name = if self.at(JsTokenKind::Identifier) {
            Some(self.advance().raw)
        } else {
            None
        };
        let params = self.parse_params();
        let body = self.parse_block();
        let span = self.span_from(start);
        Expression::Function(FunctionExpr {
            name,
            params,
            body,
            span,
        })
    }

    /// Parses a template literal token into its quasis and interpolated
    /// expressions, re-parsing each `${ ... }` interpolation with a fresh
    /// sub-parser and translating its spans back into this parser's source
    /// coordinate space.
    fn parse_template_literal(&mut self) -> Expression {
        let tok = self.advance(); // TemplateString, raw includes backticks
        let base_offset = tok.span.start.offset;
        let inner = tok
            .raw
            .get(1..tok.raw.len().saturating_sub(1))
            .unwrap_or("");

        let mut quasis = Vec::new();
        let mut exprs = Vec::new();
        for part in split_template_parts(inner) {
            match part {
                TemplatePart::Quasi(text) => quasis.push(text),
                TemplatePart::Expr {
                    source,
                    offset_in_inner,
                } => {
                    let mut sub_parser = Self::new(source);
                    let mut expr = sub_parser.parse_expression();
                    self.diagnostics.extend(sub_parser.diagnostics);
                    // +1 skips the template literal's opening backtick.
                    let delta = base_offset + 1 + offset_in_inner;
                    expr.shift_spans(delta);
                    exprs.push(expr);
                }
            }
        }
        // Invariant maintained by `split_template_parts`: one more quasi than
        // interpolated expression. Defensive fixup keeps this method total
        // even if that invariant were ever violated.
        quasis.resize(exprs.len() + 1, String::new());

        Expression::TemplateLiteral(TemplateLiteral {
            quasis,
            exprs,
            span: tok.span,
        })
    }
}

/// Parses a JavaScript numeric literal's raw text (e.g. `"1_000"`, `"0xFF"`,
/// `"1.5e10"`) into its `f64` value. Malformed input yields `0.0` rather
/// than panicking; the lexer only ever produces well-formed numeric raw text
/// in the first place.
fn parse_number_literal(raw: &str) -> f64 {
    let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
    if let Some(hex) = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).map_or(0.0, |v| v as f64)
    } else {
        cleaned.parse::<f64>().unwrap_or(0.0)
    }
}

/// A chunk produced by splitting a template literal's inner text (with the
/// surrounding backticks already stripped) into static and dynamic parts.
enum TemplatePart<'a> {
    /// A static string chunk between interpolations.
    Quasi(String),
    /// A `${ ... }` interpolation.
    Expr {
        /// The interpolation's source text, with `${` and `}` stripped.
        source: &'a str,
        /// The byte offset of `source`'s first character within `inner`.
        offset_in_inner: u32,
    },
}

/// Splits a template literal's inner text into quasis and interpolations.
///
/// Always produces exactly `exprs.len() + 1` quasis, with `quasis[i]`
/// preceding the `i`-th interpolation and the final quasi being the
/// trailing tail.
fn split_template_parts(inner: &str) -> Vec<TemplatePart<'_>> {
    let bytes = inner.as_bytes();
    let mut parts = Vec::new();
    let mut quasi_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        if bytes[i] == b'$' && bytes.get(i + 1) == Some(&b'{') {
            parts.push(TemplatePart::Quasi(inner[quasi_start..i].to_string()));
            let expr_start = i + 2;
            let expr_end = find_matching_brace(inner, expr_start);
            parts.push(TemplatePart::Expr {
                source: &inner[expr_start..expr_end],
                offset_in_inner: expr_start as u32,
            });
            i = (expr_end + 1).min(bytes.len());
            quasi_start = i;
            continue;
        }
        i += 1;
    }
    parts.push(TemplatePart::Quasi(inner[quasi_start..].to_string()));
    parts
}

/// Finds the byte offset of the `}` matching the `${` whose body starts at
/// `start`, skipping over nested braces, string literals, and nested
/// template literals so they cannot miscount the depth.
fn find_matching_brace(inner: &str, start: usize) -> usize {
    let bytes = inner.as_bytes();
    let mut i = start;
    let mut depth: u32 = 1;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
                i += 1;
            }
            b'\'' | b'"' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i = (i + 1).min(bytes.len());
            }
            b'`' => {
                i = skip_nested_template(inner, i + 1);
            }
            _ => i += 1,
        }
    }
    bytes.len()
}

/// Skips over a nested template literal body starting right after its
/// opening backtick, returning the offset just past its closing backtick.
fn skip_nested_template(inner: &str, mut i: usize) -> usize {
    let bytes = inner.as_bytes();
    while i < bytes.len() {
        match bytes[i] {
            b'`' => return i + 1,
            b'\\' => i = (i + 2).min(bytes.len()),
            b'$' if bytes.get(i + 1) == Some(&b'{') => {
                i = find_matching_brace(inner, i + 2) + 1;
            }
            _ => i += 1,
        }
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_decl() {
        let mut p = JsParser::new("let x = 1;");
        let program = p.parse().expect("should parse");
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::VarDecl(_)));
    }

    #[test]
    fn test_multiple_declarators() {
        let mut p = JsParser::new("var a = 1, b = 2, c;");
        let program = p.parse().expect("should parse");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        assert_eq!(decl.declarators.len(), 3);
        assert!(decl.declarators[2].init.is_none());
    }

    #[test]
    fn test_function_decl() {
        let mut p = JsParser::new("function add(a, b) { return a + b; }");
        let program = p.parse().expect("should parse");
        let Statement::FunctionDecl(f) = &program.body[0] else {
            panic!("expected function decl");
        };
        assert_eq!(f.name, "add");
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.body.body.len(), 1);
    }

    #[test]
    fn test_arrow_single_param() {
        let mut p = JsParser::new("const double = x => x * 2;");
        let program = p.parse().expect("should parse");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        let Some(Expression::Arrow(arrow)) = &decl.declarators[0].init else {
            panic!("expected arrow function");
        };
        assert_eq!(arrow.params.len(), 1);
        assert!(matches!(arrow.body, ArrowBody::Expr(_)));
    }

    #[test]
    fn test_arrow_multi_param_block_body() {
        let mut p = JsParser::new("const add = (a, b) => { return a + b; };");
        let program = p.parse().expect("should parse");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        let Some(Expression::Arrow(arrow)) = &decl.declarators[0].init else {
            panic!("expected arrow function");
        };
        assert_eq!(arrow.params.len(), 2);
        assert!(matches!(arrow.body, ArrowBody::Block(_)));
    }

    #[test]
    fn test_grouped_expression_is_not_arrow() {
        let mut p = JsParser::new("(a, b);");
        let program = p.parse().expect("should parse");
        let Statement::Expr(stmt) = &program.body[0] else {
            panic!("expected expr stmt");
        };
        assert!(matches!(stmt.expr, Expression::Sequence(_)));
    }

    #[test]
    fn test_template_literal_with_interpolation() {
        let mut p = JsParser::new("const msg = `Hi, ${name}!`;");
        let program = p.parse().expect("should parse");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        let Some(Expression::TemplateLiteral(tpl)) = &decl.declarators[0].init else {
            panic!("expected template literal");
        };
        assert_eq!(tpl.quasis, vec!["Hi, ".to_string(), "!".to_string()]);
        assert_eq!(tpl.exprs.len(), 1);
        assert!(matches!(tpl.exprs[0], Expression::Identifier(_)));
    }

    #[test]
    fn test_template_literal_span_offset_is_absolute() {
        let src = "const msg = `Hi, ${name}!`;";
        let mut p = JsParser::new(src);
        let program = p.parse().expect("should parse");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        let Some(Expression::TemplateLiteral(tpl)) = &decl.declarators[0].init else {
            panic!("expected template literal");
        };
        let name_span = tpl.exprs[0].span();
        let expected_start = src.find("name").expect("name substring present") as u32;
        assert_eq!(name_span.start.offset, expected_start);
    }

    #[test]
    fn test_if_else() {
        let mut p = JsParser::new("if (x) { y(); } else { z(); }");
        let program = p.parse().expect("should parse");
        let Statement::If(stmt) = &program.body[0] else {
            panic!("expected if statement");
        };
        assert!(stmt.alternate.is_some());
    }

    #[test]
    fn test_for_loop() {
        let mut p = JsParser::new("for (let i = 0; i < 10; i++) { sum += i; }");
        let program = p.parse().expect("should parse");
        assert!(matches!(program.body[0], Statement::For(_)));
    }

    #[test]
    fn test_for_of_loop() {
        let mut p = JsParser::new("for (const item of items) { console.log(item); }");
        let program = p.parse().expect("should parse");
        let Statement::ForOf(stmt) = &program.body[0] else {
            panic!("expected for-of loop");
        };
        assert_eq!(stmt.left, "item");
    }

    #[test]
    fn test_for_in_loop() {
        let mut p = JsParser::new("for (const key in obj) { use(key); }");
        let program = p.parse().expect("should parse");
        assert!(matches!(program.body[0], Statement::ForIn(_)));
    }

    #[test]
    fn test_while_loop() {
        let mut p = JsParser::new("while (running) { tick(); }");
        let program = p.parse().expect("should parse");
        assert!(matches!(program.body[0], Statement::While(_)));
    }

    #[test]
    fn test_do_while_loop() {
        let mut p = JsParser::new("do { tick(); } while (running);");
        let program = p.parse().expect("should parse");
        assert!(matches!(program.body[0], Statement::DoWhile(_)));
    }

    #[test]
    fn test_import_default_and_named() {
        let mut p = JsParser::new("import React, { useState as useS } from 'react';");
        let program = p.parse().expect("should parse");
        let Statement::Import(decl) = &program.body[0] else {
            panic!("expected import decl");
        };
        assert_eq!(decl.default.as_deref(), Some("React"));
        assert_eq!(decl.named[0].imported, "useState");
        assert_eq!(decl.named[0].local, "useS");
        assert_eq!(decl.source, "react");
    }

    #[test]
    fn test_import_namespace() {
        let mut p = JsParser::new("import * as utils from './utils.js';");
        let program = p.parse().expect("should parse");
        let Statement::Import(decl) = &program.body[0] else {
            panic!("expected import decl");
        };
        assert_eq!(decl.namespace.as_deref(), Some("utils"));
    }

    #[test]
    fn test_export_default() {
        let mut p = JsParser::new("export default function main() {}");
        let program = p.parse().expect("should parse");
        assert!(matches!(program.body[0], Statement::ExportDefault(_)));
    }

    #[test]
    fn test_export_named_declaration() {
        let mut p = JsParser::new("export const PI = 3.14;");
        let program = p.parse().expect("should parse");
        let Statement::ExportNamed(decl) = &program.body[0] else {
            panic!("expected export decl");
        };
        assert!(decl.declaration.is_some());
    }

    #[test]
    fn test_dom_addeventlistener_call() {
        let mut p =
            JsParser::new("button.addEventListener('click', () => console.log('clicked'));");
        let program = p.parse().expect("should parse");
        assert!(matches!(program.body[0], Statement::Expr(_)));
    }

    #[test]
    fn test_operator_precedence() {
        let mut p = JsParser::new("1 + 2 * 3;");
        let program = p.parse().expect("should parse");
        let Statement::Expr(stmt) = &program.body[0] else {
            panic!("expected expr stmt");
        };
        let Expression::Binary(add) = &stmt.expr else {
            panic!("expected top-level addition");
        };
        assert_eq!(add.op, BinaryOp::Add);
        assert!(matches!(*add.right, Expression::Binary(_)));
    }

    #[test]
    fn test_ternary() {
        let mut p = JsParser::new("const x = a ? b : c;");
        let program = p.parse().expect("should parse");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        assert!(matches!(
            decl.declarators[0].init,
            Some(Expression::Conditional(_))
        ));
    }

    #[test]
    fn test_object_literal_shorthand() {
        let mut p = JsParser::new("const o = { x, y: 2 };");
        let program = p.parse().expect("should parse");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        let Some(Expression::Object(obj)) = &decl.declarators[0].init else {
            panic!("expected object literal");
        };
        assert_eq!(obj.props.len(), 2);
    }

    #[test]
    fn test_array_literal() {
        let mut p = JsParser::new("const arr = [1, 2, 3];");
        let program = p.parse().expect("should parse");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        let Some(Expression::Array(arr)) = &decl.declarators[0].init else {
            panic!("expected array literal");
        };
        assert_eq!(arr.elements.len(), 3);
    }

    #[test]
    fn test_member_and_call_chain() {
        let mut p = JsParser::new("a.b.c(1)[2];");
        let program = p.parse().expect("should parse");
        let Statement::Expr(stmt) = &program.body[0] else {
            panic!("expected expr stmt");
        };
        assert!(matches!(stmt.expr, Expression::Member(_)));
    }

    #[test]
    fn test_new_expression() {
        let mut p = JsParser::new("new Foo(1, 2);");
        let program = p.parse().expect("should parse");
        let Statement::Expr(stmt) = &program.body[0] else {
            panic!("expected expr stmt");
        };
        assert!(matches!(stmt.expr, Expression::New(_)));
    }

    #[test]
    fn test_unexpected_token_reports_diagnostic_and_recovers() {
        let mut p = JsParser::new("let x = ; let y = 2;");
        let result = p.parse();
        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_hex_and_float_numbers() {
        let mut p = JsParser::new("const a = 0xFF; const b = 1.5e2;");
        let program = p.parse().expect("should parse");
        let Statement::VarDecl(decl_a) = &program.body[0] else {
            panic!("expected var decl");
        };
        assert_eq!(
            decl_a.declarators[0].init,
            Some(Expression::Number(NumberLit {
                value: 255.0,
                raw: "0xFF".to_string(),
                span: decl_a.declarators[0]
                    .init
                    .as_ref()
                    .expect("has init")
                    .span(),
            }))
        );
    }
}
