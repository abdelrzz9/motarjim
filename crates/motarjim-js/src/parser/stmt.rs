//! Statement parser: declarations, control flow, imports, exports.

use motarjim_span::SourceSpan;

use crate::ast::stmt::*;
use crate::ast::expr::*;
use crate::ast::pat::*;
use crate::ast::lit::*;
use crate::parser::JsParser;
use crate::token::JsTokenKind;

impl JsParser {
    pub(crate) fn parse_statement(&mut self) -> Statement {
        match self.kind() {
            JsTokenKind::Var | JsTokenKind::Let | JsTokenKind::Const => self.parse_var_decl_stmt(),
            JsTokenKind::Function => self.parse_function_decl(),
            JsTokenKind::Async if self.peek_kind_at(1) == JsTokenKind::Function => {
                self.parse_async_function_decl()
            }
            JsTokenKind::Class => self.parse_class_decl(),
            JsTokenKind::Return => self.parse_return(),
            JsTokenKind::If => self.parse_if(),
            JsTokenKind::Switch => self.parse_switch(),
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
            JsTokenKind::Throw => self.parse_throw(),
            JsTokenKind::Try => self.parse_try(),
            JsTokenKind::Debugger => {
                let span = self.advance().span;
                self.eat_semicolon();
                Statement::Debugger(DebuggerStmt { span })
            }
            JsTokenKind::Import => self.parse_import(),
            JsTokenKind::Export => self.parse_export(),
            JsTokenKind::LBrace => Statement::Block(self.parse_block()),
            JsTokenKind::Semicolon => Statement::Empty(self.advance().span),
            JsTokenKind::Identifier if self.peek_kind_at(1) == JsTokenKind::Colon => {
                self.parse_labelled()
            }
            JsTokenKind::At => {
                self.advance();
                self.parse_expression();
                self.error("decorators are not supported yet");
                Statement::Empty(self.cur().span)
            }
            _ => self.parse_expr_stmt(),
        }
    }

