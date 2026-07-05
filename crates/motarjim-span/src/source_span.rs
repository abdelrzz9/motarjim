#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]

use crate::SourceLocation;
use std::fmt;
use std::ops::Range;

/// A range between two [`SourceLocation`]s in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceSpan {
    /// The start position (inclusive).
    pub start: SourceLocation,
    /// The end position (exclusive).
    pub end: SourceLocation,
}

impl SourceSpan {
    /// Creates a new source span.
    #[must_use]
    pub const fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    /// Creates a span from a single location (zero-width span).
    #[must_use]
    pub const fn from_location(loc: SourceLocation) -> Self {
        Self {
            start: loc,
            end: loc,
        }
    }

    /// Returns the length of the span in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Returns true if the span has zero length.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start.offset >= self.end.offset
    }

    /// Merges two spans into one that covers both.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let start = if self.start.offset < other.start.offset {
            self.start
        } else {
            other.start
        };
        let end = if self.end.offset > other.end.offset {
            self.end
        } else {
            other.end
        };
        Self { start, end }
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.line == self.end.line {
            write!(
                f,
                "{}:{}-{}",
                self.start.line, self.start.column, self.end.column
            )
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

impl Default for SourceSpan {
    fn default() -> Self {
        Self {
            start: SourceLocation::default(),
            end: SourceLocation::default(),
        }
    }
}

impl From<Range<usize>> for SourceSpan {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: SourceLocation {
                offset: range.start,
                ..SourceLocation::default()
            },
            end: SourceLocation {
                offset: range.end,
                ..SourceLocation::default()
            },
        }
    }
}
