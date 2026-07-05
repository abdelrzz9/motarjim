#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// A registered diagnostic code with a numeric identifier and a static message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticCode {
    /// The numeric identifier (e.g., `1` for `E0001`).
    pub number: u32,
    /// A short prefix indicating the subsystem (e.g., `"JS"`, `"CSS"`).
    pub prefix: &'static str,
    /// A human-readable description of this diagnostic code.
    pub message: &'static str,
}

impl DiagnosticCode {
    /// Creates a new diagnostic code with the given number and static message.
    /// The `prefix` defaults to an empty string; use [`with_prefix`] to set it.
    #[must_use]
    pub const fn new(number: u32, message: &'static str) -> Self {
        Self {
            number,
            prefix: "",
            message,
        }
    }

    /// Sets the subsystem prefix on this diagnostic code.
    #[must_use]
    pub const fn with_prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = prefix;
        self
    }

    /// Formats the code as `{prefix}{number:04}` (e.g., `E0001`).
    #[must_use]
    pub fn formatted(&self) -> String {
        if self.prefix.is_empty() {
            format!("E{:04}", self.number)
        } else {
            format!("{}{:04}", self.prefix, self.number)
        }
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.prefix.is_empty() {
            write!(f, "E{:04}", self.number)
        } else {
            write!(f, "{}{:04}", self.prefix, self.number)
        }
    }
}
