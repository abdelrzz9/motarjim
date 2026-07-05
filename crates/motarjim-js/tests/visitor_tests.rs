use motarjim_js::{
    visitor::{Fold, Visitor, VisitorMut},
    walk_expression, walk_statement, Expression, JsParser, Statement,
};

struct CountingVisitor {
    count: usize,
}

impl Visitor for CountingVisitor {
    fn visit_expression(&mut self, _expr: &Expression) {
        self.count += 1;
        walk_expression(self, _expr);
    }
}

#[test]
fn test_visitor_counts_expressions() {
    let mut parser = JsParser::new("let x = 1 + 2 + 3;");
    let program = parser.parse().expect("should parse");
    // Debug: dump statement
    if let Statement::VarDecl(decl) = &program.body[0] {
        eprintln!(
            "DEBUG: VarDecl declarators={}, init={:?}, name={:?}",
            decl.declarators.len(),
            decl.declarators[0].init.is_some(),
            decl.declarators[0].name
        );
    } else {
        eprintln!("DEBUG: unexpected statement");
    }
    let mut visitor = CountingVisitor { count: 0 };
    visitor.visit_program(&program);
    assert_eq!(visitor.count, 5);
}

struct FunctionExprCounter {
    count: usize,
}

impl Visitor for FunctionExprCounter {
    fn visit_expression(&mut self, expr: &Expression) {
        if matches!(expr, Expression::Function(_)) {
            self.count += 1;
        }
        walk_expression(self, expr);
    }
}

#[test]
fn test_function_expression_visitor() {
    let mut parser = JsParser::new("const fn = function() {};");
    let program = parser.parse().expect("should parse");
    let mut visitor = FunctionExprCounter { count: 0 };
    visitor.visit_program(&program);
    assert_eq!(visitor.count, 1);
}

struct StatementCounter {
    count: usize,
}

impl Visitor for StatementCounter {
    fn visit_statement(&mut self, _: &Statement) {
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

impl Visitor for FunctionCounter {
    fn visit_statement(&mut self, stmt: &Statement) {
        if matches!(stmt, Statement::FunctionDecl(_)) {
            self.functions += 1;
        }
        walk_statement(self, stmt);
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
    let mut renamer = MutRenamer {
        from: "x".into(),
        to: "y".into(),
    };
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
                Expression::Array(Box::new(motarjim_js::ArrayLit {
                    elements: vec![motarjim_js::ArrayElement::Some(Expression::Number(num))],
                    span: Default::default(),
                }))
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
    let Statement::VarDecl(decl) = &result.body[0] else {
        panic!("expected var decl")
    };
    assert!(matches!(
        decl.declarators[0].init,
        Some(Expression::Array(_))
    ));
}
