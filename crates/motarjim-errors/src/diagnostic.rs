#![deny(missing_docs)]
#![forbid(unsafe_code)]

use motarjim_span::SourceSpan;

use crate::code::DiagnosticCode;
use crate::severity::Severity;

/// The main diagnostic type used throughout the compiler.
///
/// Every diagnostic includes:
/// - A severity level
/// - A registered diagnostic code
/// - A human-readable message
/// - An optional source location span
/// - Suggestions for fixes
/// - Additional notes for context
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The severity of this diagnostic.
    pub severity: Severity,
    /// The registered diagnostic code.
    pub code: DiagnosticCode,
    /// The main diagnostic message.
    pub message: String,
    /// An optional source location where the diagnostic occurred.
    pub span: Option<SourceSpan>,
    /// Possible suggestions for fixing the issue.
    pub suggestions: Vec<String>,
    /// Additional notes providing context.
    pub notes: Vec<String>,
    /// Optional hint text with a suggested fix.
    pub hint: Option<String>,
    /// Optional URL to documentation about this error.
    pub docs_url: Option<String>,
}

impl Diagnostic {
    /// Creates a new diagnostic with the given severity, code, and message.
    #[must_use]
    pub fn new(severity: Severity, code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            span: None,
            suggestions: Vec::new(),
            notes: Vec::new(),
            hint: None,
            docs_url: None,
        }
    }

    /// Attaches a source span to this diagnostic (builder pattern).
    #[must_use]
    pub const fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    /// Adds a suggestion to this diagnostic (builder pattern).
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// Adds a note to this diagnostic (builder pattern).
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Attaches a hint to this diagnostic (builder pattern).
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Attaches a documentation URL to this diagnostic (builder pattern).
    #[must_use]
    pub fn with_docs(mut self, url: impl Into<String>) -> Self {
        self.docs_url = Some(url.into());
        self
    }

    /// Returns the severity of this diagnostic.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &DiagnosticCode {
        &self.code
    }

    /// Returns the diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// A collection of diagnostics accumulated during compilation.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticBag {
    /// The underlying diagnostic entries.
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    /// Creates a new, empty diagnostic bag.
    #[must_use]
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Adds a diagnostic to the bag.
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Extends this bag with diagnostics from an iterator.
    pub fn extend(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    /// Consumes the bag and returns the underlying vector of diagnostics.
    #[must_use]
    pub fn into_inner(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Returns a slice of all diagnostics in the bag.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns `true` if any diagnostic in the bag has error severity.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity.is_error())
    }

    /// Returns the number of error-severity diagnostics.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity.is_error())
            .count()
    }

    /// Returns the number of warning-severity diagnostics.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity.is_warning())
            .count()
    }

    /// Returns `true` if the bag contains no diagnostics.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns the total number of diagnostics in the bag.
    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Pushes a diagnostic into the bag (alias for `add`).
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Convenience method to push an error diagnostic.
    pub fn push_error(&mut self, code: DiagnosticCode, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::new(Severity::Error, code, message));
    }

    /// Convenience method to push a warning diagnostic.
    pub fn push_warning(&mut self, code: DiagnosticCode, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::new(Severity::Warning, code, message));
    }

    /// Convenience method to push an info diagnostic.
    pub fn push_info(&mut self, code: DiagnosticCode, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::new(Severity::Info, code, message));
    }

    /// Returns `true` if any diagnostic has warning severity.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity.is_warning())
    }

    /// Returns an iterator over references to diagnostics.
    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.diagnostics.iter()
    }

    /// Consumes the bag and returns the underlying vector.
    #[must_use]
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Removes all diagnostics from the bag.
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

impl Extend<Diagnostic> for DiagnosticBag {
    fn extend<T: IntoIterator<Item = Diagnostic>>(&mut self, iter: T) {
        self.diagnostics.extend(iter);
    }
}

impl<'a> IntoIterator for &'a DiagnosticBag {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}

impl IntoIterator for DiagnosticBag {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl From<DiagnosticBag> for Vec<Diagnostic> {
    fn from(bag: DiagnosticBag) -> Self {
        bag.diagnostics
    }
}
