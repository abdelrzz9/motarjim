use motarjim_js::{JsDiagnosticCode, JsParser, SemanticAnalyzer};

fn analyze(src: &str) -> Vec<motarjim_js::JsDiagnostic> {
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
    assert_eq!(
        diags[0].code.number,
        JsDiagnosticCode::JS_DUPLICATE_DECLARATION.0
    );
}

#[test]
fn test_duplicate_const_declaration() {
    let diags = analyze("const x = 1; const x = 2;");
    assert_eq!(diags.len(), 1);
}

#[test]
fn test_var_redeclaration_allowed() {
    let diags = analyze("var x = 1; var x = 2;");
    assert!(diags.is_empty());
}

#[test]
fn test_assignment_to_const() {
    let diags = analyze("const x = 1; x = 2;");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.number, JsDiagnosticCode::JS_ASSIGN_TO_CONST.0);
}

#[test]
fn test_assignment_to_let_allowed() {
    let diags = analyze("let x = 1; x = 2;");
    assert!(diags.is_empty());
}

#[test]
fn test_undeclared_variable_warning() {
    let diags = analyze("console.log(mystery);");
    assert_eq!(diags.len(), 1);
    assert_eq!(
        diags[0].code.number,
        JsDiagnosticCode::JS_UNDECLARED_VARIABLE.0
    );
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
fn test_for_of_binding_declared() {
    let diags = analyze("const items = [1, 2]; for (const item of items) { console.log(item); }");
    assert!(diags.is_empty());
}

#[test]
fn test_block_scoping_allows_shadowing() {
    let diags = analyze("let x = 1; { let x = 2; console.log(x); }");
    assert!(diags.is_empty());
}

#[test]
fn test_function_can_reference_outer_scope() {
    let diags = analyze("let x = 1; function f() { return x; }");
    assert!(diags.is_empty());
}

#[test]
fn test_return_outside_function() {
    let diags = analyze("return 1;");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.number, JsDiagnosticCode::JS_ILLEGAL_RETURN.0);
}

#[test]
fn test_break_outside_loop() {
    let diags = analyze("break;");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.number, JsDiagnosticCode::JS_ILLEGAL_BREAK.0);
}

#[test]
fn test_continue_outside_loop() {
    let diags = analyze("continue;");
    assert_eq!(diags.len(), 1);
    assert_eq!(
        diags[0].code.number,
        JsDiagnosticCode::JS_ILLEGAL_CONTINUE.0
    );
}

#[test]
fn test_await_outside_async_function() {
    let diags = analyze("let promise = 1; await promise;");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.number, JsDiagnosticCode::JS_ILLEGAL_AWAIT.0);
}

#[test]
fn test_nested_function_scopes() {
    let diags = analyze("function outer() { let x = 1; function inner() { let y = x; } }");
    assert!(diags.is_empty());
}

#[test]
fn test_destructuring_declares_bindings() {
    let diags = analyze("const obj = { a: 1, b: 2 }; const { a, b } = obj; console.log(a + b);");
    assert!(diags.is_empty());
}

#[test]
fn test_destructuring_array_declares() {
    let diags = analyze("const arr = [1, 2]; const [x, y] = arr; console.log(x, y);");
    assert!(diags.is_empty());
}
