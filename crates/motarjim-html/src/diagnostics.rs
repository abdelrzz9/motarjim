//! Structured parse errors and diagnostics for the HTML parser.
//!
//! The diagnostics system uses a bag/accumulator pattern: during parsing,
//! all errors, warnings, and info messages are collected into a
//! [`DiagnosticBag`]. After parsing, the caller can inspect the
//! diagnostics to determine whether the result is usable.

use std::fmt;

use crate::span::SourceSpan;

/// The severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// An informational message.
    Info,
    /// A warning about potential issues.
    Warning,
    /// An error that prevents successful compilation.
    Error,
}

impl Severity {
    /// Returns `true` if this severity is at least as severe as `other`.
    pub fn is_at_least(&self, other: Severity) -> bool {
        *self as u8 >= other as u8
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// The kind of parse error encountered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// A generic syntax error.
    Syntax,
    /// Malformed HTML construct.
    MalformedHtml,
    /// An unsupported construct or feature.
    Unsupported,
    /// An unexpected end of input.
    UnexpectedEof,
    /// A mismatched or unclosed tag.
    MismatchedTag,
    /// An invalid character in the input.
    InvalidCharacter,
    /// A duplicate attribute.
    DuplicateAttribute,
    /// An unexpected token in the current context.
    UnexpectedToken,
    /// Errors reported by the html5ever parser (string payload).
    Html5ever(String),
}

impl ParseErrorKind {
    /// Returns a human-readable label for this error kind.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Syntax => "syntax error",
            Self::MalformedHtml => "malformed HTML",
            Self::Unsupported => "unsupported construct",
            Self::UnexpectedEof => "unexpected end of input",
            Self::MismatchedTag => "mismatched tag",
            Self::InvalidCharacter => "invalid character",
            Self::DuplicateAttribute => "duplicate attribute",
            Self::UnexpectedToken => "unexpected token",
            Self::Html5ever(_) => "html5 parse error",
        }
    }
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Html5ever(msg) => write!(f, "parse error: {msg}"),
            _ => write!(f, "{}", self.label()),
        }
    }
}

/// A parse error with location information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The kind of error.
    pub kind: ParseErrorKind,
    /// A human-readable error message.
    pub message: String,
    /// The source location of the error, if available.
    pub span: Option<SourceSpan>,
    /// The severity of the error.
    pub severity: Severity,
}

impl ParseError {
    /// Creates a new parse error with `Error` severity.
    pub fn new(kind: ParseErrorKind, message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            severity: Severity::Error,
        }
    }

    /// Creates a new warning diagnostic.
    pub fn warning(
        kind: ParseErrorKind,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            severity: Severity::Warning,
        }
    }

    /// Creates a new informational diagnostic.
    pub fn info(
        kind: ParseErrorKind,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            severity: Severity::Info,
        }
    }

    /// Sets the severity of this diagnostic.
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Returns `true` if this diagnostic is an error.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Returns `true` if this diagnostic is a warning or error.
    pub fn is_warning_or_error(&self) -> bool {
        self.severity >= Severity::Warning
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = self.span {
            write!(
                f,
                "[{}] at {}..{}: {}",
                self.severity, span.start.0, span.end.0, self.message
            )
        } else {
            write!(f, "[{}] {}", self.severity, self.message)
        }
    }
}

impl std::error::Error for ParseError {}

/// A collection of parse diagnostics.
///
/// # Usage
///
/// During parsing, all diagnostics (errors, warnings, info) are pushed
/// into a [`DiagnosticBag`]. After parsing, the caller can inspect the
/// bag to decide whether the result is usable.
///
/// If the bag contains any errors, the parsed document may be incomplete
/// or malformed. Warnings indicate non-fatal issues that the caller may
/// want to report to the user.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticBag {
    /// The collected diagnostics.
    pub errors: Vec<ParseError>,
}

impl DiagnosticBag {
    /// Creates a new empty diagnostic bag.
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Creates a new diagnostic bag with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            errors: Vec::with_capacity(capacity),
        }
    }

    /// Adds a diagnostic to the bag.
    pub fn push(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    /// Returns `true` if there are any errors (not warnings).
    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.is_error())
    }

    /// Returns `true` if there are any warnings or errors.
    pub fn has_warnings_or_errors(&self) -> bool {
        self.errors.iter().any(|e| e.is_warning_or_error())
    }

    /// Returns all diagnostics.
    pub fn diagnostics(&self) -> &[ParseError] {
        &self.errors
    }

    /// Consumes the bag and returns all diagnostics.
    pub fn into_diagnostics(self) -> Vec<ParseError> {
        self.errors
    }

    /// Returns the number of diagnostics.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns `true` if there are no diagnostics.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Removes all diagnostics from the bag.
    pub fn clear(&mut self) {
        self.errors.clear();
    }

    /// Returns an iterator over all errors only.
    pub fn errors_only(&self) -> impl Iterator<Item = &ParseError> {
        self.errors.iter().filter(|e| e.is_error())
    }

    /// Returns an iterator over all warnings only.
    pub fn warnings_only(&self) -> impl Iterator<Item = &ParseError> {
        self.errors
            .iter()
            .filter(|e| e.severity == Severity::Warning)
    }

    /// Consumes the bag and returns the first error, if any.
    pub fn into_first_error(self) -> Option<ParseError> {
        self.errors.into_iter().find(|e| e.is_error())
    }

    /// Merges another diagnostic bag into this one.
    pub fn merge(&mut self, other: DiagnosticBag) {
        self.errors.extend(other.errors);
    }
}

