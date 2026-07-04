#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]

use std::cmp::Ordering;
use std::fmt;

/// A 1-based line/column position within a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceLocation {
    /// Line number (1-based).
    pub line: u32,
    /// Column number (1-based).
    pub column: u32,
    /// Byte offset from the start of the source.
    pub offset: usize,
}

impl SourceLocation {
    /// Creates a new source location.
    #[must_use]
    pub const fn new(line: u32, column: u32, offset: usize) -> Self {
        Self { line, column, offset }
    }
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self { line: 1, column: 1, offset: 0 }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl PartialOrd for SourceLocation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceLocation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}
