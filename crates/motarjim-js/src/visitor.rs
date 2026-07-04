//! A shared visitor for read-only traversal of the JavaScript AST.
//!
//! Implement [`Visitor`] and override whichever `visit_*` methods you need;
//! call the matching `walk_*` free function to continue recursing into a
//! node's children (the default method bodies already do this, so only
//! nodes you explicitly stop at need special handling).

use crate::ast::{ArrowBody, Expression, ForInit, MemberProp, Program, PropKey, Statement};

/// A read-only visitor over the JavaScript AST.
///
/// The default method implementations simply recurse into every child node,
/// so a visitor that only cares about, say, call expressions can override
/// just [`Visitor::visit_expression`] and still see the whole tree — as long
/// as it calls [`walk_expression`] to continue the traversal.
pub trait Visitor {
    /// Visits a whole program.
    fn visit_program(&mut self, program: &Program) {
        walk_program(self, program);
    }

    /// Visits a single statement.
    fn visit_statement(&mut self, stmt: &Statement) {
        walk_statement(self, stmt);
    }

    /// Visits a single expression.
    fn visit_expression(&mut self, expr: &Expression) {
        walk_expression(self, expr);
    }
}

/// Visits every top-level statement in `program`, in order.
pub fn walk_program<V: Visitor + ?Sized>(visitor: &mut V, program: &Program) {
    for stmt in &program.body {
        visitor.visit_statement(stmt);
    }
}

/// Visits every expression and nested statement directly reachable from `stmt`.
pub fn walk_statement<V: Visitor + ?Sized>(visitor: &mut V, stmt: &Statement) {
    match stmt {
        Statement::VarDecl(decl) => {
            for declarator in &decl.declarators {
                if let Some(init) = &declarator.init {
                    visitor.visit_expression(init);
                }
            }
        }
        Statement::FunctionDecl(func) => {
            for param in &func.params {
                if let Some(default) = &param.default {
                    visitor.visit_expression(default);
                }
            }
            for stmt in &func.body.body {
                visitor.visit_statement(stmt);
            }
        }
        Statement::Return(ret) => {
            if let Some(arg) = &ret.argument {
                visitor.visit_expression(arg);
            }
        }
        Statement::If(if_stmt) => {
            visitor.visit_expression(&if_stmt.test);
            visitor.visit_statement(&if_stmt.consequent);
            if let Some(alt) = &if_stmt.alternate {
                visitor.visit_statement(alt);
            }
        }
        Statement::For(for_stmt) => {
            if let Some(init) = &for_stmt.init {
                match init {
                    ForInit::VarDecl(decl) => {
                        for declarator in &decl.declarators {
                            if let Some(init) = &declarator.init {
                                visitor.visit_expression(init);
                            }
                        }
                    }
                    ForInit::Expr(expr) => visitor.visit_expression(expr),
                }
            }
            if let Some(test) = &for_stmt.test {
                visitor.visit_expression(test);
            }
            if let Some(update) = &for_stmt.update {
                visitor.visit_expression(update);
            }
            visitor.visit_statement(&for_stmt.body);
        }
        Statement::ForOf(for_of) => {
            visitor.visit_expression(&for_of.right);
            visitor.visit_statement(&for_of.body);
        }
        Statement::ForIn(for_in) => {
            visitor.visit_expression(&for_in.right);
            visitor.visit_statement(&for_in.body);
        }
        Statement::While(while_stmt) => {
            visitor.visit_expression(&while_stmt.test);
            visitor.visit_statement(&while_stmt.body);
        }
        Statement::DoWhile(do_while) => {
            visitor.visit_statement(&do_while.body);
            visitor.visit_expression(&do_while.test);
        }
        Statement::Block(block) => {
            for stmt in &block.body {
                visitor.visit_statement(stmt);
            }
        }
        Statement::Break(_) | Statement::Continue(_) | Statement::Empty(_) => {}
        Statement::Expr(expr_stmt) => visitor.visit_expression(&expr_stmt.expr),
        Statement::Import(_) => {}
        Statement::ExportNamed(export) => {
            if let Some(decl) = &export.declaration {
                visitor.visit_statement(decl);
            }
        }
        Statement::ExportDefault(export) => visitor.visit_expression(&export.expr),
    }
}

