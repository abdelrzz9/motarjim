use motarjim_js::{Expression, JsParser, Statement, VarKind};

fn parse(src: &str) -> Vec<Statement> {
    let mut parser = JsParser::new(src);
    parser.parse().unwrap().body
}

fn parse_err(src: &str) -> Vec<motarjim_js::JsDiagnostic> {
    let mut parser = JsParser::new(src);
    parser.parse().unwrap_err()
}

#[test]
fn test_var_decl() {
    let body = parse("let x = 1;");
    assert_eq!(body.len(), 1);
    assert!(matches!(body[0], Statement::VarDecl(_)));
}

#[test]
fn test_const_decl() {
    let body = parse("const x = 1;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    assert_eq!(decl.kind, VarKind::Const);
}

#[test]
fn test_multiple_declarators() {
    let body = parse("var a = 1, b = 2, c;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    assert_eq!(decl.declarators.len(), 3);
    assert!(decl.declarators[2].init.is_none());
}

#[test]
fn test_function_decl() {
    let body = parse("function add(a, b) { return a + b; }");
    let Statement::FunctionDecl(f) = &body[0] else { panic!("expected function decl") };
    assert_eq!(f.params.len(), 2);
}

#[test]
fn test_async_function_decl() {
    let body = parse("async function fetch() { await load(); }");
    let Statement::FunctionDecl(f) = &body[0] else { panic!("expected function decl") };
    assert!(f.r#async);
}

#[test]
fn test_arrow_single_param() {
    let body = parse("const double = x => x * 2;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    let Some(Expression::Arrow(arrow)) = &decl.declarators[0].init else { panic!("expected arrow") };
    assert_eq!(arrow.params.len(), 1);
}

#[test]
fn test_arrow_multi_param() {
    let body = parse("const add = (a, b) => { return a + b; };");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    let Some(Expression::Arrow(arrow)) = &decl.declarators[0].init else { panic!("expected arrow") };
    assert_eq!(arrow.params.len(), 2);
    assert!(matches!(arrow.body, motarjim_js::ArrowBody::Block(_)));
}

#[test]
fn test_async_arrow() {
    let body = parse("const f = async (x) => x;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    let Some(Expression::Arrow(arrow)) = &decl.declarators[0].init else { panic!("expected arrow") };
    assert!(arrow.r#async);
}

#[test]
fn test_template_literal() {
    let body = parse("const msg = `Hi, ${name}!`;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    let Some(Expression::TemplateLiteral(tpl)) = &decl.declarators[0].init else { panic!("expected template") };
    assert_eq!(tpl.quasis, vec!["Hi, ".to_string(), "!".to_string()]);
    assert_eq!(tpl.exprs.len(), 1);
}

#[test]
fn test_if_else() {
    let body = parse("if (x) { y(); } else { z(); }");
    let Statement::If(stmt) = &body[0] else { panic!("expected if") };
    assert!(stmt.alternate.is_some());
}

#[test]
fn test_for_loop() {
    let body = parse("for (let i = 0; i < 10; i++) { sum += i; }");
    assert!(matches!(body[0], Statement::For(_)));
}

#[test]
fn test_for_of_loop() {
    let body = parse("for (const item of items) { console.log(item); }");
    assert!(matches!(body[0], Statement::ForOf(_)));
}

#[test]
fn test_for_in_loop() {
    let body = parse("for (const key in obj) { use(key); }");
    assert!(matches!(body[0], Statement::ForIn(_)));
}

#[test]
fn test_while_loop() {
    let body = parse("while (running) { tick(); }");
    assert!(matches!(body[0], Statement::While(_)));
}

#[test]
fn test_do_while() {
    let body = parse("do { tick(); } while (running);");
    assert!(matches!(body[0], Statement::DoWhile(_)));
}

#[test]
fn test_switch_stmt() {
    let body = parse("switch(x) { case 1: break; case 2: break; default: break; }");
    assert!(matches!(body[0], Statement::Switch(_)));
}

#[test]
fn test_try_catch() {
    let body = parse("try { doSomething(); } catch(e) { handleError(); }");
    assert!(matches!(body[0], Statement::Try(_)));
}

#[test]
fn test_try_finally() {
    let body = parse("try { x(); } finally { cleanup(); }");
    assert!(matches!(body[0], Statement::Try(_)));
}

#[test]
fn test_throw() {
    let body = parse("throw new Error('fail');");
    assert!(matches!(body[0], Statement::Throw(_)));
}

#[test]
fn test_import_default() {
    let body = parse("import React from 'react';");
    let Statement::Import(decl) = &body[0] else { panic!("expected import") };
    assert_eq!(decl.default.as_deref(), Some("React"));
}

#[test]
fn test_import_named() {
    let body = parse("import { useState as useS } from 'react';");
    let Statement::Import(decl) = &body[0] else { panic!("expected import") };
    assert_eq!(decl.named[0].imported, "useState");
    assert_eq!(decl.named[0].local, "useS");
}

#[test]
fn test_import_namespace() {
    let body = parse("import * as utils from './utils.js';");
    let Statement::Import(decl) = &body[0] else { panic!("expected import") };
    assert_eq!(decl.namespace.as_deref(), Some("utils"));
}

#[test]
fn test_export_default() {
    let body = parse("export default function main() {}");
    assert!(matches!(body[0], Statement::ExportDefault(_)));
}

#[test]
fn test_export_named() {
    let body = parse("export const PI = 3.14;");
    let Statement::ExportNamed(decl) = &body[0] else { panic!("expected export") };
    assert!(decl.declaration.is_some());
}

#[test]
fn test_export_list() {
    let body = parse("export { a, b as c };");
    let Statement::ExportNamed(decl) = &body[0] else { panic!("expected export") };
    assert_eq!(decl.specifiers.len(), 2);
}

#[test]
fn test_class_decl() {
    let body = parse("class Foo extends Bar { constructor() { super(); } }");
    assert!(matches!(body[0], Statement::ClassDecl(_)));
}

#[test]
fn test_operator_precedence() {
    let body = parse("1 + 2 * 3;");
    let Statement::Expr(stmt) = &body[0] else { panic!("expected expr stmt") };
    let Expression::Binary(add) = &stmt.expr else { panic!("expected addition") };
    assert_eq!(add.op, motarjim_js::BinaryOp::Add);
    assert!(matches!(*add.right, Expression::Binary(_)));
}

#[test]
fn test_ternary() {
    let body = parse("const x = a ? b : c;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    assert!(matches!(decl.declarators[0].init, Some(Expression::Conditional(_))));
}

#[test]
fn test_object_literal() {
    let body = parse("const o = { x, y: 2 };");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    let Some(Expression::Object(obj)) = &decl.declarators[0].init else { panic!("expected object") };
    assert_eq!(obj.props.len(), 2);
}

#[test]
fn test_array_literal() {
    let body = parse("const arr = [1, 2, 3];");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    let Some(Expression::Array(arr)) = &decl.declarators[0].init else { panic!("expected array") };
    assert_eq!(arr.elements.len(), 3);
}

#[test]
fn test_member_and_call_chain() {
    let body = parse("a.b.c(1)[2];");
    let Statement::Expr(stmt) = &body[0] else { panic!("expected expr stmt") };
    assert!(matches!(stmt.expr, Expression::Member(_)));
}

#[test]
fn test_optional_chaining() {
    let body = parse("a?.b?.c();");
    let Statement::Expr(stmt) = &body[0] else { panic!("expected expr stmt") };
    assert!(matches!(stmt.expr, Expression::Member(_)));
}

#[test]
fn test_new_expression() {
    let body = parse("new Foo(1, 2);");
    let Statement::Expr(stmt) = &body[0] else { panic!("expected expr stmt") };
    assert!(matches!(stmt.expr, Expression::New(_)));
}

#[test]
fn test_exponentiation() {
    let body = parse("const x = 2 ** 3;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    let Some(Expression::Binary(bin)) = &decl.declarators[0].init else { panic!("expected binary") };
    assert_eq!(bin.op, motarjim_js::BinaryOp::Exp);
}

#[test]
fn test_spread_in_array() {
    let body = parse("const arr = [1, ...x, 2];");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    let Some(Expression::Array(arr)) = &decl.declarators[0].init else { panic!("expected array") };
    assert!(matches!(arr.elements[1], motarjim_js::ArrayElement::Spread(_)));
}

#[test]
fn test_destructuring_object() {
    let body = parse("const { a, b: c } = obj;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    assert!(matches!(decl.declarators[0].name, motarjim_js::Pattern::Object(_)));
}

#[test]
fn test_destructuring_array() {
    let body = parse("const [a, b] = arr;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    assert!(matches!(decl.declarators[0].name, motarjim_js::Pattern::Array(_)));
}

#[test]
fn test_default_parameter() {
    let body = parse("function f(x = 1) { return x; }");
    let Statement::FunctionDecl(f) = &body[0] else { panic!("expected function decl") };
    assert!(f.params[0].default.is_some());
}

#[test]
fn test_yield() {
    let body = parse("function gen() { yield 1; }");
    let Statement::FunctionDecl(f) = &body[0] else { panic!("expected function decl") };
    let Statement::Expr(stmt) = &f.body.body[0] else { panic!("expected expr") };
    assert!(matches!(stmt.expr, Expression::Yield(_)));
}

#[test]
fn test_await() {
    let body = parse("async function f() { await promise; }");
    let Statement::FunctionDecl(f) = &body[0] else { panic!("expected function decl") };
    let Statement::Expr(stmt) = &f.body.body[0] else { panic!("expected expr") };
    assert!(matches!(stmt.expr, Expression::Await(_)));
}

#[test]
fn test_error_recovery_continues() {
    let diags = parse_err("let x = ; let y = 2;");
    assert!(!diags.is_empty());
}

#[test]
fn test_empty_program() {
    let body = parse("");
    assert_eq!(body.len(), 0);
}

#[test]
fn test_only_semicolons() {
    let body = parse(";;;");
    assert_eq!(body.len(), 3);
    for stmt in &body {
        assert!(matches!(stmt, Statement::Empty(_)));
    }
}

#[test]
fn test_labelled_statement() {
    let body = parse("label: for(;;) { break label; }");
    assert!(matches!(body[0], Statement::Labelled { .. }));
}

#[test]
fn test_debugger() {
    let body = parse("debugger;");
    assert!(matches!(body[0], Statement::Debugger(_)));
}

#[test]
fn test_bigint_literal() {
    let body = parse("const x = 123n;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    assert!(matches!(decl.declarators[0].init, Some(Expression::BigInt(_))));
}

#[test]
fn test_regex_literal() {
    let body = parse("const re = /pattern/;");
    let Statement::VarDecl(decl) = &body[0] else { panic!("expected var decl") };
    assert!(matches!(decl.declarators[0].init, Some(Expression::Regex(_))));
}

#[test]
fn test_private_field() {
    let body = parse("class Foo { #priv = 1; method() { return this.#priv; } }");
    assert!(matches!(body[0], Statement::ClassDecl(_)));
}
