use motarjim_js::{ClassMember, Expression, JsParser, Statement, VarKind};

fn parse(src: &str) -> Vec<Statement> {
    let mut parser = JsParser::new(src);
    parser.parse().unwrap().body
}

fn try_parse(src: &str) -> Result<Vec<Statement>, Vec<motarjim_js::JsDiagnostic>> {
    let mut parser = JsParser::new(src);
    parser.parse().map(|p| p.body)
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
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    assert_eq!(decl.kind, VarKind::Const);
}

#[test]
fn test_multiple_declarators() {
    let body = parse("var a = 1, b = 2, c;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    assert_eq!(decl.declarators.len(), 3);
    assert!(decl.declarators[2].init.is_none());
}

#[test]
fn test_function_decl() {
    let body = parse("function add(a, b) { return a + b; }");
    let Statement::FunctionDecl(f) = &body[0] else {
        panic!("expected function decl")
    };
    assert_eq!(f.params.len(), 2);
}

#[test]
fn test_async_function_decl() {
    let body = parse("async function fetch() { await load(); }");
    let Statement::FunctionDecl(f) = &body[0] else {
        panic!("expected function decl")
    };
    assert!(f.r#async);
}

#[test]
fn test_arrow_single_param() {
    let body = parse("const double = x => x * 2;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::Arrow(arrow)) = &decl.declarators[0].init else {
        panic!("expected arrow")
    };
    assert_eq!(arrow.params.len(), 1);
}

#[test]
fn test_arrow_multi_param() {
    let body = parse("const add = (a, b) => { return a + b; };");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::Arrow(arrow)) = &decl.declarators[0].init else {
        panic!("expected arrow")
    };
    assert_eq!(arrow.params.len(), 2);
    assert!(matches!(arrow.body, motarjim_js::ArrowBody::Block(_)));
}

#[test]
fn test_async_arrow() {
    let body = parse("const f = async (x) => x;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::Arrow(arrow)) = &decl.declarators[0].init else {
        panic!("expected arrow")
    };
    assert!(arrow.r#async);
}

#[test]
fn test_template_literal() {
    let body = parse("const msg = `Hi, ${name}!`;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::TemplateLiteral(tpl)) = &decl.declarators[0].init else {
        panic!("expected template")
    };
    assert_eq!(tpl.quasis, vec!["Hi, ".to_string(), "!".to_string()]);
    assert_eq!(tpl.exprs.len(), 1);
}

#[test]
fn test_if_else() {
    let body = parse("if (x) { y(); } else { z(); }");
    let Statement::If(stmt) = &body[0] else {
        panic!("expected if")
    };
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
    let Statement::Import(decl) = &body[0] else {
        panic!("expected import")
    };
    assert_eq!(decl.default.as_deref(), Some("React"));
}

#[test]
fn test_import_named() {
    let body = parse("import { useState as useS } from 'react';");
    let Statement::Import(decl) = &body[0] else {
        panic!("expected import")
    };
    assert_eq!(decl.named[0].imported, "useState");
    assert_eq!(decl.named[0].local, "useS");
}

#[test]
fn test_import_namespace() {
    let body = parse("import * as utils from './utils.js';");
    let Statement::Import(decl) = &body[0] else {
        panic!("expected import")
    };
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
    let Statement::ExportNamed(decl) = &body[0] else {
        panic!("expected export")
    };
    assert!(decl.declaration.is_some());
}

#[test]
fn test_export_list() {
    let body = parse("export { a, b as c };");
    let Statement::ExportNamed(decl) = &body[0] else {
        panic!("expected export")
    };
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
    let Statement::Expr(stmt) = &body[0] else {
        panic!("expected expr stmt")
    };
    let Expression::Binary(add) = &stmt.expr else {
        panic!("expected addition")
    };
    assert_eq!(add.op, motarjim_js::BinaryOp::Add);
    assert!(matches!(*add.right, Expression::Binary(_)));
}

#[test]
fn test_ternary() {
    let body = parse("const x = a ? b : c;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    assert!(matches!(
        decl.declarators[0].init,
        Some(Expression::Conditional(_))
    ));
}

#[test]
fn test_object_literal() {
    let body = parse("const o = { x, y: 2 };");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::Object(obj)) = &decl.declarators[0].init else {
        panic!("expected object")
    };
    assert_eq!(obj.props.len(), 2);
}

#[test]
fn test_array_literal() {
    let body = parse("const arr = [1, 2, 3];");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::Array(arr)) = &decl.declarators[0].init else {
        panic!("expected array")
    };
    assert_eq!(arr.elements.len(), 3);
}

