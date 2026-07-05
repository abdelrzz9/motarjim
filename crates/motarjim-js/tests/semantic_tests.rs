use motarjim_js::{JsDiagnosticCode, JsParser, SemanticAnalyzer};

fn analyze(src: &str) -> Vec<motarjim_js::JsDiagnostic> {
    let mut parser = JsParser::new(src);
    let program = parser.parse().expect("should parse");
    SemanticAnalyzer::new().analyze(&program)
}

fn analyze_module(src: &str) -> Vec<motarjim_js::JsDiagnostic> {
    let mut parser = JsParser::new(src);
    let program = parser.parse_module().expect("should parse");
    SemanticAnalyzer::new().analyze(&program)
}

// ── 1a: var hoisting ────────────────────────────────────────────────

#[test]
fn test_var_in_block_visible_after_block() {
    let diags = analyze("function f() { { var x = 1; } return x; }");
    assert!(
        diags.is_empty(),
        "var should be visible after block: {:?}",
        diags
    );
}

#[test]
fn test_var_in_for_loop_visible_after_loop() {
    let diags = analyze("function f() { for (var x = 0; x < 1; x++) {} return x; }");
    assert!(
        diags.is_empty(),
        "var should be visible after for loop: {:?}",
        diags
    );
}

#[test]
fn test_let_in_block_not_visible_after_block() {
    let diags = analyze("function f() { { let x = 1; } return x; }");
    assert_eq!(diags.len(), 1);
    assert_eq!(
        diags[0].code.number,
        JsDiagnosticCode::JS_UNDECLARED_VARIABLE.0
    );
}

// ── 1b: await validity ──────────────────────────────────────────────

#[test]
fn test_await_in_async_function_allowed() {
    let diags = analyze("async function f() { await 1; }");
    assert!(
        diags.is_empty(),
        "await in async fn should be ok: {:?}",
        diags
    );
}

#[test]
fn test_await_in_non_async_nested_function_errors() {
    let diags = analyze("async function outer() { function inner() { await 1; } }");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.number, JsDiagnosticCode::JS_ILLEGAL_AWAIT.0);
}

#[test]
fn test_await_in_async_arrow_allowed() {
    let diags = analyze("const f = async () => await 1;");
    assert!(
        diags.is_empty(),
        "await in async arrow should be ok: {:?}",
        diags
    );
}

// ── 1c: param VarKind ───────────────────────────────────────────────

#[test]
fn test_non_strict_params_use_var() {
    let diags = analyze("function f(a) { var a = 2; return a; }");
    assert!(
        diags.is_empty(),
        "non-strict: var redecl of param should be ok: {:?}",
        diags
    );
}

#[test]
fn test_strict_params_use_let() {
    let diags = analyze_module("function f(a) { let a = 2; return a; }");
    assert_eq!(diags.len(), 1);
    assert_eq!(
        diags[0].code.number,
        JsDiagnosticCode::JS_DUPLICATE_DECLARATION.0
    );
}

// ── 1d: super handling ──────────────────────────────────────────────

#[test]
fn test_super_in_extends_class_method_ok() {
    let diags = analyze("class Foo extends Bar { method() { return super.x; } }");
    // We expect no JS_ILLEGAL_SUPER; Bar undeclared is fine
    assert!(
        !diags
            .iter()
            .any(|d| d.code.number == JsDiagnosticCode::JS_ILLEGAL_SUPER.0),
        "super in extends class method should not produce ILLEGAL_SUPER: {:?}",
        diags
    );
}

#[test]
fn test_super_in_class_without_extends_errors() {
    let diags = analyze("class Foo { method() { return super.x; } }");
    assert!(
        diags
            .iter()
            .any(|d| d.code.number == JsDiagnosticCode::JS_ILLEGAL_SUPER.0),
        "expected ILLEGAL_SUPER: {:?}",
        diags
    );
}

#[test]
fn test_super_outside_class_errors() {
    let diags = analyze("function f() { return super.x; }");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.number, JsDiagnosticCode::JS_ILLEGAL_SUPER.0);
}

#[test]
fn test_super_in_nested_class_without_extends_errors() {
    let diags = analyze("class A extends B { method() { class C { m2() { super.x; } } } }");
    assert!(
        diags
            .iter()
            .any(|d| d.code.number == JsDiagnosticCode::JS_ILLEGAL_SUPER.0),
        "expected ILLEGAL_SUPER: {:?}",
        diags
    );
}

// ── 2: closure capture analysis ─────────────────────────────────────

#[test]
fn test_capture_simple_closure() {
    let src = "function f() { let x = 1; return function() { return x; }; }";
    let mut parser = JsParser::new(src);
    let program = parser.parse().expect("should parse");
    let mut analyzer = SemanticAnalyzer::new();
    let diags = analyzer.analyze(&program);
    assert!(diags.is_empty(), "capture test diagnostics: {:?}", diags);
    // The inner function expr span should have 'x' as a capture
    let has_capture = analyzer
        .captures_iter()
        .any(|(_span, names)| names.contains(&"x".to_string()));
    assert!(has_capture, "expected 'x' to be captured");
}

