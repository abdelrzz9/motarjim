#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// Represents the severity level of a diagnostic message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
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

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
