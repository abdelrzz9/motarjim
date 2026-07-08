//! Best-effort semantic analysis: scope tracking, duplicate declarations, const reassignment checks,
//! closure capture tracking, and import validation.
//!
//! # Capture analysis
//!
//! Captures are stored in the [`SemanticAnalyzer::captures`] side table as a
//! `HashMap<SourceSpan, Vec<String>>` keyed by a function's span.  Each entry
//! lists the free variable names that the function reads from or writes to
//! in an enclosing non-global scope.
//!
//! Callers retrieve capture info via [`SemanticAnalyzer::captures_for`].

use std::collections::HashMap;

use motarjim_span::SourceSpan;

use crate::ast::expr::*;
use crate::ast::pat::*;
use crate::ast::program::{Program, SourceType};
use crate::ast::stmt::*;
use crate::diagnostics::{JsDiagnostic, JsDiagnosticCode};
use crate::semantic::scope::{Binding, ScopeStack};
use crate::visitor::{walk_expression, walk_statement, Visitor};

mod scope;

#[derive(Debug)]
pub struct SemanticAnalyzer {
    scopes: ScopeStack,
    diagnostics: Vec<JsDiagnostic>,
    loop_depth: u32,
    function_depth: u32,
    function_async_stack: Vec<bool>,
    strict_mode: bool,
    class_has_superclass_stack: Vec<bool>,
    in_method_body: u32,
    captures: HashMap<SourceSpan, Vec<String>>,
    function_scope_bases: Vec<usize>,
    pending_captures_stack: Vec<Vec<(usize, String)>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            diagnostics: Vec::new(),
            loop_depth: 0,
            function_depth: 0,
            function_async_stack: Vec::new(),
            strict_mode: false,
            class_has_superclass_stack: Vec::new(),
            in_method_body: 0,
            captures: HashMap::new(),
            function_scope_bases: Vec::new(),
            pending_captures_stack: Vec::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Vec<JsDiagnostic> {
        self.strict_mode = program.source_type == SourceType::Module
            || program.body.first().is_some_and(is_use_strict_directive);
        self.visit_program(program);
        std::mem::take(&mut self.diagnostics)
    }

    /// Returns the list of captured free variables for the function whose span
    /// matches `span`, or `None` if the function is not known.
    pub fn captures_for(&self, span: &SourceSpan) -> Option<&[String]> {
        self.captures.get(span).map(|v| v.as_slice())
    }

    /// Iterate over all (span, captures) entries in the captures side table.
    pub fn captures_iter(&self) -> impl Iterator<Item = (&SourceSpan, &Vec<String>)> {
        self.captures.iter()
    }

    // ── declaration helpers ──────────────────────────────────────────────

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

