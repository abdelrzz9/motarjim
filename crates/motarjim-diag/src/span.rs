use std::fmt::Write;

/// A source location: 1-based line and column, with byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceLocation {
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
    /// Byte offset from the start of the source.
    pub offset: u32,
}

/// A range in source text, from `start` to `end` (inclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceSpan {
    /// The start of the span.
    pub start: SourceLocation,
    /// The end of the span.
    pub end: SourceLocation,
}

/// A source file with path and content.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceFile {
    /// The file path (may be a display name, not necessarily a real path).
    pub path: String,
    /// The full source text.
    pub source: String,
}

impl SourceFile {
    /// Creates a new `SourceFile` from a path and source content.
    #[must_use]
    pub fn new(path: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            source: source.into(),
        }
    }

    /// Reads a file from disk and creates a `SourceFile`.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the file cannot be read.
    pub fn from_path(path: &str) -> Result<Self, std::io::Error> {
        let source = std::fs::read_to_string(path)?;
        Ok(Self::new(path, source))
    }

    /// Returns the content of the given 1-based line, or `None` if the line is out of range.
    ///
    /// Line numbers start at 1. Returns `None` for line 0 or lines beyond the file.
    #[must_use]
    pub fn line_at(&self, line: u32) -> Option<&str> {
        if line == 0 {
            return None;
        }
        let idx = (line as usize).saturating_sub(1);
        self.source.lines().nth(idx)
    }

    /// Returns a source code snippet showing the span with surrounding context.
    ///
    /// The snippet includes line numbers and an underline (`^^^^`) under the affected range.
    /// `context_lines` controls how many extra lines are shown above and below the span.
    #[must_use]
    pub fn snippet(&self, span: &SourceSpan, context_lines: u32) -> String {
        let mut output = String::new();
        let lines: Vec<&str> = self.source.lines().collect();
        let total_lines = lines.len();

        let first_line = span.start.line.saturating_sub(context_lines).max(1);
        let last_line = (span.end.line + context_lines).min(total_lines as u32);

        for line_num in first_line..=last_line {
            let idx = (line_num as usize).saturating_sub(1);
            if idx >= lines.len() {
                continue;
            }
            let content = lines[idx];

            let _ = writeln!(output, "{line_num:>4} | {content}");

            if line_num == span.start.line {
                let _ = write!(output, "     | ");
                let start_col = (span.start.column.saturating_sub(1)) as usize;
                let end_col = if span.start.line == span.end.line {
                    (span.end.column.saturating_sub(1)) as usize
                } else {
                    content.len()
                };

                let end_col = end_col.max(start_col + 1);
                for i in 0..=content.len() {
                    if i >= start_col && i < end_col {
                        output.push('^');
                    } else {
                        output.push(' ');
                    }
                }
                output.push('\n');
            } else if line_num > span.start.line && line_num <= span.end.line {
                // Multi-line span continuation line
                let _ = writeln!(output, "     | {}", " ".repeat(content.len()));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file_new() {
        let sf = SourceFile::new("test.html", "<div>hello</div>");
        assert_eq!(sf.path, "test.html");
        assert_eq!(sf.source, "<div>hello</div>");
    }

    #[test]
    fn test_line_at() {
        let sf = SourceFile::new("test.html", "line1\nline2\nline3");
        assert_eq!(sf.line_at(1), Some("line1"));
        assert_eq!(sf.line_at(2), Some("line2"));
        assert_eq!(sf.line_at(3), Some("line3"));
        assert_eq!(sf.line_at(0), None);
        assert_eq!(sf.line_at(4), None);
    }

    #[test]
    fn test_line_at_empty_lines() {
        let sf = SourceFile::new("test.html", "a\n\nb");
        assert_eq!(sf.line_at(2), Some(""));
        assert_eq!(sf.line_at(3), Some("b"));
    }

    #[test]
    fn test_snippet_single_line() {
        let sf = SourceFile::new("test.html", "<div class=\"foo\">");
        let loc = SourceLocation {
            line: 1,
            column: 6,
            offset: 5,
        };
        let span = SourceSpan { start: loc, end: loc };
        let snippet = sf.snippet(&span, 0);
        assert!(snippet.contains("1 |"));
        assert!(snippet.contains("^"));
    }

    #[test]
    fn test_snippet_context_lines() {
        let sf = SourceFile::new("test.html", "line1\nline2\nline3\nline4\nline5");
        let loc = SourceLocation {
            line: 3,
            column: 2,
            offset: 7,
        };
        let span = SourceSpan { start: loc, end: loc };
        let snippet = sf.snippet(&span, 1);
        assert!(snippet.contains("2 |"));
        assert!(snippet.contains("3 |"));
        assert!(snippet.contains("4 |"));
        assert!(!snippet.contains("1 |"));
        assert!(!snippet.contains("5 |"));
    }

    #[test]
    fn test_snippet_multi_line() {
        let sf = SourceFile::new("test.html", "aaaa\nbbbb\ncccc");
        let start = SourceLocation {
            line: 1,
            column: 2,
            offset: 1,
        };
        let end = SourceLocation {
            line: 2,
            column: 3,
            offset: 6,
        };
        let span = SourceSpan { start, end };
        let snippet = sf.snippet(&span, 0);
        assert!(snippet.contains("1 |"));
        assert!(snippet.contains("2 |"));
    }

    #[test]
    fn test_source_location_equality() {
        let a = SourceLocation {
            line: 1,
            column: 1,
            offset: 0,
        };
        let b = SourceLocation {
            line: 1,
            column: 1,
            offset: 0,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_source_span_equality() {
        let a = SourceSpan {
            start: SourceLocation {
                line: 1,
                column: 1,
                offset: 0,
            },
            end: SourceLocation {
                line: 1,
                column: 5,
                offset: 4,
            },
        };
        let b = SourceSpan {
            start: SourceLocation {
                line: 1,
                column: 1,
                offset: 0,
            },
            end: SourceLocation {
                line: 1,
                column: 5,
                offset: 4,
            },
        };
        assert_eq!(a, b);
    }
}