    pub(crate) fn parse_block(&mut self) -> BlockStmt {
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

    fn parse_labelled(&mut self) -> Statement {
        let start = self.cur().span;
        let label = self.advance().raw;
        self.advance();
        let body = Box::new(self.parse_statement());
        let span = self.span_from(start);
        Statement::Labelled { label, body, span }
    }

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
            let name = self.parse_pattern();
            let init = if self.eat(JsTokenKind::Assign) {
                Some(self.parse_assignment_expr())
            } else {
                None
            };
            let span = self.span_from(decl_start);
            declarators.push(VarDeclarator { name, init, span });
            if !self.eat(JsTokenKind::Comma) {
                break;
            }
        }
        let span = self.span_from(start);
        VarDecl { kind, declarators, span }
    }

    fn parse_var_decl_stmt(&mut self) -> Statement {
        let decl = self.parse_var_decl();
        self.eat_semicolon();
        Statement::VarDecl(decl)
    }

    fn parse_function_decl(&mut self) -> Statement {
        self.parse_function(false)
    }

    fn parse_async_function_decl(&mut self) -> Statement {
        self.parse_function(true)
    }

    fn parse_function(&mut self, r#async: bool) -> Statement {
        let start = self.cur().span;
        if r#async {
            self.advance();
        }
        self.advance();
        let name = self.parse_pattern();
        let params = self.parse_params();
        let body = self.parse_block();
        let span = self.span_from(start);
        Statement::FunctionDecl(FunctionDecl {
            name,
            params,
            body,
            generator: false,
            r#async,
            span,
        })
    }

    pub(crate) fn parse_params(&mut self) -> Vec<Param> {
        self.expect(JsTokenKind::LParen, "(");
        let mut params = Vec::new();
        if !self.at(JsTokenKind::RParen) {
            loop {
                let pat = self.parse_pattern();
                let default = if self.eat(JsTokenKind::Assign) {
                    Some(self.parse_assignment_expr())
                } else {
                    None
                };
                let span = pat.span();
                params.push(Param { pat, default, span });
                if !self.eat(JsTokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(JsTokenKind::RParen, ")");
        params
    }

    fn parse_return(&mut self) -> Statement {
        let start = self.advance().span;
        let argument = if matches!(self.kind(), JsTokenKind::Semicolon | JsTokenKind::RBrace | JsTokenKind::Eof) {
            None
        } else {
            Some(self.parse_expression())
        };
        self.eat_semicolon();
        let span = self.span_from(start);
        Statement::Return(ReturnStmt { argument, span })
    }

    fn parse_if(&mut self) -> Statement {
        let start = self.advance().span;
        self.expect(JsTokenKind::LParen, "(");
        let test = self.parse_expression();
        self.expect(JsTokenKind::RParen, ")");
        let consequent = Box::new(self.parse_statement());
        let alternate = if self.eat(JsTokenKind::Else) {
            Some(Box::new(self.parse_statement()))
        } else {
            None
        };
        let span = self.span_from(start);
        Statement::If(IfStmt { test, consequent, alternate, span })
    }

    fn parse_switch(&mut self) -> Statement {
        let start = self.advance().span;
        self.expect(JsTokenKind::LParen, "(");
        let discriminant = self.parse_expression();
        self.expect(JsTokenKind::RParen, ")");
        self.expect(JsTokenKind::LBrace, "{");
        let mut cases = Vec::new();
        while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
            if self.eat(JsTokenKind::Case) {
                let test = self.parse_expression();
                self.expect(JsTokenKind::Colon, ":");
                let mut consequent = Vec::new();
                while !self.at_any(&[JsTokenKind::Case, JsTokenKind::Default, JsTokenKind::RBrace, JsTokenKind::Eof]) {
                    consequent.push(self.parse_statement());
                }
                let span = self.span_from(start);
                cases.push(SwitchCase { test: Some(test), consequent, span });
            } else if self.at(JsTokenKind::Default) {
                self.advance();
                self.expect(JsTokenKind::Colon, ":");
                let mut consequent = Vec::new();
                while !self.at_any(&[JsTokenKind::Case, JsTokenKind::Default, JsTokenKind::RBrace, JsTokenKind::Eof]) {
                    consequent.push(self.parse_statement());
                }
                let span = self.span_from(start);
                cases.push(SwitchCase { test: None, consequent, span });
            } else {
                self.error("expected 'case' or 'default'");
                self.advance();
            }
        }
        self.expect(JsTokenKind::RBrace, "}");
        let span = self.span_from(start);
        Statement::Switch(SwitchStmt { discriminant, cases, span })
    }

    fn parse_while(&mut self) -> Statement {
        let start = self.advance().span;
        self.expect(JsTokenKind::LParen, "(");
        let test = self.parse_expression();
        self.expect(JsTokenKind::RParen, ")");
        let body = Box::new(self.parse_statement());
        let span = self.span_from(start);
        Statement::While(WhileStmt { test, body, span })
    }

    fn parse_do_while(&mut self) -> Statement {
        let start = self.advance().span;
        let body = Box::new(self.parse_statement());
        self.expect(JsTokenKind::While, "while");
        self.expect(JsTokenKind::LParen, "(");
        let test = self.parse_expression();
        self.expect(JsTokenKind::RParen, ")");
        self.eat_semicolon();
        let span = self.span_from(start);
        Statement::DoWhile(DoWhileStmt { body, test, span })
    }

    fn parse_for(&mut self) -> Statement {
        let start = self.advance().span;
        self.expect(JsTokenKind::LParen, "(");

        if self.eat(JsTokenKind::Semicolon) {
            return self.parse_for_rest(start, None);
        }

        if self.at_any(&[JsTokenKind::Var, JsTokenKind::Let, JsTokenKind::Const]) {
            let kind = match self.kind() {
                JsTokenKind::Var => VarKind::Var,
                JsTokenKind::Let => VarKind::Let,
                _ => VarKind::Const,
            };
            self.advance();
            let name = self.parse_pattern();

            if self.eat(JsTokenKind::Of) {
                let right = self.parse_assignment_expr();
                self.expect(JsTokenKind::RParen, ")");
                let body = Box::new(self.parse_statement());
                let span = self.span_from(start);
                return Statement::ForOf(ForOfStmt {
                    left: name,
                    right,
                    body,
                    r#await: false,
                    span,
                });
            }
            if self.eat(JsTokenKind::In) {
                let right = self.parse_expression();
                self.expect(JsTokenKind::RParen, ")");
                let body = Box::new(self.parse_statement());
                let span = self.span_from(start);
                return Statement::ForIn(ForInStmt { left: name, right, body, span });
            }

            let init_expr = if self.eat(JsTokenKind::Assign) {
                Some(self.parse_assignment_expr())
            } else {
                None
            };
            let mut declarators = vec![VarDeclarator {
                name,
                init: init_expr,
                span: self.cur().span,
            }];
            while self.eat(JsTokenKind::Comma) {
                let d_name = self.parse_pattern();
                let d_init = if self.eat(JsTokenKind::Assign) {
                    Some(self.parse_assignment_expr())
                } else {
                    None
                };
                declarators.push(VarDeclarator {
                    name: d_name,
                    init: d_init,
                    span: self.cur().span,
                });
            }
            self.expect(JsTokenKind::Semicolon, ";");
            let decl = VarDecl { kind, declarators, span: self.span_from(start) };
            return self.parse_for_rest(start, Some(ForInit::VarDecl(decl)));
        }

        let expr = self.parse_expression();

        if self.eat(JsTokenKind::Of) {
            let right = self.parse_assignment_expr();
            self.expect(JsTokenKind::RParen, ")");
            let body = Box::new(self.parse_statement());
            let span = self.span_from(start);
            if let Expression::Identifier(name, _) = &expr {
                return Statement::ForOf(ForOfStmt {
                    left: Pattern::Ident(name.clone(), expr.span()),
                    right,
                    body,
                    r#await: false,
                    span,
                });
            }
            self.error("invalid left-hand side in for-of");
            return Statement::ForOf(ForOfStmt {
                left: Pattern::Ident(String::new(), expr.span()),
                right,
                body,
                r#await: false,
                span,
            });
        }

        if self.eat(JsTokenKind::In) {
            let right = self.parse_expression();
            self.expect(JsTokenKind::RParen, ")");
            let body = Box::new(self.parse_statement());
            let span = self.span_from(start);
            if let Expression::Identifier(name, _) = &expr {
                return Statement::ForIn(ForInStmt {
                    left: Pattern::Ident(name.clone(), expr.span()),
                    right,
                    body,
                    span,
                });
            }
            self.error("invalid left-hand side in for-in");
            return Statement::ForIn(ForInStmt {
                left: Pattern::Ident(String::new(), expr.span()),
                right,
                body,
                span,
            });
        }

        self.expect(JsTokenKind::Semicolon, ";");
        self.parse_for_rest(start, Some(ForInit::Expr(expr)))
    }

    fn parse_for_rest(&mut self, start: SourceSpan, init: Option<ForInit>) -> Statement {
        let test = if self.at(JsTokenKind::Semicolon) { None }
        else { Some(self.parse_expression()) };
        self.expect(JsTokenKind::Semicolon, ";");
        let update = if self.at(JsTokenKind::RParen) { None }
        else { Some(self.parse_expression()) };
        self.expect(JsTokenKind::RParen, ")");
        let body = Box::new(self.parse_statement());
        let span = self.span_from(start);
        Statement::For(Box::new(ForStmt { init, test, update, body, span }))
    }

    fn parse_throw(&mut self) -> Statement {
        let start = self.advance().span;
        let argument = self.parse_expression();
        self.eat_semicolon();
        let span = self.span_from(start);
        Statement::Throw(ThrowStmt { argument, span })
    }

    fn parse_try(&mut self) -> Statement {
        let start = self.advance().span;
        let block = self.parse_block();
        let handler = if self.at(JsTokenKind::Catch) {
            let catch_start = self.advance().span;
            let param = if self.eat(JsTokenKind::LParen) {
                let pat = self.parse_pattern();
                self.expect(JsTokenKind::RParen, ")");
                Some(pat)
            } else {
                None
            };
            let body = self.parse_block();
            let span = self.span_from(catch_start);
            Some(CatchClause { param, body, span })
        } else {
            None
        };
        let finalizer = if self.eat(JsTokenKind::Finally) {
            Some(self.parse_block())
        } else {
            None
        };
        let span = self.span_from(start);
        Statement::Try(TryStmt { block, handler, finalizer, span })
    }

    fn parse_import(&mut self) -> Statement {
        let start = self.advance().span;
        let mut default = None;
        let mut namespace = None;
        let mut named = Vec::new();

        if self.at(JsTokenKind::Identifier) {
            default = Some(self.advance().raw);
            self.eat(JsTokenKind::Comma);
        }

        if self.eat(JsTokenKind::Star) {
            self.expect(JsTokenKind::As, "as");
            namespace = Some(self.parse_binding_ident());
        } else if self.eat(JsTokenKind::LBrace) {
            while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
                let spec_start = self.cur().span;
                let imported = self.parse_binding_ident();
                let local = if self.eat(JsTokenKind::As) {
                    self.parse_binding_ident()
                } else {
                    imported.clone()
                };
                let span = self.span_from(spec_start);
                named.push(ImportSpecifier { imported, local, span });
                self.eat(JsTokenKind::Comma);
            }
            self.expect(JsTokenKind::RBrace, "}");
        }

        let has_bindings = default.is_some() || namespace.is_some() || !named.is_empty();
        let source = if self.eat(JsTokenKind::From) || !has_bindings {
            self.parse_string_literal_value()
        } else {
            self.error("expected 'from' after import specifiers");
            String::new()
        };

        self.eat_semicolon();
        let span = self.span_from(start);
        Statement::Import(ImportDecl { default, namespace, named, source, span })
    }

    fn parse_string_literal_value(&mut self) -> String {
        if self.at(JsTokenKind::String) {
            if let crate::token::TokenValue::String(s) = &self.cur().value {
                let s = s.clone();
                self.advance();
                return s;
            }
            self.advance().raw
        } else {
            self.error("expected a string literal");
            String::new()
        }
    }

    fn parse_export(&mut self) -> Statement {
        let start = self.advance().span;

        if self.eat(JsTokenKind::Default) {
            if self.at(JsTokenKind::Function) {
                let decl = self.parse_function_decl();
                return self.make_export_default(start, ExportDefaultKind::FunctionDecl(
                    match decl {
                        Statement::FunctionDecl(f) => f,
                        _ => unreachable!(),
                    }
                ));
            }
            if self.at(JsTokenKind::Class) {
                let decl = self.parse_class_decl();
                return self.make_export_default(start, ExportDefaultKind::ClassDecl(
                    match decl {
                        Statement::ClassDecl(c) => c,
                        _ => unreachable!(),
                    }
                ));
            }
            let expr = self.parse_assignment_expr();
            self.eat_semicolon();
            let span = self.span_from(start);
            return Statement::ExportDefault(ExportDefaultDecl {
                declaration: ExportDefaultKind::Expression(expr),
                span,
            });
        }

        if self.eat(JsTokenKind::LBrace) {
            let mut specifiers = Vec::new();
            while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
                let spec_start = self.cur().span;
                let local = self.parse_binding_ident();
                let exported = if self.eat(JsTokenKind::As) {
                    self.parse_binding_ident()
                } else {
                    local.clone()
                };
                let span = self.span_from(spec_start);
                specifiers.push(ExportSpecifier { local, exported, span });
                self.eat(JsTokenKind::Comma);
            }
            self.expect(JsTokenKind::RBrace, "}");
            let source = if self.eat(JsTokenKind::From) {
                Some(self.parse_string_literal_value())
            } else {
                None
            };
            self.eat_semicolon();
            let span = self.span_from(start);
            return Statement::ExportNamed(ExportNamedDecl {
                declaration: None,
                specifiers,
                source,
                span,
            });
        }

        let declaration = Box::new(self.parse_statement());
        let span = self.span_from(start);
        Statement::ExportNamed(ExportNamedDecl {
            declaration: Some(declaration),
            specifiers: Vec::new(),
            source: None,
            span,
        })
    }

    fn make_export_default(&self, start: SourceSpan, declaration: ExportDefaultKind) -> Statement {
        let span = self.span_from(start);
        Statement::ExportDefault(ExportDefaultDecl { declaration, span })
    }

    fn parse_class_decl(&mut self) -> Statement {
        let start = self.advance().span;
        let name = self.parse_pattern();
        let super_class = if self.eat(JsTokenKind::Extends) {
            Some(self.parse_expression())
        } else {
            None
        };
        let body = self.parse_class_body();
        let span = self.span_from(start);
        Statement::ClassDecl(ClassDecl { name, super_class, body, span })
    }

    pub(crate) fn parse_class_body(&mut self) -> ClassBody {
        let start = self.cur().span;
        self.expect(JsTokenKind::LBrace, "{");
        let mut body = Vec::new();
        while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
            let r#static = self.eat(JsTokenKind::Static);
            if self.at_any(&[JsTokenKind::Star, JsTokenKind::Identifier, JsTokenKind::LBracket, JsTokenKind::String, JsTokenKind::Number]) {
                let is_generator = self.eat(JsTokenKind::Star);
                let computed = self.at(JsTokenKind::LBracket);
                let key = self.parse_prop_key();
                if self.eat(JsTokenKind::LParen) {
                    self.pos -= 1;
                    let params = self.parse_params();
                    let func_body = self.parse_block();
                    let span = self.span_from(start);
                    let function = FunctionExpr {
                        id: None,
                        params,
                        body: func_body,
                        generator: is_generator,
                        r#async: false,
                        span,
                    };
                    let kind = if key_is_constructor(&key) { MethodKind::Constructor }
                        else if self.eat(JsTokenKind::Get) { MethodKind::Get }
                        else if self.eat(JsTokenKind::Set) { MethodKind::Set }
                        else { MethodKind::Method };
                    body.push(ClassMember::Method(ClassMethod {
                        key,
                        kind,
                        function,
                        computed,
                        r#static,
                        span,
                    }));
                } else {
                    let value = if self.eat(JsTokenKind::Assign) {
                        Some(self.parse_assignment_expr())
                    } else {
                        None
                    };
                    self.eat_semicolon();
                    let span = self.span_from(start);
                    body.push(ClassMember::Property(ClassProperty {
                        key,
                        value,
                        computed,
                        r#static,
                        span,
                    }));
                }
            } else if self.at(JsTokenKind::Semicolon) {
                self.advance();
            } else {
                self.error("unexpected token in class body");
                self.advance();
            }
        }
        self.expect(JsTokenKind::RBrace, "}");
        let span = self.span_from(start);
        ClassBody { body, span }
    }

    pub(crate) fn parse_prop_key(&mut self) -> PropKey {
        match self.kind() {
            JsTokenKind::LBracket => {
                self.advance();
                let expr = self.parse_assignment_expr();
                self.expect(JsTokenKind::RBracket, "]");
                PropKey::Computed(Box::new(expr))
            }
            JsTokenKind::String => {
                let tok = self.advance();
                PropKey::String(tok.raw)
            }
            JsTokenKind::Number => {
                let tok = self.advance();
                PropKey::String(tok.raw)
            }
            _ => PropKey::Ident(self.advance().raw),
        }
    }

    fn parse_expr_stmt(&mut self) -> Statement {
        let start = self.cur().span;
        let expr = self.parse_expression();
        self.eat_semicolon();
        let span = self.span_from(start);
        Statement::Expr(ExprStmt { expr, span })
    }

    fn parse_binding_ident(&mut self) -> String {
        if self.at(JsTokenKind::Identifier) {
            self.advance().raw
        } else {
            self.error("expected an identifier");
            String::new()
        }
    }
}

fn key_is_constructor(key: &PropKey) -> bool {
    matches!(key, PropKey::Ident(name) if name == "constructor")
}