    fn declare_in_function_scope(&mut self, name: &str, kind: VarKind, span: SourceSpan) {
        if name.is_empty() {
            return;
        }
        if let Some(existing) = self.scopes.declare_in_function_scope(name, kind, span) {
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
            Some(Binding {
                kind: VarKind::Const,
                ..
            }) => {
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
        let kind = if self.strict_mode {
            VarKind::Let
        } else {
            VarKind::Var
        };
        for param in params {
            self.declare_pat(&param.pat, kind);
            if let Some(default) = &param.default {
                self.visit_expression(default);
            }
        }
    }

    fn declare_pat(&mut self, pat: &Pattern, kind: VarKind) {
        self.declare_pat_inner(pat, kind, false);
    }

    fn declare_pat_hoisted(&mut self, pat: &Pattern, kind: VarKind) {
        self.declare_pat_inner(pat, kind, true);
    }

    fn declare_pat_inner(&mut self, pat: &Pattern, kind: VarKind, hoist: bool) {
        match pat {
            Pattern::Ident(name, span) => {
                if hoist {
                    self.declare_in_function_scope(name, kind, *span);
                } else {
                    self.declare(name, kind, *span);
                }
            }
            Pattern::Object(obj) => {
                for prop in &obj.props {
                    match prop {
                        ObjectPatProp::KeyValue { value, .. } => {
                            self.declare_pat_inner(value, kind, hoist);
                        }
                        ObjectPatProp::Shorthand { name, span } => {
                            if hoist {
                                self.declare_in_function_scope(name, kind, *span);
                            } else {
                                self.declare(name, kind, *span);
                            }
                        }
                        ObjectPatProp::Rest(pat, _) => self.declare_pat_inner(pat, kind, hoist),
                    }
                }
            }
            Pattern::Array(arr) => {
                for el in arr.elements.iter().flatten() {
                    self.declare_pat_inner(el, kind, hoist);
                }
            }
            Pattern::Assign(assign) => self.declare_pat_inner(&assign.left, kind, hoist),
            Pattern::Rest(pat) => self.declare_pat_inner(pat, kind, hoist),
            Pattern::Default(default) => self.declare_pat_inner(&default.left, kind, hoist),
            Pattern::Member(_) => {}
        }
    }

    // ── scope management ────────────────────────────────────────────────

    fn push_function_scope(&mut self, is_async: bool) {
        self.scopes.push_function();
        self.function_depth += 1;
        self.function_async_stack.push(is_async);
        self.function_scope_bases.push(self.scopes.len() - 1);
        self.pending_captures_stack.push(Vec::new());
    }

    fn record_capture(&mut self, name: &str, depth: usize) {
        if let Some(top) = self.pending_captures_stack.last_mut() {
            top.push((depth, name.to_string()));
        }
    }

    // ── class context helpers ───────────────────────────────────────────

    fn visit_class_body(&mut self, body: &ClassBody, has_superclass: bool) {
        self.class_has_superclass_stack.push(has_superclass);
        for member in &body.body {
            match member {
                ClassMember::Method(m) => {
                    if let Some(default) =
                        &m.function.params.iter().find_map(|p| p.default.as_ref())
                    {
                        self.visit_expression(default);
                    }
                    self.in_method_body += 1;
                    self.push_function_scope(m.function.r#async);
                    let kind = if self.strict_mode {
                        VarKind::Let
                    } else {
                        VarKind::Var
                    };
                    for param in &m.function.params {
                        self.declare_pat(&param.pat, kind);
                        if let Some(default) = &param.default {
                            self.visit_expression(default);
                        }
                    }
                    for stmt in &m.function.body.body {
                        self.visit_statement(stmt);
                    }
                    // Pop and store captures for this method
                    self.scopes.pop();
                    self.function_depth -= 1;
                    self.function_async_stack.pop();
                    let func_base = self
                        .function_scope_bases
                        .pop()
                        .expect("scope base mismatch");
                    let func_captures = self
                        .pending_captures_stack
                        .pop()
                        .expect("captures stack mismatch");
                    let mut names: Vec<String> = func_captures
                        .into_iter()
                        .filter(|(depth, _)| *depth < func_base)
                        .map(|(_, n)| n)
                        .collect();
                    names.sort();
                    names.dedup();
                    self.captures.insert(m.function.span, names);
                    self.in_method_body -= 1;
                }
                ClassMember::Property(p) => {
                    if let Some(value) = &p.value {
                        self.visit_expression(value);
                    }
                }
            }
        }
        self.class_has_superclass_stack.pop();
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
                    if decl.kind == VarKind::Var {
                        self.declare_pat_hoisted(&declarator.name, decl.kind);
                    } else {
                        self.declare_pat(&declarator.name, decl.kind);
                    }
                }
            }
            Statement::FunctionDecl(func) => {
                self.declare_pat(&func.name, VarKind::Var);
                self.push_function_scope(func.r#async);
                self.declare_params(&func.params);
                for stmt in &func.body.body {
                    self.visit_statement(stmt);
                }
                // Pop and store captures for this function
                self.scopes.pop();
                self.function_depth -= 1;
                self.function_async_stack.pop();
                let func_base = self
                    .function_scope_bases
                    .pop()
                    .expect("scope base mismatch");
                let func_captures = self
                    .pending_captures_stack
                    .pop()
                    .expect("captures stack mismatch");
                let mut names: Vec<String> = func_captures
                    .into_iter()
                    .filter(|(depth, _)| *depth < func_base)
                    .map(|(_, n)| n)
                    .collect();
                names.sort();
                names.dedup();
                self.captures.insert(func.span, names);
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
                                if decl.kind == VarKind::Var {
                                    self.declare_pat_hoisted(&declarator.name, decl.kind);
                                } else {
                                    self.declare_pat(&declarator.name, decl.kind);
                                }
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
                        JsDiagnostic::error(
                            JsDiagnosticCode::JS_ILLEGAL_BREAK,
                            "'break' outside of loop",
                        )
                        .with_span(*span),
                    );
                }
            }
            Statement::Continue(span) => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(
                        JsDiagnostic::error(
                            JsDiagnosticCode::JS_ILLEGAL_CONTINUE,
                            "'continue' outside of loop",
                        )
                        .with_span(*span),
                    );
                }
            }
            Statement::Return(ret) => {
                if self.function_depth == 0 {
                    self.diagnostics.push(
                        JsDiagnostic::error(
                            JsDiagnosticCode::JS_ILLEGAL_RETURN,
                            "'return' outside of function",
                        )
                        .with_span(ret.span),
                    );
                }
                if let Some(arg) = &ret.argument {
                    self.visit_expression(arg);
                }
            }
            Statement::ClassDecl(class) => {
                self.declare_pat(&class.name, VarKind::Let);
                if let Some(super_class) = &class.super_class {
                    self.visit_expression(super_class);
                }
                let has_super = class.super_class.is_some();
                self.visit_class_body(&class.body, has_super);
            }
            Statement::Import(import) => {
                self.validate_import(import);
            }
            _ => walk_statement(self, stmt),
        }
    }

    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name, span) => {
                self.check_reference(name, *span);
                if !self.function_scope_bases.is_empty() {
                    if let Some((_binding, depth)) = self.scopes.lookup_with_depth(name) {
                        if !is_known_global(name) {
                            self.record_capture(name, depth);
                        }
                    }
                }
            }
            Expression::Assignment(assign) => {
                if let Expression::Identifier(name, span) = assign.target.as_ref() {
                    self.check_assignment_target(name, *span);
                    if !self.function_scope_bases.is_empty() {
                        if let Some((_binding, depth)) = self.scopes.lookup_with_depth(name) {
                            if !is_known_global(name) {
                                self.record_capture(name, depth);
                            }
                        }
                    }
                } else {
                    self.visit_expression(&assign.target);
                }
                self.visit_expression(&assign.value);
            }
            Expression::Unary(unary) if unary.op == UnaryOp::Delete => {
                self.visit_expression(&unary.argument);
            }
            Expression::Function(func) => {
                self.push_function_scope(func.r#async);
                self.declare_params(&func.params);
                for stmt in &func.body.body {
                    self.visit_statement(stmt);
                }
                // Pop and store captures for this function expr
                self.scopes.pop();
                self.function_depth -= 1;
                self.function_async_stack.pop();
                let func_base = self
                    .function_scope_bases
                    .pop()
                    .expect("scope base mismatch");
                let func_captures = self
                    .pending_captures_stack
                    .pop()
                    .expect("captures stack mismatch");
                let mut names: Vec<String> = func_captures
                    .into_iter()
                    .filter(|(depth, _)| *depth < func_base)
                    .map(|(_, n)| n)
                    .collect();
                names.sort();
                names.dedup();
                self.captures.insert(func.span, names);
            }
            Expression::Arrow(arrow) => {
                self.push_function_scope(arrow.r#async);
                self.declare_params(&arrow.params);
                match &arrow.body {
                    ArrowBody::Block(block) => {
                        for stmt in &block.body {
                            self.visit_statement(stmt);
                        }
                    }
                    ArrowBody::Expr(body_expr) => self.visit_expression(body_expr),
                }
                // Pop and store captures for this arrow
                self.scopes.pop();
                self.function_depth -= 1;
                self.function_async_stack.pop();
                let func_base = self
                    .function_scope_bases
                    .pop()
                    .expect("scope base mismatch");
                let func_captures = self
                    .pending_captures_stack
                    .pop()
                    .expect("captures stack mismatch");
                let mut names: Vec<String> = func_captures
                    .into_iter()
                    .filter(|(depth, _)| *depth < func_base)
                    .map(|(_, n)| n)
                    .collect();
                names.sort();
                names.dedup();
                self.captures.insert(arrow.span, names);
            }
            Expression::Await(await_expr) => {
                let in_async = self.function_async_stack.last().copied().unwrap_or(false);
                if self.function_depth == 0 || !in_async {
                    self.diagnostics.push(
                        JsDiagnostic::error(
                            JsDiagnosticCode::JS_ILLEGAL_AWAIT,
                            "'await' outside of async function",
                        )
                        .with_span(await_expr.span),
                    );
                }
                self.visit_expression(&await_expr.argument);
            }
            Expression::Yield(yield_expr) => {
                if self.function_depth == 0 {
                    self.diagnostics.push(
                        JsDiagnostic::error(
                            JsDiagnosticCode::JS_ILLEGAL_YIELD,
                            "'yield' outside of generator",
                        )
                        .with_span(yield_expr.span),
                    );
                }
                if let Some(arg) = &yield_expr.argument {
                    self.visit_expression(arg);
                }
            }
            Expression::Super(span) => {
                let has_super = self
                    .class_has_superclass_stack
                    .last()
                    .copied()
                    .unwrap_or(false);
                if self.in_method_body == 0 || !has_super {
                    let msg = if self.in_method_body == 0 {
                        "'super' outside of class method"
                    } else {
                        "'super' in class without extends"
                    };
                    self.diagnostics.push(
                        JsDiagnostic::error(JsDiagnosticCode::JS_ILLEGAL_SUPER, msg)
                            .with_span(*span),
                    );
                }
            }
            Expression::ClassExpr(class_expr) => {
                if let Some(super_class) = &class_expr.super_class {
                    self.visit_expression(super_class);
                }
                let has_super = class_expr.super_class.is_some();
                self.visit_class_body(&class_expr.body, has_super);
            }
            _ => walk_expression(self, expr),
        }
    }
}

