//! CSS parsing error types.
//!
//! Converts Lightning CSS errors into Motarjim diagnostics.
//! No Lightning CSS error types are exposed in the public API.

use motarjim_diag::codes;
use motarjim_diag::{Diagnostic, DiagnosticBag, Severity, SourceSpan};

/// A CSS parsing error with diagnostic information.
#[derive(Debug, Clone)]
pub struct CssError {
    /// The diagnostic bag containing all errors and warnings.
    pub diagnostics: DiagnosticBag,
}

impl CssError {
    /// Creates a new CSS error from a single diagnostic message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        let mut bag = DiagnosticBag::new();
        bag.push_error(codes::CSS_PARSE_ERROR, message);
        Self { diagnostics: bag }
    }

    /// Creates a CSS error from an existing diagnostic bag.
    #[must_use]
    pub const fn from_bag(diagnostics: DiagnosticBag) -> Self {
        Self { diagnostics }
    }

    /// Creates a CSS error with a specific code and message.
    #[must_use]
    pub fn with_code(
        code: motarjim_diag::DiagnosticCode,
        message: impl Into<String>,
    ) -> Self {
        let mut bag = DiagnosticBag::new();
        bag.push_error(code, message);
        Self { diagnostics: bag }
    }

    /// Creates a CSS error with span information.
    #[must_use]
    pub fn with_span(message: impl Into<String>, span: SourceSpan) -> Self {
        let mut bag = DiagnosticBag::new();
        bag.push(
            Diagnostic::new(Severity::Error, codes::CSS_PARSE_ERROR, message).with_span(span),
        );
        Self { diagnostics: bag }
    }

    /// Converts a Lightning CSS parser error into a Motarjim CSS error.
    #[must_use]
    pub fn from_lightningcss(
        _source: &str,
        error: lightningcss::error::Error<lightningcss::error::ParserError<'_>>,
    ) -> Self {
        let mut bag = DiagnosticBag::new();
        let message = error.to_string();

        if let Some(loc) = &error.loc {
            let span = SourceSpan::from_location(
                motarjim_span::SourceLocation::new(loc.line + 1, loc.column, 0),
            );
            bag.push(
                Diagnostic::new(Severity::Error, codes::CSS_PARSE_ERROR, message)
                    .with_span(span)
                    .with_suggestion("Check the CSS syntax near this location"),
            );
        } else {
            bag.push_error(codes::CSS_PARSE_ERROR, message);
        }

        Self { diagnostics: bag }
    }

    /// Returns the diagnostics from this error.
    #[must_use]
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.diagnostics.clone().into_diagnostics()
    }

    /// Returns the main error message.
    #[must_use]
    pub fn message(&self) -> String {
        self.diagnostics
            .diagnostics()
            .first()
            .map(|d| d.message.clone())
            .unwrap_or_default()
    }

    /// Returns true if this error contains any error-severity diagnostics.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }
}

impl std::fmt::Display for CssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for CssError {}

impl From<CssError> for Vec<Diagnostic> {
    fn from(error: CssError) -> Self {
        error.diagnostics.into_inner()
    }
}
