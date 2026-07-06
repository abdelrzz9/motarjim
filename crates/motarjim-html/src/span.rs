//! Source location span types.
//!
//! Spans identify ranges of bytes in the original source text.
//! They are used to correlate AST nodes back to the input for
//! error reporting and diagnostics.
//!
//! The types here are deliberately simple (byte-offset-based) to avoid
//! coupling with any particular parser infrastructure. If line/column
//! tracking is needed, it can be computed lazily from the source text
//! and these offsets.

use std::fmt;

/// A byte-offset-based source location span.
///
/// Spans identify a range of bytes in the original source text.
/// They are used to correlate AST nodes back to the input for
/// error reporting and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    /// The byte offset of the start of the span (inclusive).
    pub start: BytePos,
    /// The byte offset of the end of the span (exclusive).
    pub end: BytePos,
}

impl SourceSpan {
    /// Creates a new span from start and end byte positions.
    pub const fn new(start: BytePos, end: BytePos) -> Self {
        Self { start, end }
    }

    /// Creates a span encompassing a single byte position.
    pub const fn point(pos: BytePos) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Returns `true` if the span is empty (start == end).
    pub const fn is_empty(&self) -> bool {
        self.start.0 == self.end.0
    }

    /// Returns the length of the span in bytes.
    pub fn len(&self) -> u32 {
        self.end.0.saturating_sub(self.start.0)
    }

    /// Merges two spans into one that covers both.
    ///
    /// The resulting span starts at the earlier position and
    /// ends at the later position.
    pub fn merge(&self, other: &Self) -> Self {
        let start = self.start.min(other.start);
        let end = self.end.max(other.end);
        Self { start, end }
    }

    /// Returns `true` if the given byte position is within this span.
    pub fn contains(&self, pos: BytePos) -> bool {
        pos.0 >= self.start.0 && pos.0 < self.end.0
    }

    /// Shifts the span by the given offset.
    pub fn shift(&self, offset: u32) -> Self {
        Self {
            start: BytePos(self.start.0.saturating_add(offset)),
            end: BytePos(self.end.0.saturating_add(offset)),
        }
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start.0, self.end.0)
    }
}

impl From<(BytePos, BytePos)> for SourceSpan {
    fn from((start, end): (BytePos, BytePos)) -> Self {
        Self { start, end }
    }
}

/// A byte offset position in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BytePos(pub u32);

impl BytePos {
    /// The zero position (start of input).
    pub const ZERO: Self = BytePos(0);

    /// Creates a new byte position.
    pub const fn new(pos: u32) -> Self {
        Self(pos)
    }

    /// Returns the next byte position.
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Returns the position advanced by `n` bytes.
    pub fn advance(self, n: u32) -> Self {
        Self(self.0.saturating_add(n))
    }

    /// Returns the difference between two positions.
    pub fn diff(self, other: Self) -> i64 {
        self.0 as i64 - other.0 as i64
    }
}

impl fmt::Display for BytePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for BytePos {
    fn from(pos: u32) -> Self {
        Self(pos)
    }
}

impl From<usize> for BytePos {
    fn from(pos: usize) -> Self {
        Self(pos as u32)
    }
}

impl From<BytePos> for u32 {
    fn from(pos: BytePos) -> Self {
        pos.0
    }
}

impl From<BytePos> for usize {
    fn from(pos: BytePos) -> Self {
        pos.0 as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = SourceSpan::new(BytePos(0), BytePos(10));
        assert_eq!(span.start.0, 0);
        assert_eq!(span.end.0, 10);
    }

    #[test]
    fn test_span_point() {
        let span = SourceSpan::point(BytePos(5));
        assert!(span.is_empty());
        assert_eq!(span.len(), 0);
    }

    #[test]
    fn test_span_merge() {
        let a = SourceSpan::new(BytePos(0), BytePos(5));
        let b = SourceSpan::new(BytePos(10), BytePos(15));
        let merged = a.merge(&b);
        assert_eq!(merged.start.0, 0);
        assert_eq!(merged.end.0, 15);
    }

    #[test]
    fn test_span_contains() {
        let span = SourceSpan::new(BytePos(5), BytePos(10));
        assert!(span.contains(BytePos(7)));
        assert!(!span.contains(BytePos(4)));
        assert!(!span.contains(BytePos(10)));
    }

    #[test]
    fn test_span_shift() {
        let span = SourceSpan::new(BytePos(5), BytePos(10));
        let shifted = span.shift(3);
        assert_eq!(shifted.start.0, 8);
        assert_eq!(shifted.end.0, 13);
    }

    #[test]
    fn test_byte_pos_operations() {
        let pos = BytePos(10);
        assert_eq!(pos.next(), BytePos(11));
        assert_eq!(pos.advance(5), BytePos(15));
        assert_eq!(pos.diff(BytePos(3)), 7);
    }

    #[test]
    fn test_byte_pos_conversions() {
        let from_u32: BytePos = 42u32.into();
        assert_eq!(from_u32.0, 42);

        let from_usize: BytePos = 100usize.into();
        assert_eq!(from_usize.0, 100);

        let to_u32: u32 = BytePos(77).into();
        assert_eq!(to_u32, 77);

        let to_usize: usize = BytePos(88).into();
        assert_eq!(to_usize, 88);
    }

    #[test]
    fn test_span_display() {
        let span = SourceSpan::new(BytePos(0), BytePos(5));
        assert_eq!(format!("{span}"), "0..5");
    }

    #[test]
    fn test_span_from_tuple() {
        let span: SourceSpan = (BytePos(1), BytePos(3)).into();
        assert_eq!(span.start.0, 1);
        assert_eq!(span.end.0, 3);
    }
}
