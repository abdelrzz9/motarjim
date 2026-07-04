//! Best-effort semantic analysis: scope tracking, duplicate declarations, const reassignment checks.

use crate::ast::expr::*;
use crate::ast::pat::*;
use crate::ast::program::Program;
use crate::ast::stmt::*;
use crate::diagnostics::{JsDiagnostic, JsDiagnosticCode};
use crate::semantic::scope::{Binding, ScopeStack};
use crate::visitor::{walk_expression, walk_expression_mut, walk_statement_mut, Visitor, VisitorMut};

mod scope;

#[derive(Debug)]
pub struct SemanticAnalyzer {
    scopes: ScopeStack,
    diagnostics: Vec<JsDiagnostic>,
    loop_depth: u32,
    function_depth: u32,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            diagnostics: Vec::new(),
            loop_depth: 0,
            function_depth: 0,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Vec<JsDiagnostic> {
        self.visit_program(program);
        std::mem::take(&mut self.diagnostics)
    }

    fn declare(&mut self, name: &str, kind: VarKind, span: SourceSpan) {
        if name.is_empty() {
            return;
        }
        if let Some(existing) = self.scopes.declare(name, kind, span) {
            self.diagnostics.push(
                JsDiagnostic::error(
                    JsDiagnosticCode::JS_DUPLICATE_DECLARATION,
                    format!("'{name}' is already declared in this scope"),
                )
                .with_span(span)
                .with_note(format!(
                    "previous declaration at byte offset {}",
                    existing.span.start.offset
                )),
            );
        }
    }

    fn check_reference(&mut self, name: &str, span: SourceSpan) {
        if self.scopes.lookup(name).is_none() && !is_known_global(name) {
            self.diagnostics.push(
                JsDiagnostic::warning(
                    JsDiagnosticCode::JS_UNDECLARED_VARIABLE,
                    format!("'{name}' is not declared"),
                )
                .with_span(span),
            );
        }
    }

    fn check_assignment_target(&mut self, name: &str, span: SourceSpan) {
        match self.scopes.lookup(name) {
            Some(Binding { kind: VarKind::Const, .. }) => {
                self.diagnostics.push(
                    JsDiagnostic::error(
                        JsDiagnosticCode::JS_ASSIGN_TO_CONST,
                        format!("cannot assign to '{name}' because it is a constant"),
                    )
                    .with_span(span),
                );
            }
            Some(_) => {}
            None => self.check_reference(name, span),
        }
    }

    fn declare_params(&mut self, params: &[Param]) {
        for param in params {
            self.declare_pat(&param.pat, VarKind::Var);
            if let Some(default) = &param.default {
                self.visit_expression(default);
            }
        }
    }

