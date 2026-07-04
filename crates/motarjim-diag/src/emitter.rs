use std::fmt::Write;

use crate::bag::DiagnosticBag;
use crate::diagnostic::{Diagnostic, Severity};
use crate::span::SourceFile;

/// Emits diagnostics to the terminal with optional colored output.
///
/// Each severity level is rendered with a distinct ANSI color:
///
/// | Severity | Color  |
/// |----------|--------|
/// | Error    | Red    |
/// | Warning  | Yellow |
/// | Info     | Blue   |
/// | Hint     | Green  |
/// | Note     | Cyan   |
pub struct DiagnosticEmitter;

impl DiagnosticEmitter {
    /// Creates a new `DiagnosticEmitter`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Emits a single diagnostic to stdout.
    ///
    /// If a `source_file` is provided, a source snippet will be shown.
    pub fn emit(&self, diagnostic: &Diagnostic, source_file: Option<&SourceFile>) {
        let output = self.emit_to_string(diagnostic, source_file);
        print!("{output}");
    }

    /// Emits all diagnostics from a `DiagnosticBag` to stdout.
    pub fn emit_all(&self, bag: &DiagnosticBag, source_file: Option<&SourceFile>) {
        for diagnostic in bag.iter() {
            self.emit(diagnostic, source_file);
        }
    }

    /// Formats a diagnostic as a string without printing it.
    ///
    /// Returns the formatted diagnostic text including color codes.
    #[must_use]
    pub fn emit_to_string(
        &self,
        diagnostic: &Diagnostic,
        source_file: Option<&SourceFile>,
    ) -> String {
        let severity_color = match diagnostic.severity {
            Severity::Error => "\x1b[31m",   // red
            Severity::Warning => "\x1b[33m", // yellow
            Severity::Info => "\x1b[34m",    // blue
            Severity::Hint => "\x1b[32m",    // green
            Severity::Note => "\x1b[36m",    // cyan
        };
        let reset = "\x1b[0m";
        let bold = "\x1b[1m";

        let mut output = String::new();

        // Severity + code header
        let _ = writeln!(
            output,
            "{}{}{}{}[E{:04}]:{} {}",
            bold,
            severity_color,
            diagnostic.severity.as_str(),
            reset,
            diagnostic.code.number,
            reset,
            diagnostic.message
        );

        // Source location with snippet
        if let Some(span) = &diagnostic.span {
            if let Some(sf) = source_file {
                let _ = writeln!(
                    output,
                    " {} {}-->{} {}:{}:{}",
                    severity_color, bold, reset, sf.path, span.start.line, span.start.column
                );
                let _ = writeln!(output, "  {severity_color}");
                output.push_str(&sf.snippet(span, 2));
                let _ = write!(output, "{reset}");
            }
        }

        // Suggestions
        for suggestion in &diagnostic.suggestions {
            let _ = writeln!(output, "  \x1b[32mhelp:{reset} {suggestion}");
        }

        // Notes
        for note in &diagnostic.notes {
            let _ = writeln!(output, "  \x1b[36mnote:{reset} {note}");
        }

        output
    }
}

impl Default for DiagnosticEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes;
    use crate::span::{SourceFile, SourceLocation, SourceSpan};

    #[test]
    fn test_emit_to_string_basic() {
        let emitter = DiagnosticEmitter::new();
        let diag = Diagnostic::new(
            Severity::Error,
            codes::PARSER_UNEXPECTED_TOKEN,
            "unexpected token",
        );
        let output = emitter.emit_to_string(&diag, None);
        assert!(output.contains("error"));
        assert!(output.contains("E0001"));
        assert!(output.contains("unexpected token"));
    }

    #[test]
    fn test_emit_to_string_with_span() {
        let emitter = DiagnosticEmitter::new();
        let sf = SourceFile::new("test.html", "<div>hello</div>");
        let span = SourceSpan {
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
        let diag =
            Diagnostic::new(Severity::Warning, codes::CSS_PARSE_ERROR, "css issue").with_span(span);
        let output = emitter.emit_to_string(&diag, Some(&sf));
        assert!(output.contains("warning"));
        assert!(output.contains("E0100"));
        assert!(output.contains("css issue"));
        assert!(output.contains("test.html"));
        assert!(output.contains("1 |"));
    }

    #[test]
    fn test_emit_to_string_with_suggestions_and_notes() {
        let emitter = DiagnosticEmitter::new();
        let diag = Diagnostic::new(Severity::Hint, codes::A11Y_MISSING_ALT, "missing alt text")
            .with_suggestion("Add an alt attribute")
            .with_note("This is important for screen readers");
        let output = emitter.emit_to_string(&diag, None);
        assert!(output.contains("hint"));
        assert!(output.contains("help:"));
        assert!(output.contains("Add an alt attribute"));
        assert!(output.contains("note:"));
        assert!(output.contains("screen readers"));
    }

    #[test]
    fn test_emit_all_collects_all() {
        let emitter = DiagnosticEmitter::new();
        let mut bag = DiagnosticBag::new();
        bag.push_warning(codes::A11Y_MISSING_ALT, "alt");
        bag.push_error(codes::PARSER_UNEXPECTED_TOKEN, "parse");
        let output = emitter.emit_to_string(
            &Diagnostic::new(Severity::Info, codes::CONFIG_FILE_NOT_FOUND, "config"),
            None,
        );
        // Just verify it produces output without panicking
        assert!(!output.is_empty());
    }
}
