//! AST-to-AST transform infrastructure.
//!
//! A [`Transform`] rewrites a [`Program`] in place. This is the extension
//! point for future lowering passes (e.g. rewriting arrow functions to
//! function expressions for older targets); [`TemplateLiteralToConcat`] is
//! the first concrete transform, lowering template literals into string
//! concatenation for targets without native interpolation support.

use crate::ast::{
    ArrowBody, BinaryExpr, BinaryOp, BlockStmt, Expression, ForInit, MemberProp, Program, PropKey,
    Statement, StringLit, TemplateLiteral,
};

/// A single AST-to-AST rewrite applied to a whole program.
pub trait Transform {
    /// A short, stable name identifying this transform (for logging).
    fn name(&self) -> &'static str;

    /// Rewrites `program` in place.
    fn apply(&self, program: &mut Program);
}

/// Runs every transform in `transforms`, in order, over `program`.
pub fn run_transforms(program: &mut Program, transforms: &[Box<dyn Transform>]) {
    for transform in transforms {
        transform.apply(program);
    }
}

/// Lowers every template literal into an equivalent chain of string
/// concatenations, e.g. `` `Hi, ${name}!` `` becomes `"Hi, " + name + "!"`.
///
/// Useful for code generation targets that have no native template literal
/// equivalent.
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

/// Recursively rewrites every template literal reachable from `expr`,
/// including nested ones inside its own interpolated expressions.
fn rewrite_expr(expr: &mut Expression) {
    match expr {
        Expression::TemplateLiteral(tpl) => {
            for e in &mut tpl.exprs {
                rewrite_expr(e);
            }
        }
        Expression::Array(arr) => {
            for e in &mut arr.elements {
                rewrite_expr(e);
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
        Expression::Unary(u) => rewrite_expr(&mut u.argument),
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
        Expression::Identifier(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Bool(_)
        | Expression::Null(_)
        | Expression::Undefined(_)
        | Expression::This(_) => {}
    }

    // Children (if any) have already been rewritten above; now replace this
    // node itself if it is a template literal.
    if matches!(expr, Expression::TemplateLiteral(_)) {
        let placeholder_span = expr.span();
        let owned = std::mem::replace(expr, Expression::Undefined(placeholder_span));
        let Expression::TemplateLiteral(tpl) = owned else {
            unreachable!("just matched Expression::TemplateLiteral above")
        };
        *expr = template_to_concat(tpl);
    }
}

/// Converts a template literal's quasis and interpolations into a
/// left-associative chain of `+` (string concatenation) expressions.
fn template_to_concat(tpl: TemplateLiteral) -> Expression {
    let TemplateLiteral {
        quasis,
        exprs,
        span,
    } = tpl;
    let mut quasis = quasis.into_iter();
    let mut result = Expression::String(StringLit {
        value: quasis.next().unwrap_or_default(),
        span,
    });
    for (expr, quasi) in exprs.into_iter().zip(quasis) {
        result = Expression::Binary(BinaryExpr {
            op: BinaryOp::Add,
            left: Box::new(result),
            right: Box::new(expr),
            span,
        });
        result = Expression::Binary(BinaryExpr {
            op: BinaryOp::Add,
            left: Box::new(result),
            right: Box::new(Expression::String(StringLit { value: quasi, span })),
            span,
        });
    }
    result
}

/// Rewrites every statement in a block.
fn rewrite_block(block: &mut BlockStmt) {
    for stmt in &mut block.body {
        rewrite_stmt(stmt);
    }
}

/// Recursively rewrites every expression reachable from `stmt`.
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
        | Statement::Import(_) => {}
        Statement::Expr(expr_stmt) => rewrite_expr(&mut expr_stmt.expr),
        Statement::ExportNamed(export) => {
            if let Some(decl) = &mut export.declaration {
                rewrite_stmt(decl);
            }
        }
        Statement::ExportDefault(export) => rewrite_expr(&mut export.expr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::JsParser;

    fn transform(src: &str) -> Program {
        let mut parser = JsParser::new(src);
        let mut program = parser.parse().expect("should parse");
        TemplateLiteralToConcat.apply(&mut program);
        program
    }

    #[test]
    fn test_simple_template_literal_becomes_concat() {
        let program = transform("const msg = `Hi, ${name}!`;");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        assert!(matches!(
            decl.declarators[0].init,
            Some(Expression::Binary(_))
        ));
    }

    #[test]
    fn test_no_template_literal_is_untouched() {
        let program = transform("const x = 1 + 2;");
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        assert!(matches!(
            decl.declarators[0].init,
            Some(Expression::Binary(_))
        ));
    }

    #[test]
    fn test_template_literal_inside_function_body() {
        let program = transform("function greet(name) { return `Hi, ${name}!`; }");
        let Statement::FunctionDecl(func) = &program.body[0] else {
            panic!("expected function decl");
        };
        let Statement::Return(ret) = &func.body.body[0] else {
            panic!("expected return statement");
        };
        assert!(matches!(ret.argument, Some(Expression::Binary(_))));
    }

    #[test]
    fn test_transform_name() {
        assert_eq!(TemplateLiteralToConcat.name(), "template-literal-to-concat");
    }

    #[test]
    fn test_run_transforms_helper() {
        let mut parser = JsParser::new("const msg = `Hi, ${name}!`;");
        let mut program = parser.parse().expect("should parse");
        let transforms: Vec<Box<dyn Transform>> = vec![Box::new(TemplateLiteralToConcat)];
        run_transforms(&mut program, &transforms);
        let Statement::VarDecl(decl) = &program.body[0] else {
            panic!("expected var decl");
        };
        assert!(matches!(
            decl.declarators[0].init,
            Some(Expression::Binary(_))
        ));
    }
}
