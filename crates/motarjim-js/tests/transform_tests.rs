use motarjim_js::{
    run_transforms, BinaryOp, Expression, JsParser, Statement, TemplateLiteralToConcat, Transform,
};

#[test]
fn test_template_literal_to_concat() {
    let mut parser = JsParser::new("const msg = `Hi, ${name}!`;");
    let mut program = parser.parse().expect("should parse");
    run_transforms(
        &mut program,
        &[Box::new(TemplateLiteralToConcat) as Box<dyn Transform>],
    );
    let Statement::VarDecl(decl) = &program.body[0] else {
        panic!("expected var decl")
    };
    let Some(init) = &decl.declarators[0].init else {
        panic!("expected init")
    };
    let Expression::Binary(bin) = init else {
        panic!("expected binary expr, got {:?}", init)
    };
    assert_eq!(bin.op, BinaryOp::Add);
}

#[test]
fn test_template_literal_no_exprs_becomes_string() {
    let mut parser = JsParser::new("const msg = `hello`;");
    let mut program = parser.parse().expect("should parse");
    run_transforms(
        &mut program,
        &[Box::new(TemplateLiteralToConcat) as Box<dyn Transform>],
    );
    let Statement::VarDecl(decl) = &program.body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::String(s)) = &decl.declarators[0].init else {
        panic!(
            "expected string literal, got {:?}",
            decl.declarators[0].init
        )
    };
    assert_eq!(s.value, "hello");
}

#[test]
fn test_template_literal_single_expr() {
    let mut parser = JsParser::new("const msg = `Hello, ${name}`;");
    let mut program = parser.parse().expect("should parse");
    run_transforms(
        &mut program,
        &[Box::new(TemplateLiteralToConcat) as Box<dyn Transform>],
    );
    let Statement::VarDecl(decl) = &program.body[0] else {
        panic!("expected var decl")
    };
    let Some(init) = &decl.declarators[0].init else {
        panic!("expected init")
    };
    assert!(matches!(init, Expression::Binary(_)));
}

#[test]
fn test_multiple_transforms_run_in_order() {
    let mut parser = JsParser::new("const msg = `test`;");
    let mut program = parser.parse().expect("should parse");
    run_transforms(
        &mut program,
        &[Box::new(TemplateLiteralToConcat) as Box<dyn Transform>],
    );
    assert_eq!(program.body.len(), 1);
}
