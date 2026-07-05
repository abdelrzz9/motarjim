//! Built-in transform passes (e.g., [`TemplateLiteralToConcat`]).

use crate::ast::expr::*;
use crate::ast::lit::*;
use crate::ast::pat::*;
use crate::ast::program::Program;
use crate::ast::stmt::*;
use crate::transform::Transform;

#[derive(Debug, Default)]
pub struct TemplateLiteralToConcat;

impl Transform for TemplateLiteralToConcat {
    fn name(&self) -> &'static str {
        "template-literal-to-concat"
    }

    fn apply(&self, program: &mut Program) {
        for stmt in &mut program.body {
            rewrite_stmt(stmt);
        }
    }
}

fn rewrite_expr(expr: &mut Expression) {
    match expr {
        Expression::TemplateLiteral(tpl) => {
            for e in &mut tpl.exprs {
                rewrite_expr(e);
            }
        }
        Expression::Array(arr) => {
            for el in &mut arr.elements {
                match el {
                    ArrayElement::Some(e) => rewrite_expr(e),
                    ArrayElement::Spread(e) => rewrite_expr(e),
                    ArrayElement::None(_) => {}
                }
            }
        }
        Expression::Object(obj) => {
            for prop in &mut obj.props {
                if let PropKey::Computed(key) = &mut prop.key {
                    rewrite_expr(key);
                }
                rewrite_expr(&mut prop.value);
            }
        }
        Expression::Function(func) => {
            for param in &mut func.params {
                if let Some(default) = &mut param.default {
                    rewrite_expr(default);
                }
            }
            rewrite_block(&mut func.body);
        }
        Expression::Arrow(arrow) => {
            for param in &mut arrow.params {
                if let Some(default) = &mut param.default {
                    rewrite_expr(default);
                }
            }
            match &mut arrow.body {
                ArrowBody::Block(block) => rewrite_block(block),
                ArrowBody::Expr(e) => rewrite_expr(e),
            }
        }
        Expression::Unary(u) | Expression::Update(u) => rewrite_expr(&mut u.argument),
        Expression::Binary(b) => {
            rewrite_expr(&mut b.left);
            rewrite_expr(&mut b.right);
        }
        Expression::Logical(l) => {
            rewrite_expr(&mut l.left);
            rewrite_expr(&mut l.right);
        }
        Expression::Assignment(a) => {
            rewrite_expr(&mut a.target);
            rewrite_expr(&mut a.value);
        }
        Expression::Conditional(c) => {
            rewrite_expr(&mut c.test);
            rewrite_expr(&mut c.consequent);
            rewrite_expr(&mut c.alternate);
        }
        Expression::Call(c) => {
            rewrite_expr(&mut c.callee);
            for a in &mut c.args {
                rewrite_expr(a);
            }
        }
        Expression::New(n) => {
            rewrite_expr(&mut n.callee);
            for a in &mut n.args {
                rewrite_expr(a);
            }
        }
        Expression::Member(m) => {
            rewrite_expr(&mut m.object);
            if let MemberProp::Computed(p) = &mut m.property {
                rewrite_expr(p);
            }
        }
        Expression::Sequence(s) => {
            for e in &mut s.exprs {
                rewrite_expr(e);
            }
        }
        Expression::Yield(y) => {
            if let Some(arg) = &mut y.argument {
                rewrite_expr(arg);
            }
        }
        Expression::Await(a) => rewrite_expr(&mut a.argument),
        Expression::Spread(s) => rewrite_expr(s),
        Expression::Parenthesized(p) => rewrite_expr(p),
        Expression::ClassExpr(c) => {
            if let Some(super_class) = &mut c.super_class {
                rewrite_expr(super_class);
            }
            for member in &mut c.body.body {
                match member {
                    ClassMember::Method(m) => {
                        for param in &mut m.function.params {
                            if let Some(default) = &mut param.default {
                                rewrite_expr(default);
                            }
                        }
                        rewrite_block(&mut m.function.body);
                    }
                    ClassMember::Property(p) => {
                        if let Some(value) = &mut p.value {
                            rewrite_expr(value);
                        }
                    }
                }
            }
        }
        Expression::Identifier(..)
        | Expression::PrivateIdentifier(..)
        | Expression::Number(_)
        | Expression::BigInt(_)
        | Expression::String(_)
        | Expression::Bool(_)
        | Expression::Regex(_)
        | Expression::Null(_)
        | Expression::Undefined(_)
        | Expression::This(_)
        | Expression::Super(_)
        | Expression::MetaProperty(_) => {}
    }

    if matches!(expr, Expression::TemplateLiteral(_)) {
        let placeholder_span = expr.span();
        let owned = std::mem::replace(expr, Expression::Null(placeholder_span));
        let Expression::TemplateLiteral(tpl) = owned else {
            unreachable!()
        };
        *expr = template_to_concat(*tpl);
    }
}