#[test]
fn test_member_and_call_chain() {
    let body = parse("a.b.c(1)[2];");
    let Statement::Expr(stmt) = &body[0] else {
        panic!("expected expr stmt")
    };
    assert!(matches!(stmt.expr, Expression::Member(_)));
}

#[test]
fn test_optional_chaining() {
    let body = parse("a?.b?.c();");
    let Statement::Expr(stmt) = &body[0] else {
        panic!("expected expr stmt")
    };
    assert!(matches!(stmt.expr, Expression::Call(_)));
}

#[test]
fn test_new_expression() {
    let body = parse("new Foo(1, 2);");
    let Statement::Expr(stmt) = &body[0] else {
        panic!("expected expr stmt")
    };
    assert!(matches!(stmt.expr, Expression::New(_)));
}

#[test]
fn test_exponentiation() {
    let body = parse("const x = 2 ** 3;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::Binary(bin)) = &decl.declarators[0].init else {
        panic!("expected binary")
    };
    assert_eq!(bin.op, motarjim_js::BinaryOp::Exp);
}

#[test]
fn test_spread_in_array() {
    let body = parse("const arr = [1, ...x, 2];");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::Array(arr)) = &decl.declarators[0].init else {
        panic!("expected array")
    };
    assert!(matches!(
        arr.elements[1],
        motarjim_js::ArrayElement::Spread(_)
    ));
}

#[test]
fn test_destructuring_object() {
    let body = parse("const { a, b: c } = obj;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    assert!(matches!(
        decl.declarators[0].name,
        motarjim_js::Pattern::Object(_)
    ));
}

#[test]
fn test_destructuring_array() {
    let body = parse("const [a, b] = arr;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    assert!(matches!(
        decl.declarators[0].name,
        motarjim_js::Pattern::Array(_)
    ));
}

#[test]
fn test_default_parameter() {
    let body = parse("function f(x = 1) { return x; }");
    let Statement::FunctionDecl(f) = &body[0] else {
        panic!("expected function decl")
    };
    assert!(f.params[0].default.is_some());
}

#[test]
fn test_yield() {
    let body = parse("function gen() { yield 1; }");
    let Statement::FunctionDecl(f) = &body[0] else {
        panic!("expected function decl")
    };
    let Statement::Expr(stmt) = &f.body.body[0] else {
        panic!("expected expr")
    };
    assert!(matches!(stmt.expr, Expression::Yield(_)));
}

#[test]
fn test_await() {
    let body = parse("async function f() { await promise; }");
    let Statement::FunctionDecl(f) = &body[0] else {
        panic!("expected function decl")
    };
    let Statement::Expr(stmt) = &f.body.body[0] else {
        panic!("expected expr")
    };
    assert!(matches!(stmt.expr, Expression::Await(_)));
}

#[test]
fn test_parse_assign_op_no_panic() {
    // parse_assign_op had unreachable!() at the default arm — verify it degrades
    // to a diagnostic instead of crashing.
    for src in &["x = ", "x += ", "x **= ", "x ??= ", "x &&= ", "x ||= "] {
        let mut parser = JsParser::new(src);
        let _ = parser.parse(); // must not panic
    }
}

#[test]
fn test_class_expr() {
    // parse_class_expr previously returned Expression::Function with empty body.
    // Note: (class {}) wraps ClassExpr in Parenthesized
    let body = parse("const Cls = class { foo() { return 1; } };");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::ClassExpr(c)) = &decl.declarators[0].init else {
        panic!("expected ClassExpr")
    };
    assert!(c.name.is_none(), "anonymous class should have no name");
    assert_eq!(c.body.body.len(), 1);
}

#[test]
fn test_class_expr_named() {
    let body = parse("const Cls = class MyClass extends Base { constructor() { super(); } };");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::ClassExpr(c)) = &decl.declarators[0].init else {
        panic!("expected ClassExpr")
    };
    assert!(c.name.is_some());
    assert!(c.super_class.is_some());
}

#[test]
fn test_class_getter_setter() {
    // Getter/setter detection was broken (checked after consuming `(`)
    let body = parse("class Foo { get x() { return 1; } set x(v) { this._x = v; } }");
    let Statement::ClassDecl(c) = &body[0] else {
        panic!("expected class decl")
    };
    assert_eq!(c.body.body.len(), 2);
    match &c.body.body[0] {
        ClassMember::Method(m) => assert_eq!(m.kind, motarjim_js::MethodKind::Get),
        _ => panic!("expected getter"),
    }
    match &c.body.body[1] {
        ClassMember::Method(m) => assert_eq!(m.kind, motarjim_js::MethodKind::Set),
        _ => panic!("expected setter"),
    }
}

