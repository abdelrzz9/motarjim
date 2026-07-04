use motarjim_diag::{SourceLocation, SourceSpan};

/// Character cursor for zero-copy tokenization.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    /// The source text.
    source: &'a str,
    /// Current byte offset.
    pos: usize,
    /// Current line number (1-based).
    line: u32,
    /// Current column number (1-based).
    col: u32,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor at the start of the source.
    #[must_use]
    pub const fn new(source: &'a str) -> Self {
        Self { source, pos: 0, line: 1, col: 1 }
    }

    /// Returns the current byte position.
    #[must_use]
    pub const fn pos(&self) -> usize { self.pos }

    /// Returns the current line number (1-based).
    #[must_use]
    pub const fn line(&self) -> u32 { self.line }

    /// Returns the current column number (1-based).
    #[must_use]
    pub const fn col(&self) -> u32 { self.col }

    /// Returns the remaining source text.
    #[must_use]
    pub fn remaining(&self) -> &'a str { &self.source[self.pos..] }

    /// Returns `true` if the cursor is at the end of input.
    #[must_use]
    pub const fn is_eof(&self) -> bool { self.pos >= self.source.len() }

    /// Peeks at the current character without consuming it.
    #[must_use]
    pub fn peek(&self) -> Option<char> { self.remaining().chars().next() }

    /// Peeks ahead by `n` characters.
    #[must_use]
    pub fn peek_at(&self, n: usize) -> Option<char> {
        self.remaining().chars().nth(n)
    }

    /// Consumes and returns the next character.
    pub fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        let len = c.len_utf8();
        self.pos += len;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    /// Consumes characters while the predicate returns true.
    pub fn take_while(&mut self, mut pred: impl FnMut(char) -> bool) -> &'a str {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if !pred(c) { break; }
            self.advance();
        }
        &self.source[start..self.pos]
    }

    /// Skips whitespace characters.
    pub fn skip_whitespace(&mut self) {
        self.take_while(|c| c.is_ascii_whitespace());
    }

    /// Creates a source span from the given start position to the current position.
    #[must_use] 
    pub const fn span_since(&self, start: usize) -> SourceSpan {
        let start_loc = SourceLocation { line: 0, column: 0, offset: start as u32 };
        let end_loc = SourceLocation { line: 0, column: 0, offset: self.pos as u32 };
        SourceSpan { start: start_loc, end: end_loc }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor() {
        let mut c = Cursor::new("abc");
        assert_eq!(c.peek(), Some('a'));
        assert_eq!(c.advance(), Some('a'));
        assert_eq!(c.advance(), Some('b'));
        assert_eq!(c.advance(), Some('c'));
        assert!(c.is_eof());
        assert_eq!(c.advance(), None);
    }

    #[test]
    fn test_take_while() {
        let mut c = Cursor::new("123abc");
        let nums = c.take_while(|c| c.is_ascii_digit());
        assert_eq!(nums, "123");
        assert_eq!(c.peek(), Some('a'));
    }
}
