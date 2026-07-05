use motarjim_diag::Severity;
use motarjim_js::{JsDiagnostic, JsDiagnosticCode};

#[test]
fn test_diagnostic_code_values() {
    let cases: Vec<(JsDiagnosticCode, u32)> = vec![
        (JsDiagnosticCode::JS_TEMPLATE_LEX, 1),
        (JsDiagnosticCode::JS_NUMBER_PARSE, 2),
        (JsDiagnosticCode::JS_STRING_ESCAPE, 3),
        (JsDiagnosticCode::JS_UNEXPECTED_TOKEN, 100),
        (JsDiagnosticCode::JS_EXPECTED_TOKEN, 101),
        (JsDiagnosticCode::JS_MIXING_NULLISH_AND_LOGICAL, 102),
        (JsDiagnosticCode::JS_INVALID_CLASS_MEMBER, 103),
        (JsDiagnosticCode::JS_INVALID_DESTRUCTURING_PATTERN, 104),
        (JsDiagnosticCode::JS_UNSUPPORTED_SYNTAX, 105),
        (JsDiagnosticCode::JS_INVALID_LHS, 106),
        (JsDiagnosticCode::JS_DUPLICATE_DECLARATION, 301),
        (JsDiagnosticCode::JS_ASSIGN_TO_CONST, 302),
        (JsDiagnosticCode::JS_UNDECLARED_VARIABLE, 303),
        (JsDiagnosticCode::JS_UNREACHABLE_CODE, 304),
        (JsDiagnosticCode::JS_MISSING_INITIALIZER, 305),
        (JsDiagnosticCode::JS_ILLEGAL_RETURN, 306),
        (JsDiagnosticCode::JS_ILLEGAL_BREAK, 307),
        (JsDiagnosticCode::JS_ILLEGAL_CONTINUE, 308),
        (JsDiagnosticCode::JS_ILLEGAL_AWAIT, 309),
        (JsDiagnosticCode::JS_ILLEGAL_YIELD, 310),
        (JsDiagnosticCode::JS_DUPLICATE_EXPORT, 311),
    ];
    for (code, expected) in cases {
        assert_eq!(code.0, expected, "code {:?} has number {}", code, expected);
    }
}

#[test]
fn test_diagnostic_error_severity() {
    let diag = JsDiagnostic::error(JsDiagnosticCode::JS_DUPLICATE_DECLARATION, "duplicate");
    assert_eq!(diag.severity, Severity::Error);
}

#[test]
fn test_diagnostic_warning_severity() {
    let diag = JsDiagnostic::warning(JsDiagnosticCode::JS_UNDECLARED_VARIABLE, "undeclared");
    assert_eq!(diag.severity, Severity::Warning);
}

#[test]
fn test_diagnostic_with_span() {
    let diag =
        JsDiagnostic::error(JsDiagnosticCode::JS_INVALID_LHS, "invalid").with_span((5..10).into());
    assert_eq!(diag.span.start.offset, 5);
}

#[test]
fn test_diagnostic_notes() {
    let diag = JsDiagnostic::warning(JsDiagnosticCode::JS_UNDECLARED_VARIABLE, "x is not defined")
        .with_span((0..1).into());
    assert_eq!(diag.notes.len(), 0);
}

#[test]
fn test_diagnostic_help_and_suggestions() {
    let diag = JsDiagnostic::error(
        JsDiagnosticCode::JS_MIXING_NULLISH_AND_LOGICAL,
        "mixing ?? with ||",
    )
    .with_help("wrap the expression in parentheses to clarify precedence")
    .with_suggestion("(a ?? b) || c");
    assert_eq!(
        diag.help.as_deref(),
        Some("wrap the expression in parentheses to clarify precedence")
    );
    assert_eq!(diag.suggestions.len(), 1);
}

#[test]
fn test_parse_error_helper() {
    let diag = JsDiagnostic::parse_error("expected ';'", (3..4).into());
    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.code.number, 100);
    assert_eq!(diag.message, "expected ';'");
}

#[test]
fn test_diagnostic_code_into_conversion() {
    let code: motarjim_diag::DiagnosticCode = JsDiagnosticCode::JS_DUPLICATE_DECLARATION.into();
    assert_eq!(code.number, 301);
    assert_eq!(code.prefix, "JS");
}

#[test]
fn test_from_js_diagnostic_preserves_help_and_suggestions() {
    let js_diag = JsDiagnostic::error(JsDiagnosticCode::JS_MIXING_NULLISH_AND_LOGICAL, "mixed")
        .with_help("add parentheses")
        .with_suggestion("(a ?? b) || c");
    let diag: motarjim_diag::Diagnostic = js_diag.into();
    assert!(diag.suggestions.iter().any(|s| s == "add parentheses"));
    assert!(diag.suggestions.iter().any(|s| s == "(a ?? b) || c"));
}
