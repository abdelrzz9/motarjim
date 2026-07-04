//! Best-effort semantic analysis: scope tracking, duplicate declarations,
//! `const` reassignment, and undeclared-variable references.
//!
//! # Known limitations
//!
//! Scoping here approximates every block (`if`/`for`/`while`/`{ }`) as
//! introducing its own scope for `var` as well as `let`/`const`. Real
//! JavaScript hoists `var` (and function declarations) to the nearest
//! enclosing function, so code that declares a `var` inside a block and
//! reads it afterwards in the same function may trigger a spurious
//! "undeclared variable" warning. Because that check is a warning (not an
//! error) and is intended as an editor hint rather than a hard gate, this
//! trade-off keeps the analyzer simple without blocking compilation.

use std::collections::HashMap;

use motarjim_diag::{codes, Diagnostic, DiagnosticBag, Severity, SourceSpan};

use crate::ast::{ArrowBody, Expression, Ident, Program, Statement, VarKind};
use crate::visitor::{walk_expression, Visitor};

/// A single name bound in a [`Scope`].
#[derive(Debug, Clone, Copy)]
struct Binding {
    /// The declaration kind (`var`, `let`, or `const`).
    kind: VarKind,
    /// Where the binding was introduced; reported back in duplicate-declaration notes.
    span: SourceSpan,
}

/// A lexical scope: a set of name bindings, plus a link to the enclosing
/// scope via the analyzer's scope stack.
#[derive(Debug, Default)]
struct Scope {
    /// Bindings introduced directly in this scope.
    bindings: HashMap<String, Binding>,
}

/// Walks a [`Program`] tracking scopes, reporting:
///
/// - duplicate `let`/`const` declarations in the same scope,
/// - assignments to `const` bindings, and
/// - references to names with no matching declaration (excluding a small
///   allowlist of common browser/JS globals).
///
/// # Example
///
/// ```rust
/// use motarjim_js::{JsParser, SemanticAnalyzer};
///
/// let mut parser = JsParser::new("const x = 1; x = 2;");
/// let program = parser.parse().expect("valid syntax");
/// let diagnostics = SemanticAnalyzer::new().analyze(&program);
/// assert_eq!(diagnostics.len(), 1);
/// ```
#[derive(Debug)]
pub struct SemanticAnalyzer {
    /// The scope stack; the last entry is the innermost (current) scope.
    scopes: Vec<Scope>,
    /// Diagnostics collected so far.
    diagnostics: DiagnosticBag,
}

