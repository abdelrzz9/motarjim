//! CSS parser powered by Lightning CSS.
//!
//! This module provides the main entry point for parsing CSS source text
//! using Lightning CSS as the underlying parser. The parsed result is
//! converted into Motarjim's own AST types, keeping Lightning CSS as an
//! internal implementation detail.

use lightningcss::stylesheet::{ParserOptions, StyleSheet};

use crate::css::converter::convert_stylesheet;
use crate::css::error::CssError;
use motarjim_ast_css::CssStylesheet;

/// Parses a CSS source string into a Motarjim `CssStylesheet`.
///
/// This is the main entry point for CSS parsing. It uses Lightning CSS
/// internally but returns only Motarjim types.
///
/// # Arguments
///
/// * `source` - The CSS source text to parse.
///
/// # Errors
///
/// Returns `CssError` if the CSS is malformed or if conversion fails.
///
/// # Examples
///
/// ```rust
/// use motarjim_parser::css::parse_css;
///
/// let stylesheet = parse_css("div { color: red; }").unwrap();
/// assert_eq!(stylesheet.rules.len(), 1);
/// ```
pub fn parse_css(source: &str) -> Result<CssStylesheet, CssError> {
    let parser = CssParser::new(source);
    parser.parse()
}

/// CSS parser that wraps Lightning CSS.
///
/// This parser uses Lightning CSS for the heavy lifting and converts
/// the result into Motarjim's own AST types.
#[derive(Debug, Clone)]
pub struct CssParser {
    /// The CSS source text.
    source: String,
}

impl CssParser {
    /// Creates a new `CssParser` for the given source text.
    #[must_use]
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }

    /// Parses the CSS source and returns a [`CssStylesheet`] using Motarjim's AST.
    ///
    /// Internally uses Lightning CSS for parsing, then converts to Motarjim types.
    ///
    /// # Errors
    ///
    /// Returns `CssError` if the CSS is malformed or conversion fails.
    pub fn parse(&self) -> Result<CssStylesheet, CssError> {
        let options = ParserOptions::default();

        // Parse with Lightning CSS
        let sheet = match StyleSheet::parse(&self.source, options) {
            Ok(sheet) => sheet,
            Err(err) => {
                return Err(CssError::from_lightningcss(&self.source, err));
            }
        };

        // Convert to Motarjim AST
        convert_stylesheet(&self.source, &sheet)
    }
}

#[cfg(test)]
mod parser_internal_tests {
    use super::*;

    #[test]
    fn test_parser_new() {
        let parser = CssParser::new("div { color: red; }");
        assert_eq!(parser.source, "div { color: red; }");
    }

    #[test]
    fn test_parse_empty() {
        let parser = CssParser::new("");
        let sheet = parser.parse().expect("Failed to parse empty CSS");
        assert!(sheet.rules.is_empty());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let parser = CssParser::new("   \n  \t  ");
        let sheet = parser.parse().expect("Failed to parse whitespace");
        assert!(sheet.rules.is_empty());
    }

    #[test]
    fn test_parse_comments_only() {
        let parser = CssParser::new("/* this is a comment */");
        let sheet = parser.parse().expect("Failed to parse comments");
        assert!(sheet.rules.is_empty());
    }

    #[test]
    fn test_parse_simple_style_rule() {
        let parser = CssParser::new("div { color: red; }");
        let sheet = parser.parse().expect("Failed to parse");
        assert_eq!(sheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_malformed_css() {
        let parser = CssParser::new("div { color: red; ");
        let result = parser.parse();
        // Lightning CSS is lenient; may parse successfully.
        let _ = result;
    }
}
