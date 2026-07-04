use crate::span::SourceSpan;

/// Represents the severity of a diagnostic message.
///
/// Ordering is by priority: Error > Warning > Info > Hint > Note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// An error that prevents compilation from succeeding.
    Error,
    /// A warning about a potential issue that does not block compilation.
    Warning,
    /// An informational message about the compilation process.
    Info,
    /// A hint for improving code style or performance.
    Hint,
    /// An additional note providing context about a diagnostic.
    Note,
}

impl Severity {
    /// Returns `true` if this severity is `Error`.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    /// Returns `true` if this severity is `Warning`.
    #[must_use]
    pub const fn is_warning(&self) -> bool {
        matches!(self, Self::Warning)
    }

    /// Returns a static string representation of this severity.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Hint => "hint",
            Self::Note => "note",
        }
    }
}

/// A registered diagnostic code with a numeric identifier and a static message.
///
/// Codes are grouped by category:
///
/// | Range   | Category   |
/// |---------|------------|
/// | 1-99    | Parser     |
/// | 100-199 | CSS        |
/// | 200-299 | Semantic   |
/// | 300-399 | A11y       |
/// | 400-499 | IR         |
/// | 500-599 | Generator  |
/// | 600-699 | Config     |
/// | 700-799 | JavaScript |
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticCode {
    /// The numeric identifier (e.g. `1` for `E0001`).
    pub number: u32,
    /// A human-readable description of this diagnostic code.
    pub message: &'static str,
}

#[cfg(feature = "json")]
impl serde::Serialize for DiagnosticCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("DiagnosticCode", 2)?;
        s.serialize_field("number", &self.number)?;
        s.serialize_field("message", self.message)?;
        s.end()
    }
}

#[cfg(feature = "json")]
impl<'de> serde::Deserialize<'de> for DiagnosticCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct DiagnosticCodeHelper {
            number: u32,
            #[serde(default)]
            message: String,
        }
        let helper = DiagnosticCodeHelper::deserialize(deserializer)?;
        // Leak the string to get a &'static str for the deserialized code.
        // This is acceptable because diagnostic codes are small and few.
        let message: &'static str = Box::leak(helper.message.into_boxed_str());
        Ok(Self::new(helper.number, message))
    }
}

impl DiagnosticCode {
    /// Creates a new diagnostic code with the given number and static message.
    #[must_use]
    pub const fn new(number: u32, message: &'static str) -> Self {
        Self { number, message }
    }
}

/// A single diagnostic message with optional source location, suggestions, and notes.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    /// The severity level of this diagnostic.
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
