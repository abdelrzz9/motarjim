//! Diagnostic codes, severity, and structured error reporting.

use motarjim_diag::{Diagnostic, Severity, DiagnosticCode};
use motarjim_span::SourceSpan;

pub struct JsDiagnosticCode(pub u32);

impl JsDiagnosticCode {
    pub const JS_UNEXPECTED_TOKEN: Self = Self(1001);
    pub const JS_EXPECTED_TOKEN: Self = Self(1002);
    pub const JS_DUPLICATE_DECLARATION: Self = Self(2001);
    pub const JS_ASSIGN_TO_CONST: Self = Self(2002);
    pub const JS_UNDECLARED_VARIABLE: Self = Self(2003);
    pub const JS_UNREACHABLE_CODE: Self = Self(2004);
    pub const JS_MISSING_INITIALIZER: Self = Self(2005);
    pub const JS_ILLEGAL_RETURN: Self = Self(2006);
    pub const JS_ILLEGAL_BREAK: Self = Self(2007);
    pub const JS_ILLEGAL_CONTINUE: Self = Self(2008);
    pub const JS_ILLEGAL_AWAIT: Self = Self(2009);
    pub const JS_ILLEGAL_YIELD: Self = Self(2010);
    pub const JS_DUPLICATE_EXPORT: Self = Self(2011);
    pub const JS_INVALID_LHS: Self = Self(3001);
    pub const JS_TEMPLATE_LEX: Self = Self(4001);
    pub const JS_NUMBER_PARSE: Self = Self(4002);
    pub const JS_STRING_ESCAPE: Self = Self(4003);
}

impl From<JsDiagnosticCode> for DiagnosticCode {
    fn from(code: JsDiagnosticCode) -> Self {
        DiagnosticCode::new(code.0, "JS diagnostic")
    }
}

#[derive(Debug, Clone)]
pub struct JsDiagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub span: SourceSpan,
    pub notes: Vec<String>,
}

impl JsDiagnostic {
    pub fn error(code: JsDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            span: SourceSpan::default(),
            notes: Vec::new(),
        }
    }

    pub fn warning(code: JsDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            span: SourceSpan::default(),
            notes: Vec::new(),
        }
    }

    pub fn parse_error(message: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            severity: Severity::Error,
            code: JsDiagnosticCode::JS_UNEXPECTED_TOKEN.into(),
            message: message.into(),
            span,
            notes: Vec::new(),
        }
    }

    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = span;
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

impl From<JsDiagnostic> for Diagnostic {
    fn from(d: JsDiagnostic) -> Self {
        let mut diag = Diagnostic::new(d.severity, d.code, d.message).with_span(d.span);
        for note in d.notes {
            diag = diag.with_note(note);
        }
        diag
    }
}