#[test]
fn test_no_capture_for_local_var() {
    let src = "function f() { let x = 1; return x; }";
    let mut parser = JsParser::new(src);
    let program = parser.parse().expect("should parse");
    let mut analyzer = SemanticAnalyzer::new();
    let diags = analyzer.analyze(&program);
    assert!(diags.is_empty());
    // No function contains a capture because 'f' is at top level and 'x' is local
    let all_empty = analyzer
        .captures_iter()
        .all(|(_span, names)| names.is_empty());
    assert!(all_empty, "expected no captures for local-only function");
}

#[test]
fn test_capture_nested_closure_grandparent() {
    // Inner closure captures variable from the grandparent scope
    let src = "function f() { let x = 1; return function() { return function() { return x; }; }; }";
    let mut parser = JsParser::new(src);
    let program = parser.parse().expect("should parse");
    let mut analyzer = SemanticAnalyzer::new();
    let diags = analyzer.analyze(&program);
    assert!(diags.is_empty(), "nested capture diags: {:?}", diags);
    let capture_count = analyzer
        .captures_iter()
        .filter(|(_span, names)| names.contains(&"x".to_string()))
        .count();
    // Two functions should capture 'x': the inner arrow and the outer arrow should capture from grandparent
    assert!(
        capture_count >= 1,
        "expected at least one function to capture 'x'"
    );
}

#[test]
fn test_capture_sibling_closures() {
    // Multiple sibling closures capturing the same variable
    let src = "function f() { let x = 1; return [() => x, () => x]; }";
    let mut parser = JsParser::new(src);
    let program = parser.parse().expect("should parse");
    let mut analyzer = SemanticAnalyzer::new();
    let diags = analyzer.analyze(&program);
    assert!(diags.is_empty(), "sibling capture diags: {:?}", diags);
    let capture_count = analyzer
        .captures_iter()
        .filter(|(_span, names)| names.contains(&"x".to_string()))
        .count();
    assert!(
        capture_count >= 2,
        "expected at least two functions to capture 'x', got {}",
        capture_count
    );
}

#[test]
fn test_capture_reassigned_variable() {
    // Captured variable is reassigned inside the closure
    let src = "function f() { let x = 1; return function() { x = 2; }; }";
    let mut parser = JsParser::new(src);
    let program = parser.parse().expect("should parse");
    let mut analyzer = SemanticAnalyzer::new();
    let diags = analyzer.analyze(&program);
    assert!(diags.is_empty(), "reassign capture diags: {:?}", diags);
    let has_capture = analyzer
        .captures_iter()
        .any(|(_span, names)| names.contains(&"x".to_string()));
    assert!(has_capture, "expected 'x' to be captured when reassigned");
}

#[test]
fn test_capture_arrow_function() {
    // Arrow function capturing an outer variable
    let src = "function f() { let x = 1; return () => x; }";
    let mut parser = JsParser::new(src);
    let program = parser.parse().expect("should parse");
    let mut analyzer = SemanticAnalyzer::new();
    let diags = analyzer.analyze(&program);
    assert!(diags.is_empty(), "arrow capture diags: {:?}", diags);
    let has_capture = analyzer
        .captures_iter()
        .any(|(_span, names)| names.contains(&"x".to_string()));
    assert!(has_capture, "expected 'x' to be captured by arrow");
}

// ── 3: import resolution (single-file) ──────────────────────────────

#[test]
fn test_import_default_from_module() {
    let diags = analyze_module("import foo from 'bar';");
    assert!(
        diags.is_empty(),
        "default import in module should be ok: {:?}",
        diags
    );
}

#[test]
fn test_import_named_from_module() {
    let diags = analyze_module("import { foo, bar } from 'baz';");
    assert!(
        diags.is_empty(),
        "named import in module should be ok: {:?}",
        diags
    );
}

#[test]
fn test_import_default_as_named_errors() {
    let diags = analyze_module("import { default } from 'foo';");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.number, JsDiagnosticCode::JS_IMPORT_ERROR.0);
}

#[test]
fn test_import_duplicate_binding_errors() {
    let diags = analyze_module("import { foo, foo } from 'bar';");
    // Could be 2 (one for the duplicate name + one from the import spec)
    assert!(diags.iter().any(
        |d| d.code.number == JsDiagnosticCode::JS_DUPLICATE_DECLARATION.0
            || d.code.number == JsDiagnosticCode::JS_IMPORT_ERROR.0
    ));
}

#[test]
fn test_self_import_errors() {
    let diags = analyze_module("import foo from '';");
    assert!(diags
        .iter()
        .any(|d| d.code.number == JsDiagnosticCode::JS_IMPORT_ERROR.0));
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