fn template_to_concat(tpl: TemplateLiteral) -> Expression {
    let TemplateLiteral {
        quasis,
        exprs,
        span,
    } = tpl;
    let mut quasis = quasis.into_iter();
    let mut result = Expression::String(Box::new(StringLit {
        value: quasis.next().unwrap_or_default(),
        span,
    }));
    for (expr, quasi) in exprs.into_iter().zip(quasis) {
        result = Expression::Binary(Box::new(BinaryExpr {
            op: BinaryOp::Add,
            left: Box::new(result),
            right: Box::new(expr),
            span,
        }));
        result = Expression::Binary(Box::new(BinaryExpr {
            op: BinaryOp::Add,
            left: Box::new(result),
            right: Box::new(Expression::String(Box::new(StringLit {
                value: quasi,
                span,
            }))),
            span,
        }));
    }
    result
}

fn rewrite_block(block: &mut BlockStmt) {
    for stmt in &mut block.body {
        rewrite_stmt(stmt);
    }
}

fn rewrite_stmt(stmt: &mut Statement) {
    match stmt {
        Statement::VarDecl(decl) => {
            for declarator in &mut decl.declarators {
                if let Some(init) = &mut declarator.init {
                    rewrite_expr(init);
                }
            }
        }
        Statement::FunctionDecl(func) => {
            for param in &mut func.params {
                if let Some(default) = &mut param.default {
                    rewrite_expr(default);
                }
            }
            rewrite_block(&mut func.body);
        }
        Statement::Return(ret) => {
            if let Some(arg) = &mut ret.argument {
                rewrite_expr(arg);
            }
        }
        Statement::If(if_stmt) => {
            rewrite_expr(&mut if_stmt.test);
            rewrite_stmt(&mut if_stmt.consequent);
            if let Some(alt) = &mut if_stmt.alternate {
                rewrite_stmt(alt);
            }
        }
        Statement::For(for_stmt) => {
            if let Some(init) = &mut for_stmt.init {
                match init {
                    ForInit::VarDecl(decl) => {
                        for declarator in &mut decl.declarators {
                            if let Some(init) = &mut declarator.init {
                                rewrite_expr(init);
                            }
                        }
                    }
                    ForInit::Expr(expr) => rewrite_expr(expr),
                }
            }
            if let Some(test) = &mut for_stmt.test {
                rewrite_expr(test);
            }
            if let Some(update) = &mut for_stmt.update {
                rewrite_expr(update);
            }
            rewrite_stmt(&mut for_stmt.body);
        }
        Statement::ForOf(for_of) => {
            rewrite_expr(&mut for_of.right);
            rewrite_stmt(&mut for_of.body);
        }
        Statement::ForIn(for_in) => {
            rewrite_expr(&mut for_in.right);
            rewrite_stmt(&mut for_in.body);
        }
        Statement::While(while_stmt) => {
            rewrite_expr(&mut while_stmt.test);
            rewrite_stmt(&mut while_stmt.body);
        }
        Statement::DoWhile(do_while) => {
            rewrite_stmt(&mut do_while.body);
            rewrite_expr(&mut do_while.test);
        }
        Statement::Block(block) => rewrite_block(block),
        Statement::Break(_)
        | Statement::Continue(_)
        | Statement::Empty(_)
        | Statement::Import(_)
        | Statement::Debugger(_) => {}
        Statement::Throw(s) => rewrite_expr(&mut s.argument),
        Statement::Try(s) => {
            rewrite_block(&mut s.block);
            if let Some(handler) = &mut s.handler {
                rewrite_block(&mut handler.body);
            }
            if let Some(finalizer) = &mut s.finalizer {
                rewrite_block(finalizer);
            }
        }
        Statement::Switch(s) => {
            rewrite_expr(&mut s.discriminant);
            for case in &mut s.cases {
                if let Some(test) = &mut case.test {
                    rewrite_expr(test);
                }
            }
        }
        Statement::Expr(expr_stmt) => rewrite_expr(&mut expr_stmt.expr),
        Statement::ExportNamed(export) => {
            if let Some(decl) = &mut export.declaration {
                rewrite_stmt(decl);
            }
        }
        Statement::ExportDefault(export) => match &mut export.declaration {
            ExportDefaultKind::Expression(expr) => rewrite_expr(expr),
            ExportDefaultKind::FunctionDecl(f) => {
                for param in &mut f.params {
                    if let Some(default) = &mut param.default {
                        rewrite_expr(default);
                    }
                }
                rewrite_block(&mut f.body);
            }
            ExportDefaultKind::ClassDecl(c) => {
                for member in &mut c.body.body {
                    match member {
                        ClassMember::Method(m) => {
                            for param in &mut m.function.params {
                                if let Some(default) = &mut param.default {
                                    rewrite_expr(default);
                                }
                            }
                            rewrite_block(&mut m.function.body);
                        }
                        ClassMember::Property(p) => {
                            if let Some(value) = &mut p.value {
                                rewrite_expr(value);
                            }
                        }
                    }
                }
            }
        },
        Statement::ClassDecl(c) => {
            for member in &mut c.body.body {
                match member {
                    ClassMember::Method(m) => {
                        for param in &mut m.function.params {
                            if let Some(default) = &mut param.default {
                                rewrite_expr(default);
                            }
                        }
                        rewrite_block(&mut m.function.body);
                    }
                    ClassMember::Property(p) => {
                        if let Some(value) = &mut p.value {
                            rewrite_expr(value);
                        }
                    }
                }
            }
        }
        Statement::Labelled { body, .. } => rewrite_stmt(body),
    }
}