impl SemanticAnalyzer {
    /// Creates a new analyzer with a single, empty top-level scope.
    #[must_use]
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
            diagnostics: DiagnosticBag::new(),
        }
    }

    /// Analyzes `program` and returns every diagnostic found.
    pub fn analyze(&mut self, program: &Program) -> Vec<Diagnostic> {
        self.visit_program(program);
        std::mem::take(&mut self.diagnostics).into_diagnostics()
    }

    /// Pushes a new, empty scope.
    fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    /// Pops the innermost scope.
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Declares `name` in the innermost scope, reporting a duplicate
    /// declaration if a `let`/`const` binding with the same name already
    /// exists there. Declaring under `var` never reports a duplicate,
    /// matching real JavaScript's re-declaration rules.
    fn declare(&mut self, name: &str, kind: VarKind, span: SourceSpan) {
        if name.is_empty() {
            // The parser already reported the missing-identifier error.
            return;
        }
        let scope = self
            .scopes
            .last_mut()
            .expect("at least one scope is always present");
        if kind != VarKind::Var {
            if let Some(existing) = scope.bindings.get(name) {
                self.diagnostics.push(
                    Diagnostic::new(
                        Severity::Error,
                        codes::JS_DUPLICATE_DECLARATION,
                        format!("'{name}' is already declared in this scope"),
                    )
                    .with_span(span)
                    .with_note(format!(
                        "previous declaration of '{name}' at byte offset {}",
                        existing.span.start.offset
                    )),
                );
            }
        }
        scope
            .bindings
            .insert(name.to_string(), Binding { kind, span });
    }

    /// Looks up `name` in the current scope chain, innermost first.
    fn lookup(&self, name: &str) -> Option<Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(name).copied())
    }

    /// Reports an undeclared-variable warning for `id` unless it resolves to
    /// a scope binding or a known global.
    fn check_reference(&mut self, id: &Ident) {
        if self.lookup(&id.name).is_none() && !is_known_global(&id.name) {
            self.diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    codes::JS_UNDECLARED_VARIABLE,
                    format!("'{}' is not declared", id.name),
                )
                .with_span(id.span),
            );
        }
    }

    /// Reports an assignment to a `const` binding, or falls back to the
    /// undeclared-variable check when `id` has no binding at all.
    fn check_assignment_target(&mut self, id: &Ident) {
        match self.lookup(&id.name) {
            Some(binding) if binding.kind == VarKind::Const => {
                self.diagnostics.push(
                    Diagnostic::new(
                        Severity::Error,
                        codes::JS_ASSIGN_TO_CONST,
                        format!("cannot assign to '{}' because it is a constant", id.name),
                    )
                    .with_span(id.span),
                );
            }
            Some(_) => {}
            None => self.check_reference(id),
        }
    }

    /// Declares every parameter of a function-like callee in the current
    /// (already-pushed) scope, visiting default value expressions first.
    fn declare_params(&mut self, params: &[crate::ast::Param]) {
        for param in params {
            if let Some(default) = &param.default {
                self.visit_expression(default);
            }
            self.declare(&param.name, VarKind::Var, param.span);
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
                    self.declare(&declarator.name, decl.kind, declarator.span);
                }
            }
            Statement::FunctionDecl(func) => {
                self.declare(&func.name, VarKind::Var, func.span);
                self.push_scope();
                self.declare_params(&func.params);
                for stmt in &func.body.body {
                    self.visit_statement(stmt);
                }
                self.pop_scope();
            }
            Statement::Block(block) => {
                self.push_scope();
                for stmt in &block.body {
                    self.visit_statement(stmt);
                }
                self.pop_scope();
            }
            Statement::For(for_stmt) => {
                self.push_scope();
                if let Some(init) = &for_stmt.init {
                    match init {
                        crate::ast::ForInit::VarDecl(decl) => {
                            for declarator in &decl.declarators {
                                if let Some(init) = &declarator.init {
                                    self.visit_expression(init);
                                }
                                self.declare(&declarator.name, decl.kind, declarator.span);
                            }
                        }
                        crate::ast::ForInit::Expr(expr) => self.visit_expression(expr),
                    }
                }
                if let Some(test) = &for_stmt.test {
                    self.visit_expression(test);
                }
                if let Some(update) = &for_stmt.update {
                    self.visit_expression(update);
                }
                self.visit_statement(&for_stmt.body);
                self.pop_scope();
            }
            Statement::ForOf(for_of) => {
                self.visit_expression(&for_of.right);
                self.push_scope();
                self.declare(
                    &for_of.left,
                    for_of.decl_kind.unwrap_or(VarKind::Var),
                    for_of.span,
                );
                self.visit_statement(&for_of.body);
                self.pop_scope();
            }
            Statement::ForIn(for_in) => {
                self.visit_expression(&for_in.right);
                self.push_scope();
                self.declare(
                    &for_in.left,
                    for_in.decl_kind.unwrap_or(VarKind::Var),
                    for_in.span,
                );
                self.visit_statement(&for_in.body);
                self.pop_scope();
            }
            _ => crate::visitor::walk_statement(self, stmt),
        }
    }

    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(id) => self.check_reference(id),
            Expression::Assignment(assign) => {
                if let Expression::Identifier(id) = assign.target.as_ref() {
                    self.check_assignment_target(id);
                } else {
                    self.visit_expression(&assign.target);
                }
                self.visit_expression(&assign.value);
            }
            Expression::Function(func) => {
                self.push_scope();
                self.declare_params(&func.params);
                for stmt in &func.body.body {
                    self.visit_statement(stmt);
                }
                self.pop_scope();
            }
            Expression::Arrow(arrow) => {
                self.push_scope();
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
            _ => walk_expression(self, expr),
        }
    }
}

/// Returns `true` for identifiers that name common browser/JavaScript
/// globals, so referencing them never triggers an undeclared-variable
/// warning.
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
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::JsParser;

    fn analyze(src: &str) -> Vec<Diagnostic> {
        let mut parser = JsParser::new(src);
        let program = parser.parse().expect("should parse");
        SemanticAnalyzer::new().analyze(&program)
    }

    #[test]
    fn test_no_diagnostics_for_clean_code() {
        let diags = analyze("let x = 1; console.log(x);");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_duplicate_let_declaration() {
        let diags = analyze("let x = 1; let x = 2;");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code.number, codes::JS_DUPLICATE_DECLARATION.number);
    }

    #[test]
    fn test_duplicate_const_declaration() {
        let diags = analyze("const x = 1; const x = 2;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_var_redeclaration_is_allowed() {
        let diags = analyze("var x = 1; var x = 2;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_assignment_to_const() {
        let diags = analyze("const x = 1; x = 2;");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code.number, codes::JS_ASSIGN_TO_CONST.number);
    }

    #[test]
    fn test_assignment_to_let_is_allowed() {
        let diags = analyze("let x = 1; x = 2;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_undeclared_variable_warning() {
        let diags = analyze("console.log(mystery);");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Warning);
        assert_eq!(diags[0].code.number, codes::JS_UNDECLARED_VARIABLE.number);
    }

    #[test]
    fn test_known_globals_do_not_warn() {
        let diags = analyze("window.addEventListener('load', () => console.log(Math.PI));");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_function_params_are_declared() {
        let diags = analyze("function add(a, b) { return a + b; }");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_arrow_params_are_declared() {
        let diags = analyze("const add = (a, b) => a + b;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_for_of_binding_is_declared() {
        let diags =
            analyze("const items = [1, 2]; for (const item of items) { console.log(item); }");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_block_scoping_allows_shadowing_in_nested_block() {
        let diags = analyze("let x = 1; { let x = 2; console.log(x); }");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_function_can_reference_outer_scope() {
        let diags = analyze("let x = 1; function f() { return x; }");
        assert!(diags.is_empty());
    }
}