impl From<Vec<ParseError>> for DiagnosticBag {
    fn from(errors: Vec<ParseError>) -> Self {
        Self { errors }
    }
}

impl Extend<ParseError> for DiagnosticBag {
    fn extend<T: IntoIterator<Item = ParseError>>(&mut self, iter: T) {
        self.errors.extend(iter);
    }
}

impl IntoIterator for DiagnosticBag {
    type Item = ParseError;
    type IntoIter = std::vec::IntoIter<ParseError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_severity_is_at_least() {
        assert!(Severity::Error.is_at_least(Severity::Warning));
        assert!(Severity::Error.is_at_least(Severity::Error));
        assert!(!Severity::Info.is_at_least(Severity::Warning));
    }

    #[test]
    fn test_parse_error_creation() {
        let err = ParseError::new(ParseErrorKind::Syntax, "bad html", None);
        assert!(err.is_error());
        assert_eq!(err.severity, Severity::Error);
    }

    #[test]
    fn test_warning_creation() {
        let warn = ParseError::warning(ParseErrorKind::DuplicateAttribute, "duplicate class", None);
        assert!(!warn.is_error());
        assert!(warn.is_warning_or_error());
    }

    #[test]
    fn test_info_creation() {
        let info = ParseError::info(ParseErrorKind::Unsupported, "feature not supported", None);
        assert!(!info.is_warning_or_error());
    }

    #[test]
    fn test_diagnostic_bag() {
        let mut bag = DiagnosticBag::new();
        bag.push(ParseError::new(ParseErrorKind::Syntax, "error 1", None));
        bag.push(ParseError::warning(
            ParseErrorKind::MalformedHtml,
            "warn 1",
            None,
        ));
        assert!(bag.has_errors());
        assert!(bag.has_warnings_or_errors());
        assert_eq!(bag.len(), 2);
    }

    #[test]
    fn test_diagnostic_bag_clear() {
        let mut bag = DiagnosticBag::new();
        bag.push(ParseError::new(ParseErrorKind::Syntax, "error", None));
        bag.clear();
        assert!(bag.is_empty());
    }

    #[test]
    fn test_diagnostic_bag_errors_only() {
        let mut bag = DiagnosticBag::new();
        bag.push(ParseError::new(ParseErrorKind::Syntax, "error", None));
        bag.push(ParseError::warning(
            ParseErrorKind::MalformedHtml,
            "warn",
            None,
        ));
        assert_eq!(bag.errors_only().count(), 1);
        assert_eq!(bag.warnings_only().count(), 1);
    }

    #[test]
    fn test_diagnostic_bag_merge() {
        let mut bag1 = DiagnosticBag::new();
        bag1.push(ParseError::new(ParseErrorKind::Syntax, "err1", None));
        let mut bag2 = DiagnosticBag::new();
        bag2.push(ParseError::new(ParseErrorKind::Syntax, "err2", None));
        bag1.merge(bag2);
        assert_eq!(bag1.len(), 2);
    }

    #[test]
    fn test_diagnostic_bag_into_first_error() {
        let mut bag = DiagnosticBag::new();
        bag.push(ParseError::warning(
            ParseErrorKind::MalformedHtml,
            "warn",
            None,
        ));
        bag.push(ParseError::new(
            ParseErrorKind::Syntax,
            "actual error",
            None,
        ));
        let first = bag.into_first_error();
        assert!(first.is_some());
        assert_eq!(first.unwrap().message, "actual error");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::new(ParseErrorKind::Syntax, "test error", None);
        let display = format!("{err}");
        assert!(display.contains("test error"));
    }

    #[test]
    fn test_parse_error_kind_display() {
        assert_eq!(ParseErrorKind::Syntax.to_string(), "syntax error");
        assert_eq!(ParseErrorKind::MalformedHtml.to_string(), "malformed HTML");
        assert_eq!(
            ParseErrorKind::Html5ever("msg".into()).to_string(),
            "parse error: msg"
        );
    }

    #[test]
    fn test_from_vec() {
        let bag: DiagnosticBag = vec![
            ParseError::new(ParseErrorKind::Syntax, "e1", None),
            ParseError::new(ParseErrorKind::Syntax, "e2", None),
        ]
        .into();
        assert_eq!(bag.len(), 2);
    }

    #[test]
    fn test_extend() {
        let mut bag = DiagnosticBag::new();
        bag.extend(vec![
            ParseError::new(ParseErrorKind::Syntax, "e1", None),
            ParseError::new(ParseErrorKind::Syntax, "e2", None),
        ]);
        assert_eq!(bag.len(), 2);
    }
}
