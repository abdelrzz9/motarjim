//! All registered diagnostic codes for the Motarjim compiler.
//!
//! Each code is a `DiagnosticCode` constant grouped by category:
//!
//! | Range   | Category   | Prefix |
//! |---------|------------|--------|
//! | 1-99    | Parser     | E0001  |
//! | 100-199 | CSS        | E0100  |
//! | 200-299 | Semantic   | E0200  |
//! | 300-399 | A11y       | E0300  |
//! | 400-499 | IR         | E0400  |
//! | 500-599 | Generator  | E0500  |
//! | 600-699 | Config     | E0600  |

use crate::DiagnosticCode;

// Parser codes (E0001-E0099)

/// Unexpected token during parsing.
pub const PARSER_UNEXPECTED_TOKEN: DiagnosticCode = DiagnosticCode::new(1, "Unexpected token");

/// An unclosed tag was found.
pub const PARSER_UNCLOSED_TAG: DiagnosticCode = DiagnosticCode::new(2, "Unclosed tag");

/// An unknown or unrecognized tag.
pub const PARSER_UNKNOWN_TAG: DiagnosticCode = DiagnosticCode::new(3, "Unknown tag");

/// An invalid attribute was encountered.
pub const PARSER_INVALID_ATTRIBUTE: DiagnosticCode = DiagnosticCode::new(4, "Invalid attribute");

/// A malformed doctype declaration.
pub const PARSER_MALFORMED_DOCTYPE: DiagnosticCode = DiagnosticCode::new(5, "Malformed doctype");

// CSS codes (E0100-E0199)

/// A general CSS parse error.
pub const CSS_PARSE_ERROR: DiagnosticCode = DiagnosticCode::new(100, "CSS parse error");

/// An unknown or unsupported CSS property.
pub const CSS_UNKNOWN_PROPERTY: DiagnosticCode = DiagnosticCode::new(101, "Unknown CSS property");

/// An invalid CSS value for a property.
pub const CSS_INVALID_VALUE: DiagnosticCode = DiagnosticCode::new(102, "Invalid CSS value");

/// An unsupported CSS selector.
pub const CSS_UNSUPPORTED_SELECTOR: DiagnosticCode =
    DiagnosticCode::new(103, "Unsupported selector");

// Semantic codes (E0200-E0299)

/// An ambiguous semantic role was detected.
pub const SEMANTIC_AMBIGUOUS: DiagnosticCode = DiagnosticCode::new(200, "Ambiguous semantic role");

// Accessibility codes (E0300-E0399)

/// An image is missing alt text.
pub const A11Y_MISSING_ALT: DiagnosticCode = DiagnosticCode::new(300, "Missing alt text");

/// A form element is missing a label.
pub const A11Y_MISSING_LABEL: DiagnosticCode = DiagnosticCode::new(301, "Missing form label");

/// Text and background have low color contrast.
pub const A11Y_LOW_CONTRAST: DiagnosticCode = DiagnosticCode::new(302, "Low color contrast");

// IR codes (E0400-E0499)

/// An unsupported layout strategy was encountered.
pub const IR_UNSUPPORTED_LAYOUT: DiagnosticCode = DiagnosticCode::new(400, "Unsupported layout");

// Generator codes (E0500-E0599)

/// An unsupported feature for the target platform.
pub const GEN_UNSUPPORTED_FEATURE: DiagnosticCode =
    DiagnosticCode::new(500, "Unsupported feature for target");

// Config codes (E0600-E0699)

/// A configuration file could not be found.
pub const CONFIG_FILE_NOT_FOUND: DiagnosticCode =
    DiagnosticCode::new(600, "Configuration file not found");

/// A configuration file could not be parsed.
pub const CONFIG_PARSE_ERROR: DiagnosticCode =
    DiagnosticCode::new(601, "Configuration parse error");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_codes() {
        assert_eq!(PARSER_UNEXPECTED_TOKEN.number, 1);
        assert_eq!(PARSER_UNCLOSED_TAG.number, 2);
        assert_eq!(PARSER_UNKNOWN_TAG.number, 3);
        assert_eq!(PARSER_INVALID_ATTRIBUTE.number, 4);
        assert_eq!(PARSER_MALFORMED_DOCTYPE.number, 5);
    }

    #[test]
    fn test_css_codes() {
        assert_eq!(CSS_PARSE_ERROR.number, 100);
        assert_eq!(CSS_UNKNOWN_PROPERTY.number, 101);
        assert_eq!(CSS_INVALID_VALUE.number, 102);
        assert_eq!(CSS_UNSUPPORTED_SELECTOR.number, 103);
    }

    #[test]
    fn test_semantic_codes() {
        assert_eq!(SEMANTIC_AMBIGUOUS.number, 200);
    }

    #[test]
    fn test_a11y_codes() {
        assert_eq!(A11Y_MISSING_ALT.number, 300);
        assert_eq!(A11Y_MISSING_LABEL.number, 301);
        assert_eq!(A11Y_LOW_CONTRAST.number, 302);
    }

    #[test]
    fn test_ir_codes() {
        assert_eq!(IR_UNSUPPORTED_LAYOUT.number, 400);
    }

    #[test]
    fn test_generator_codes() {
        assert_eq!(GEN_UNSUPPORTED_FEATURE.number, 500);
    }

    #[test]
    fn test_config_codes() {
        assert_eq!(CONFIG_FILE_NOT_FOUND.number, 600);
        assert_eq!(CONFIG_PARSE_ERROR.number, 601);
    }

    #[test]
    fn test_code_messages() {
        assert_eq!(PARSER_UNEXPECTED_TOKEN.message, "Unexpected token");
        assert_eq!(A11Y_MISSING_ALT.message, "Missing alt text");
        assert_eq!(CONFIG_PARSE_ERROR.message, "Configuration parse error");
    }

    #[test]
    fn test_code_ranges() {
        assert!((1..=99).contains(&PARSER_UNEXPECTED_TOKEN.number));
        assert!((1..=99).contains(&PARSER_MALFORMED_DOCTYPE.number));
        assert!((100..=199).contains(&CSS_PARSE_ERROR.number));
        assert!((100..=199).contains(&CSS_UNSUPPORTED_SELECTOR.number));
        assert!((200..=299).contains(&SEMANTIC_AMBIGUOUS.number));
        assert!((300..=399).contains(&A11Y_MISSING_ALT.number));
        assert!((300..=399).contains(&A11Y_LOW_CONTRAST.number));
        assert!((400..=499).contains(&IR_UNSUPPORTED_LAYOUT.number));
        assert!((500..=599).contains(&GEN_UNSUPPORTED_FEATURE.number));
        assert!((600..=699).contains(&CONFIG_FILE_NOT_FOUND.number));
        assert!((600..=699).contains(&CONFIG_PARSE_ERROR.number));
    }
}