impl SemanticAnalyzer {
    // ── import validation (single-file) ─────────────────────────────────

    fn validate_import(&mut self, import: &ImportDecl) {
        let mut names_in_use: Vec<&str> = Vec::new();

        // Check default import: `import x from 'y'`
        if let Some(ref default) = import.default {
            if default == "default" {
                self.diagnostics.push(
                    JsDiagnostic::error(
                        JsDiagnosticCode::JS_IMPORT_ERROR,
                        "cannot use 'default' as a local binding name",
                    )
                    .with_span(import.span),
                );
            }
            names_in_use.push(default.as_str());
        }
        if import.namespace.is_some() {
            if let Some(ref ns) = import.namespace {
                names_in_use.push(ns.as_str());
            }
        }
        for spec in &import.named {
            if &spec.imported == "default" {
                self.diagnostics.push(
                    JsDiagnostic::error(
                        JsDiagnosticCode::JS_IMPORT_ERROR,
                        "use default-import syntax instead of `import { default }`",
                    )
                    .with_span(spec.span),
                );
            }
            if names_in_use.contains(&spec.local.as_str()) {
                self.diagnostics.push(
                    JsDiagnostic::error(
                        JsDiagnosticCode::JS_DUPLICATE_DECLARATION,
                        format!("'{}' is already bound by this import", spec.local),
                    )
                    .with_span(spec.span),
                );
            }
            names_in_use.push(&spec.local);
        }

        // Self-import detection (same source) – best-effort, single file.
        if import.source == "*self*" || import.source.is_empty() {
            self.diagnostics.push(
                JsDiagnostic::error(
                    JsDiagnosticCode::JS_IMPORT_ERROR,
                    "self-import is not allowed",
                )
                .with_span(import.span),
            );
        }
    }
}