/// Visits every child expression and nested statement directly reachable
/// from `expr`.
pub fn walk_expression<V: Visitor + ?Sized>(visitor: &mut V, expr: &Expression) {
    match expr {
        Expression::Identifier(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Bool(_)
        | Expression::Null(_)
        | Expression::Undefined(_)
        | Expression::This(_) => {}
        Expression::TemplateLiteral(tpl) => {
            for expr in &tpl.exprs {
                visitor.visit_expression(expr);
            }
        }
        Expression::Array(arr) => {
            for expr in &arr.elements {
                visitor.visit_expression(expr);
            }
        }
        Expression::Object(obj) => {
            for prop in &obj.props {
                if let PropKey::Computed(key) = &prop.key {
                    visitor.visit_expression(key);
                }
                visitor.visit_expression(&prop.value);
            }
        }
        Expression::Function(func) => {
            for param in &func.params {
                if let Some(default) = &param.default {
                    visitor.visit_expression(default);
                }
            }
            for stmt in &func.body.body {
                visitor.visit_statement(stmt);
            }
        }
        Expression::Arrow(arrow) => {
            for param in &arrow.params {
                if let Some(default) = &param.default {
                    visitor.visit_expression(default);
                }
            }
            match &arrow.body {
                ArrowBody::Block(block) => {
                    for stmt in &block.body {
                        visitor.visit_statement(stmt);
                    }
                }
                ArrowBody::Expr(expr) => visitor.visit_expression(expr),
            }
        }
        Expression::Unary(unary) => visitor.visit_expression(&unary.argument),
        Expression::Binary(bin) => {
            visitor.visit_expression(&bin.left);
            visitor.visit_expression(&bin.right);
        }
        Expression::Logical(logical) => {
            visitor.visit_expression(&logical.left);
            visitor.visit_expression(&logical.right);
        }
        Expression::Assignment(assign) => {
            visitor.visit_expression(&assign.target);
            visitor.visit_expression(&assign.value);
        }
        Expression::Conditional(cond) => {
            visitor.visit_expression(&cond.test);
            visitor.visit_expression(&cond.consequent);
            visitor.visit_expression(&cond.alternate);
        }
        Expression::Call(call) => {
            visitor.visit_expression(&call.callee);
            for arg in &call.args {
                visitor.visit_expression(arg);
            }
        }
        Expression::New(new_expr) => {
            visitor.visit_expression(&new_expr.callee);
            for arg in &new_expr.args {
                visitor.visit_expression(arg);
            }
        }
        Expression::Member(member) => {
            visitor.visit_expression(&member.object);
            if let MemberProp::Computed(prop) = &member.property {
                visitor.visit_expression(prop);
            }
        }
        Expression::Sequence(seq) => {
            for expr in &seq.exprs {
                visitor.visit_expression(expr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::JsParser;

    #[derive(Default)]
    struct IdentifierCounter {
        count: usize,
    }

    impl Visitor for IdentifierCounter {
        fn visit_expression(&mut self, expr: &Expression) {
            if matches!(expr, Expression::Identifier(_)) {
                self.count += 1;
            }
            walk_expression(self, expr);
        }
    }

    #[test]
    fn test_visitor_counts_identifiers() {
        let mut parser = JsParser::new("const x = a + b * c;");
        let program = parser.parse().expect("should parse");
        let mut counter = IdentifierCounter::default();
        counter.visit_program(&program);
        assert_eq!(counter.count, 3);
    }

    #[test]
    fn test_visitor_walks_nested_function_bodies() {
        let mut parser = JsParser::new("function f() { const x = g(y); }");
        let program = parser.parse().expect("should parse");
        let mut counter = IdentifierCounter::default();
        counter.visit_program(&program);
        assert_eq!(counter.count, 2);
    }

    #[test]
    fn test_visitor_walks_arrow_block_body() {
        let mut parser = JsParser::new("const f = () => { return a + b; };");
        let program = parser.parse().expect("should parse");
        let mut counter = IdentifierCounter::default();
        counter.visit_program(&program);
        assert_eq!(counter.count, 2);
    }
}