    fn declare_pat(&mut self, pat: &Pattern, kind: VarKind) {
        match pat {
            Pattern::Ident(name, span) => self.declare(name, kind, *span),
            Pattern::Object(obj) => {
                for prop in &obj.props {
                    match prop {
                        ObjectPatProp::KeyValue { value, .. } => self.declare_pat(value, kind),
                        ObjectPatProp::Shorthand { name, span } => self.declare(name, kind, *span),
                        ObjectPatProp::Rest(pat, _) => self.declare_pat(pat, kind),
                    }
                }
            }
            Pattern::Array(arr) => {
                for el in arr.elements.iter().flatten() {
                    self.declare_pat(el, kind);
                }
            }
            Pattern::Assign(assign) => self.declare_pat(&assign.left, kind),
            Pattern::Rest(pat) => self.declare_pat(pat, kind),
            Pattern::Default(default) => self.declare_pat(&default.left, kind),
            Pattern::Member(_) => {}
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor for SemanticAnalyzer {
    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl(decl) => {
                for declarator in &decl.declarators {
                    if let Some(init) = &declarator.init {
                        self.visit_expression(init);
                    }
                    self.declare_pat(&declarator.name, decl.kind);
                }
            }
            Statement::FunctionDecl(func) => {
                self.declare_pat(&func.name, VarKind::Var);
                self.push_function_scope();
                self.declare_params(&func.params);
                for stmt in &func.body.body {
                    self.visit_statement(stmt);
                }
                self.pop_scope();
            }
            Statement::Block(block) => {
                self.scopes.push();
                for stmt in &block.body {
                    self.visit_statement(stmt);
                }
                self.scopes.pop();
            }
            Statement::For(for_stmt) => {
                self.scopes.push();
                if let Some(init) = &for_stmt.init {
                    match init {
                        ForInit::VarDecl(decl) => {
                            for declarator in &decl.declarators {
                                if let Some(init) = &declarator.init {
                                    self.visit_expression(init);
                                }
                                self.declare_pat(&declarator.name, decl.kind);
                            }
                        }
                        ForInit::Expr(expr) => self.visit_expression(expr),
                    }
                }
                if let Some(test) = &for_stmt.test {
                    self.visit_expression(test);
                }
                if let Some(update) = &for_stmt.update {
                    self.visit_expression(update);
                }
                self.loop_depth += 1;
                self.visit_statement(&for_stmt.body);
                self.loop_depth -= 1;
                self.scopes.pop();
            }
            Statement::ForOf(for_of) => {
                self.visit_expression(&for_of.right);
                self.scopes.push();
                self.declare_pat(&for_of.left, VarKind::Let);
                self.loop_depth += 1;
                self.visit_statement(&for_of.body);
                self.loop_depth -= 1;
                self.scopes.pop();
            }
            Statement::ForIn(for_in) => {
                self.visit_expression(&for_in.right);
                self.scopes.push();
                self.declare_pat(&for_in.left, VarKind::Let);
                self.loop_depth += 1;
                self.visit_statement(&for_in.body);
                self.loop_depth -= 1;
                self.scopes.pop();
            }
            Statement::While(s) | Statement::DoWhile(s) if false => {}
            Statement::While(while_stmt) => {
                self.visit_expression(&while_stmt.test);
                self.loop_depth += 1;
                self.visit_statement(&while_stmt.body);
                self.loop_depth -= 1;
            }
            Statement::DoWhile(do_while) => {
                self.loop_depth += 1;
                self.visit_statement(&do_while.body);
                self.loop_depth -= 1;
                self.visit_expression(&do_while.test);
            }
            Statement::Break(span) => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(
                        JsDiagnostic::error(JsDiagnosticCode::JS_ILLEGAL_BREAK, "'break' outside of loop")
                            .with_span(*span),
                    );
                }
            }
            Statement::Continue(span) => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(
                        JsDiagnostic::error(JsDiagnosticCode::JS_ILLEGAL_CONTINUE, "'continue' outside of loop")
                            .with_span(*span),
                    );
                }
            }
            Statement::Return(ret) => {
                if self.function_depth == 0 {
                    self.diagnostics.push(
                        JsDiagnostic::error(JsDiagnosticCode::JS_ILLEGAL_RETURN, "'return' outside of function")
                            .with_span(ret.span),
                    );
                }
                if let Some(arg) = &ret.argument {
                    self.visit_expression(arg);
                }
            }
            _ => walk_statement(self, stmt),
        }
    }

    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name, span) => self.check_reference(name, *span),
            Expression::Assignment(assign) => {
                if let Expression::Identifier(name, span) = assign.target.as_ref() {
                    self.check_assignment_target(name, *span);
                } else {
                    self.visit_expression(&assign.target);
                }
                self.visit_expression(&assign.value);
            }
            Expression::Unary(unary) if unary.op == UnaryOp::Delete => {
                self.visit_expression(&unary.argument);
            }
            Expression::Function(func) => {
                self.push_function_scope();
                self.declare_params(&func.params);
                for stmt in &func.body.body {
                    self.visit_statement(stmt);
                }
                self.pop_scope();
            }
            Expression::Arrow(arrow) => {
                self.push_function_scope();
                self.declare_params(&arrow.params);
                match &arrow.body {
                    ArrowBody::Block(block) => {
                        for stmt in &block.body {
                            self.visit_statement(stmt);
                        }
                    }
                    ArrowBody::Expr(expr) => self.visit_expression(expr),
                }
                self.pop_scope();
            }
            Expression::Await(await_expr) => {
                if self.function_depth == 0 {
                    self.diagnostics.push(
                        JsDiagnostic::error(JsDiagnosticCode::JS_ILLEGAL_AWAIT, "'await' outside of async function")
                            .with_span(await_expr.span),
                    );
                }
                self.visit_expression(&await_expr.argument);
            }
            Expression::Yield(yield_expr) => {
                if self.function_depth == 0 {
                    self.diagnostics.push(
                        JsDiagnostic::error(JsDiagnosticCode::JS_ILLEGAL_YIELD, "'yield' outside of generator")
                            .with_span(yield_expr.span),
                    );
                }
                if let Some(arg) = &yield_expr.argument {
                    self.visit_expression(arg);
                }
            }
            Expression::Super(span) => {
                self.diagnostics.push(
                    JsDiagnostic::error(JsDiagnosticCode::JS_ILLEGAL_RETURN, "'super' outside of class")
                        .with_span(*span),
                );
            }
            _ => walk_expression(self, expr),
        }
    }
}

impl SemanticAnalyzer {
    fn push_function_scope(&mut self) {
        self.scopes.push();
        self.function_depth += 1;
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        if self.function_depth > 0 {
            self.function_depth -= 1;
        }
    }
}

fn is_known_global(name: &str) -> bool {
    matches!(
        name,
        "window" | "document" | "console" | "Math" | "JSON" | "Array"
            | "Object" | "String" | "Number" | "Boolean" | "Promise"
            | "Map" | "Set" | "Symbol" | "Error" | "RegExp" | "Date"
            | "parseInt" | "parseFloat" | "isNaN" | "isFinite"
            | "setTimeout" | "setInterval" | "clearTimeout" | "clearInterval"
            | "fetch" | "alert" | "confirm" | "prompt"
            | "localStorage" | "sessionStorage" | "navigator" | "location"
            | "history" | "NaN" | "Infinity" | "globalThis"
            | "requestAnimationFrame" | "self" | "Event" | "CustomEvent"
            | "Element" | "Node" | "HTMLElement" | "console"
            | "undefined" | "null" | "true" | "false"
            | "eval" | "isNaN" | "isFinite" | "encodeURI"
            | "encodeURIComponent" | "decodeURI" | "decodeURIComponent"
            | "Intl" | "Proxy" | "Reflect" | "WeakMap" | "WeakSet"
            | "WeakRef" | "FinalizationRegistry" | "Atomics"
            | "SharedArrayBuffer" | "BigInt" | "BigInt64Array"
            | "BigUint64Array" | "Float32Array" | "Float64Array"
            | "Int8Array" | "Int16Array" | "Int32Array" | "Uint8Array"
            | "Uint8ClampedArray" | "Uint16Array" | "Uint32Array"
            | "ArrayBuffer" | "DataView" | "Generator" | "GeneratorFunction"
            | "AsyncFunction" | "AsyncGenerator" | "AsyncGeneratorFunction"
    )
}