fn is_use_strict_directive(stmt: &Statement) -> bool {
    if let Statement::Expr(expr_stmt) = stmt {
        if let Expression::String(s) = &expr_stmt.expr {
            return s.value == "use strict";
        }
    }
    false
}

fn is_known_global(name: &str) -> bool {
    matches!(
        name,
        "window"
            | "document"
            | "console"
            | "Math"
            | "JSON"
            | "Array"
            | "Object"
            | "String"
            | "Number"
            | "Boolean"
            | "Promise"
            | "Map"
            | "Set"
            | "Symbol"
            | "Error"
            | "RegExp"
            | "Date"
            | "parseInt"
            | "parseFloat"
            | "isNaN"
            | "isFinite"
            | "setTimeout"
            | "setInterval"
            | "clearTimeout"
            | "clearInterval"
            | "fetch"
            | "alert"
            | "confirm"
            | "prompt"
            | "localStorage"
            | "sessionStorage"
            | "navigator"
            | "location"
            | "history"
            | "NaN"
            | "Infinity"
            | "globalThis"
            | "requestAnimationFrame"
            | "self"
            | "Event"
            | "CustomEvent"
            | "Element"
            | "Node"
            | "HTMLElement"
            | "undefined"
            | "null"
            | "true"
            | "false"
            | "eval"
            | "encodeURI"
            | "encodeURIComponent"
            | "decodeURI"
            | "decodeURIComponent"
            | "Intl"
            | "Proxy"
            | "Reflect"
            | "WeakMap"
            | "WeakSet"
            | "WeakRef"
            | "FinalizationRegistry"
            | "Atomics"
            | "SharedArrayBuffer"
            | "BigInt"
            | "BigInt64Array"
            | "BigUint64Array"
            | "Float32Array"
            | "Float64Array"
            | "Int8Array"
            | "Int16Array"
            | "Int32Array"
            | "Uint8Array"
            | "Uint8ClampedArray"
            | "Uint16Array"
            | "Uint32Array"
            | "ArrayBuffer"
            | "DataView"
            | "Generator"
            | "GeneratorFunction"
            | "AsyncFunction"
            | "AsyncGenerator"
            | "AsyncGeneratorFunction"
    )
}
