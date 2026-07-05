//! Diagnostic codes, severity, and structured error reporting.
//!
//! # Code scheme
//!
//! | Range       | Category          |
//! |-------------|-------------------|
//! | E0001–E0099 | Lexer errors      |
//! | E0100–E0299 | Parser errors      |
//! | E0300–E0499 | Semantic/analysis  |
//! | E0500–E0599 | Transform errors   |
//! | E0600–E0799 | Reserved           |

use motarjim_diag::{Diagnostic, DiagnosticCode, Severity};
use motarjim_span::SourceSpan;

#[derive(Debug, Clone, Copy)]
pub struct JsDiagnosticCode(pub u32);

/// Lexer errors: E0001–E0099
impl JsDiagnosticCode {
    // Lexer errors ────────────────────────────────────────── E0001–E0099
    pub const JS_TEMPLATE_LEX: Self = Self(1);
    pub const JS_NUMBER_PARSE: Self = Self(2);
    pub const JS_STRING_ESCAPE: Self = Self(3);

    // Parser errors ───────────────────────────────────────── E0100–E0299
    pub const JS_UNEXPECTED_TOKEN: Self = Self(100);
    pub const JS_EXPECTED_TOKEN: Self = Self(101);
    pub const JS_MIXING_NULLISH_AND_LOGICAL: Self = Self(102);
    pub const JS_INVALID_CLASS_MEMBER: Self = Self(103);
    pub const JS_INVALID_DESTRUCTURING_PATTERN: Self = Self(104);
    pub const JS_UNSUPPORTED_SYNTAX: Self = Self(105);
    pub const JS_INVALID_LHS: Self = Self(106);

    // Semantic/analysis errors ────────────────────────────── E0300–E0499
    pub const JS_DUPLICATE_DECLARATION: Self = Self(301);
    pub const JS_ASSIGN_TO_CONST: Self = Self(302);
    pub const JS_UNDECLARED_VARIABLE: Self = Self(303);
    pub const JS_UNREACHABLE_CODE: Self = Self(304);
    pub const JS_MISSING_INITIALIZER: Self = Self(305);
    pub const JS_ILLEGAL_RETURN: Self = Self(306);
    pub const JS_ILLEGAL_BREAK: Self = Self(307);
    pub const JS_ILLEGAL_CONTINUE: Self = Self(308);
    pub const JS_ILLEGAL_AWAIT: Self = Self(309);
    pub const JS_ILLEGAL_YIELD: Self = Self(310);
    pub const JS_DUPLICATE_EXPORT: Self = Self(311);
}

impl From<JsDiagnosticCode> for DiagnosticCode {
    fn from(code: JsDiagnosticCode) -> Self {
        DiagnosticCode::new(code.0, "JS diagnostic").with_prefix("JS")
    }
}

#[derive(Debug, Clone)]
pub struct JsDiagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub span: SourceSpan,
    pub notes: Vec<String>,
    pub help: Option<String>,
    pub suggestions: Vec<String>,
}

impl JsDiagnostic {
    pub fn error(code: JsDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            span: SourceSpan::default(),
            notes: Vec::new(),
            help: None,
            suggestions: Vec::new(),
        }
    }

    pub fn warning(code: JsDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            span: SourceSpan::default(),
            notes: Vec::new(),
            help: None,
            suggestions: Vec::new(),
        }
    }

    pub fn parse_error(message: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            severity: Severity::Error,
            code: JsDiagnosticCode::JS_UNEXPECTED_TOKEN.into(),
            message: message.into(),
            span,
            notes: Vec::new(),
            help: None,
            suggestions: Vec::new(),
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

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }
}

impl From<JsDiagnostic> for Diagnostic {
    fn from(d: JsDiagnostic) -> Self {
        let mut diag = Diagnostic::new(d.severity, d.code, d.message).with_span(d.span);
        for note in d.notes {
            diag = diag.with_note(note);
        }
        for suggestion in d.suggestions {
            diag = diag.with_suggestion(suggestion);
        }
        if let Some(help) = d.help {
            diag = diag.with_suggestion(help);
        }
        diag
    }
}
