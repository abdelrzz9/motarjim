#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// A registered diagnostic code with a numeric identifier and a static message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticCode {
    /// The numeric identifier (e.g., `1` for `E0001`).
    pub number: u32,
    /// A human-readable description of this diagnostic code.
    pub message: &'static str,
}

impl DiagnosticCode {
    /// Creates a new diagnostic code with the given number and static message.
    #[must_use]
    pub const fn new(number: u32, message: &'static str) -> Self {
        Self { number, message }
    }

    /// Formats the code as `E{number:04}` (e.g., `E0001`).
    #[must_use]
    pub fn formatted(&self) -> String {
        format!("E{:04}", self.number)
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{:04}", self.number)
    }
}
