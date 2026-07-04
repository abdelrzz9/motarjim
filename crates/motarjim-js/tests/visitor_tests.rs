use motarjim_js::{
    Expression, JsParser, Pattern, Statement,
    visitor::{Visitor, VisitorMut, Fold, walk_program},
};

struct CountingVisitor {
    count: usize,
}

impl<'ast> Visitor<'ast> for CountingVisitor {
    fn visit_expression(&mut self, _: &'ast Expression) {
        self.count += 1;
        walk_program::walk_expression(self, _);
    }
}

#[test]
fn test_visitor_counts_expressions() {
    let mut parser = JsParser::new("let x = 1 + 2 + 3;");
    let program = parser.parse().expect("should parse");
    let mut visitor = CountingVisitor { count: 0 };
    visitor.visit_program(&program);
    assert_eq!(visitor.count, 4);
}

struct ClassCounter {
    classes: usize,
}

impl<'ast> Visitor<'ast> for ClassCounter {
    fn visit_expression(&mut self, expr: &'ast Expression) {
        if matches!(expr, Expression::ClassExpr(_)) {
            self.classes += 1;
        }
        walk_program::walk_expression(self, expr);
    }
}

#[test]
fn test_class_expression_visitor() {
    let mut parser = JsParser::new("const Cls = class {};");
    let program = parser.parse().expect("should parse");
    let mut visitor = ClassCounter { classes: 0 };
    visitor.visit_program(&program);
    assert_eq!(visitor.classes, 1);
}

struct StatementCounter {
    count: usize,
}

impl<'ast> Visitor<'ast> for StatementCounter {
    fn visit_statement(&mut self, _: &'ast Statement) {
        self.count += 1;
    }
}

#[test]
fn test_visitor_counts_statements() {
    let src = "let a = 1; let b = 2; let c = 3;";
    let mut parser = JsParser::new(src);
    let program = parser.parse().expect("should parse");
    let mut visitor = StatementCounter { count: 0 };
    visitor.visit_program(&program);
    assert_eq!(visitor.count, 3);
}

struct FunctionCounter {
    functions: usize,
}

impl<'ast> Visitor<'ast> for FunctionCounter {
    fn visit_statement(&mut self, stmt: &'ast Statement) {
        if matches!(stmt, Statement::FunctionDecl(_)) {
            self.functions += 1;
        }
        walk_program::walk_statement(self, stmt);
    }
}

#[test]
fn test_visitor_counts_functions() {
    let mut parser = JsParser::new("function a() {} function b() {}");
    let program = parser.parse().expect("should parse");
    let mut visitor = FunctionCounter { functions: 0 };
    visitor.visit_program(&program);
    assert_eq!(visitor.functions, 2);
}

// VisitorMut tests

struct MutRenamer {
    from: String,
    to: String,
}

impl VisitorMut for MutRenamer {
    fn visit_expression_mut(&mut self, expr: &mut Expression) {
        if let Expression::Identifier(name, _) = expr {
            if name == &self.from {
                *name = self.to.clone();
            }
        }
    }
}

#[test]
fn test_visitor_mut_renames_identifier() {
    let mut parser = JsParser::new("let x = 1; console.log(x);");
    let mut program = parser.parse().expect("should parse");
    let mut renamer = MutRenamer { from: "x".into(), to: "y".into() };
    renamer.visit_program_mut(&mut program);
    // After rename, we should get undeclared 'y' in semantic analysis
    let diags = motarjim_js::SemanticAnalyzer::new().analyze(&program);
    assert!(diags.is_empty(), "y should be declared: {:?}", diags);
}

// Fold test

struct FoldAddWrapper;

impl Fold for FoldAddWrapper {
    fn fold_expression(&mut self, expr: Expression) -> Expression {
        match expr {
            Expression::Number(num) => {
                // Wrap each number in an array: [n]
                Expression::Array(
                    motarjim_js::ArrayLit {
                        elements: vec![motarjim_js::ArrayElement::Expr(Box::new(Expression::Number(num)))],
                        span: Default::default(),
                    }
                )
            }
            other => motarjim_js::visitor::walk_fold_expression(self, other),
        }
    }
}

#[test]
fn test_fold_wraps_numbers_in_array() {
    let mut parser = JsParser::new("const x = 42;");
    let program = parser.parse().expect("should parse");
    let mut f = FoldAddWrapper;
    let result = f.fold_program(program);
    let Statement::VarDecl(decl) = &result.body[0] else { panic!("expected var decl") };
    assert!(matches!(decl.declarators[0].init, Some(Expression::Array(_))));
}