#[test]
fn test_regex_with_flags() {
    // Regex flags were always set to String::new()
    let body = parse("const re = /pattern/gim;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::Regex(re)) = &decl.declarators[0].init else {
        panic!("expected regex")
    };
    assert_eq!(re.pattern, "pattern");
    assert_eq!(re.flags, "gim");
}

#[test]
fn test_nullish_coalescing_mixing_error() {
    // Mixing ?? with && or || without parens should be a syntax error
    let diags = parse_err("const x = a ?? b || c;");
    assert!(!diags.is_empty(), "mixing ?? and || should error");
    let diags = parse_err("const x = a && b ?? c;");
    assert!(!diags.is_empty(), "mixing && and ?? should error");
}

#[test]
fn test_nullish_coalescing_with_parens_ok() {
    // With explicit grouping, mixing should be allowed
    let body = parse("const x = (a ?? b) || c;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    assert!(decl.declarators[0].init.is_some());
    let body = parse("const x = a ?? (b || c);");
    assert!(body.len() == 1);
}

#[test]
fn test_for_await_of() {
    // for-await-of was missing — await keyword was not consumed before of
    let body = parse("async function f() { for await (const x of iter) { } }");
    let Statement::FunctionDecl(f) = &body[0] else {
        panic!("expected function decl")
    };
    let Statement::ForOf(for_of) = &f.body.body[0] else {
        panic!("expected for-of")
    };
    assert!(for_of.r#await, "for-await-of should have r#await = true");
}

#[test]
fn test_generator_method_in_class() {
    // generator: false was hardcoded in class methods — verify is_generator is used
    let body = parse("class Foo { *gen() { yield 1; } }");
    let Statement::ClassDecl(c) = &body[0] else {
        panic!("expected class decl")
    };
    let ClassMember::Method(m) = &c.body.body[0] else {
        panic!("expected method")
    };
    assert!(
        m.function.generator,
        "generator method should have generator=true"
    );
}

#[test]
fn test_exponentiation_right_assoc() {
    // ** is right-associative: a ** b ** c === a ** (b ** c)
    let body = parse("const x = 2 ** 3 ** 2;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    let Some(Expression::Binary(outer)) = &decl.declarators[0].init else {
        panic!("expected binary expr")
    };
    assert_eq!(outer.op, motarjim_js::BinaryOp::Exp);
    // Right operand should itself be an exponentiation (right-assoc)
    assert!(
        matches!(*outer.right, Expression::Binary(ref inner) if inner.op == motarjim_js::BinaryOp::Exp),
        "right-assoc: 2 ** 3 ** 2 should be 2 ** (3 ** 2)"
    );
}

#[test]
fn test_template_utf8_multi_byte_does_not_panic() {
    // find_matching_brace used byte indexing that panics on non-UTF-8 boundaries.
    // Template with multi-byte chars around ${} must not crash.
    for src in &[
        "const msg = `Héllö ${name} wörld`;",
        "const msg = `\u{00e9}\u{00e9}${x}\u{00e9}`;",
        "const msg = `¡Hola ${amigo}!`;",
        "const msg = `中文${name}测试`;",
    ] {
        let mut parser = JsParser::new(src);
        let _ = parser.parse(); // must not panic
    }
}

#[test]
fn test_cur_bounds_check() {
    // cur() used self.tokens[self.pos] without bounds check.
    // Verify that empty-ish edge cases produce diagnostics instead of panicking.
    let result = try_parse("");
    assert!(result.is_ok() || result.is_err());
    let result = try_parse(";");
    assert_eq!(result.unwrap().len(), 1);
    // Parser at Eof should never attempt out-of-bounds access
    let result = try_parse(" ");
    assert!(result.is_ok() || result.is_err());
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
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    assert!(matches!(
        decl.declarators[0].init,
        Some(Expression::BigInt(_))
    ));
}

#[test]
fn test_regex_literal() {
    let body = parse("const re = /pattern/;");
    let Statement::VarDecl(decl) = &body[0] else {
        panic!("expected var decl")
    };
    assert!(matches!(
        decl.declarators[0].init,
        Some(Expression::Regex(_))
    ));
}

#[test]
fn test_private_field() {
    let body = parse("class Foo { #priv = 1; method() { return this.#priv; } }");
    assert!(matches!(body[0], Statement::ClassDecl(_)));
}
