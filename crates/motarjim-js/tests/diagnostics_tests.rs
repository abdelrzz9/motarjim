use motarjim_diag::Severity;
use motarjim_js::{JsDiagnosticCode, JsDiagnostic};

#[test]
fn test_diagnostic_code_values() {
    let cases: Vec<(JsDiagnosticCode, u32)> = vec![
        (JsDiagnosticCode::JS_UNEXPECTED_TOKEN, 1001),
        (JsDiagnosticCode::JS_EXPECTED_TOKEN, 1002),
        (JsDiagnosticCode::JS_DUPLICATE_DECLARATION, 2001),
        (JsDiagnosticCode::JS_ASSIGN_TO_CONST, 2002),
        (JsDiagnosticCode::JS_UNDECLARED_VARIABLE, 2003),
        (JsDiagnosticCode::JS_UNREACHABLE_CODE, 2004),
        (JsDiagnosticCode::JS_MISSING_INITIALIZER, 2005),
        (JsDiagnosticCode::JS_ILLEGAL_RETURN, 2006),
        (JsDiagnosticCode::JS_ILLEGAL_BREAK, 2007),
        (JsDiagnosticCode::JS_ILLEGAL_CONTINUE, 2008),
        (JsDiagnosticCode::JS_ILLEGAL_AWAIT, 2009),
        (JsDiagnosticCode::JS_ILLEGAL_YIELD, 2010),
        (JsDiagnosticCode::JS_DUPLICATE_EXPORT, 2011),
        (JsDiagnosticCode::JS_INVALID_LHS, 3001),
        (JsDiagnosticCode::JS_TEMPLATE_LEX, 4001),
        (JsDiagnosticCode::JS_NUMBER_PARSE, 4002),
        (JsDiagnosticCode::JS_STRING_ESCAPE, 4003),
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
    let diag = JsDiagnostic::error(JsDiagnosticCode::JS_INVALID_LHS, "invalid")
        .with_span((5..10).into());
    assert_eq!(diag.span.start, 5);
}

#[test]
fn test_diagnostic_notes() {
    let diag = JsDiagnostic::warning(JsDiagnosticCode::JS_UNDECLARED_VARIABLE, "x is not defined")
        .with_span((0..1).into());
    assert_eq!(diag.notes.len(), 0);
}

#[test]
fn test_parse_error_helper() {
    let diag = JsDiagnostic::parse_error("expected ';'", (3..4).into());
    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.code.number, 1001);
    assert_eq!(diag.message, "expected ';'");
}

#[test]
fn test_diagnostic_code_into_conversion() {
    let code: motarjim_diag::DiagnosticCode = JsDiagnosticCode::JS_DUPLICATE_DECLARATION.into();
    assert_eq!(code.number, 2001);
    assert_eq!(code.prefix, "JS");
}
