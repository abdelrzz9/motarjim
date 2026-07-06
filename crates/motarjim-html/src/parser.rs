//! HTML5 parser wrapping Servo's html5ever.
//!
//! This module provides the public parsing API. Internally it uses
//! html5ever's spec-compliant parser to produce an RcDom, then converts
//! it into Motarjim's internal AST via [`crate::converter`].
//!
//! # Parsing flow
//!
//! ```text
//! HTML source
//!     │
//!     ▼
//! html5ever (spec-compliant parser)
//!     │
//!     ▼
//! RcDom (reference-counted DOM tree)
//!     │
//!     ▼
//! converter (recursive walk + conversion)
//!     │
//!     ▼
//! Motarjim AST (tree-based, owned types)
//! ```

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::RcDom;

use crate::ast::{Document, Fragment};
use crate::converter::{rcdom_to_document, rcdom_to_fragment};
use crate::diagnostics::{DiagnosticBag, ParseError, ParseErrorKind};

/// A high-performance HTML5 parser that produces Motarjim AST.
///
/// Wraps Servo's html5ever — a browser-grade, spec-compliant HTML parser —
/// and converts its output into Motarjim's internal representation.
///
/// No html5ever types are exposed in the public API.
///
/// # Example
///
/// ```rust
/// use motarjim_html::HtmlParser;
///
/// let doc = HtmlParser::parse("<p>Hello</p>").expect("parsing failed");
/// assert!(!doc.children.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct HtmlParser;

impl HtmlParser {
    /// Parses an HTML source string into a Motarjim [`Document`].
    ///
    /// The parser follows the HTML5 specification, including error recovery.
    /// Malformed input produces structured errors rather than panicking.
    ///
    /// html5ever always produces a complete document tree. Even for a
    /// fragment like `"<p>Hello</p>"`, the result will contain the
    /// implied `<html>`, `<head>`, and `<body>` wrapper elements per
    /// the HTML5 fragment parsing algorithm.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the input cannot be parsed. Note that
    /// html5ever is highly resilient — most inputs produce a valid tree
    /// alongside parse error diagnostics.
    pub fn parse(source: &str) -> Result<Document, ParseError> {
        let mut errors = DiagnosticBag::new();

        let rcdom = match parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut source.as_bytes())
        {
            Ok(dom) => dom,
            Err(e) => {
                return Err(ParseError::new(
                    ParseErrorKind::Html5ever(e.to_string()),
                    format!("Failed to parse HTML: {e}"),
                    None,
                ));
            }
        };

        for err in rcdom.errors.borrow().iter() {
            errors.push(ParseError::warning(
                ParseErrorKind::Html5ever(err.to_string()),
                format!("HTML5 parse warning: {err}"),
                None,
            ));
        }

        let doc = rcdom_to_document(rcdom, &mut errors);

        if errors.has_errors() {
            let first_error = errors
                .errors
                .into_iter()
                .find(|e| e.is_error())
                .unwrap_or_else(|| {
                    ParseError::new(ParseErrorKind::Syntax, "Unknown parse error", None)
                });
            return Err(first_error);
        }

        Ok(doc)
    }

    /// Parses HTML with full diagnostic collection.
    ///
    /// Unlike [`Self::parse`], this method returns all diagnostics
    /// (warnings, info messages, and errors) alongside the document.
    /// The document is still produced even if non-fatal errors occur.
    ///
    /// # Errors
    ///
    /// Returns the diagnostics even on success, so callers can inspect
    /// warnings and informational messages.
    pub fn parse_with_diagnostics(
        source: &str,
    ) -> (Result<Document, ParseError>, Vec<ParseError>) {
        let mut errors = DiagnosticBag::new();

        let rcdom = match parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut source.as_bytes())
        {
            Ok(dom) => dom,
            Err(e) => {
                let err = ParseError::new(
                    ParseErrorKind::Html5ever(e.to_string()),
                    format!("Failed to parse HTML: {e}"),
                    None,
                );
                return (Err(err), vec![]);
            }
        };

        for err in rcdom.errors.borrow().iter() {
            errors.push(ParseError::warning(
                ParseErrorKind::Html5ever(err.to_string()),
                format!("HTML5 parse warning: {err}"),
                None,
            ));
        }

        let doc = rcdom_to_document(rcdom, &mut errors);
        let all_diags = errors.into_diagnostics();

        if all_diags.iter().any(|e| e.is_error()) {
            let first_error = all_diags
                .iter()
                .find(|e| e.is_error())
                .cloned()
                .unwrap_or_else(|| {
                    ParseError::new(ParseErrorKind::Syntax, "Unknown parse error", None)
                });
            (Err(first_error), all_diags)
        } else {
            (Ok(doc), all_diags)
        }
    }

    /// Parses HTML input into a [`Fragment`] without implied document structure.
    ///
    /// Unlike [`Self::parse`], this does not insert `<html>`, `<head>`,
    /// or `<body>` wrapper elements. This is useful for parsing HTML
    /// fragments or template partials.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if parsing fails.
    pub fn parse_fragment(source: &str) -> Result<Fragment, ParseError> {
        let mut errors = DiagnosticBag::new();

        let rcdom = match parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut source.as_bytes())
        {
            Ok(dom) => dom,
            Err(e) => {
                return Err(ParseError::new(
                    ParseErrorKind::Html5ever(e.to_string()),
                    format!("Failed to parse HTML fragment: {e}"),
                    None,
                ));
            }
        };

        for err in rcdom.errors.borrow().iter() {
            errors.push(ParseError::warning(
                ParseErrorKind::Html5ever(err.to_string()),
                format!("HTML5 parse warning: {err}"),
                None,
            ));
        }

        let frag = rcdom_to_fragment(rcdom, &mut errors);

        if errors.has_errors() {
            let first_error = errors
                .errors
                .into_iter()
                .find(|e| e.is_error())
                .unwrap_or_else(|| {
                    ParseError::new(ParseErrorKind::Syntax, "Unknown parse error", None)
                });
            return Err(first_error);
        }

        Ok(frag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let doc = HtmlParser::parse("").expect("empty string should parse");
        assert!(!doc.children.is_empty());
    }

    #[test]
    fn test_parse_simple() {
        let doc = HtmlParser::parse("<div></div>").expect("simple div should parse");
        assert!(doc.find_tag("div").is_some());
    }

    #[test]
    fn test_parse_with_diagnostics_ok() {
        let (result, diags) = HtmlParser::parse_with_diagnostics("<p>hello</p>");
        assert!(result.is_ok());
        // html5ever may produce warnings even for valid input
        assert!(diags.iter().all(|d| !d.is_error()));
    }

    #[test]
    fn test_parse_fragment() {
        let frag = HtmlParser::parse_fragment("<div>hello</div>")
            .expect("fragment should parse");
        assert!(!frag.is_empty());
    }

    #[test]
    fn test_parse_malformed() {
        let doc = HtmlParser::parse("<div><span>unclosed").expect("should handle malformed");
        assert!(!doc.children.is_empty());
    }

    #[test]
    fn test_parse_unicode() {
        let doc = HtmlParser::parse("<p>Hello 世界 🎉</p>").expect("unicode should parse");
        let p = doc.find_tag("p").expect("should find p");
        let text = p.text_content();
        assert!(text.contains("世界"));
        assert!(text.contains("🎉"));
    }
}
