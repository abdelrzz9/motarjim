use smallvec::SmallVec;
use smol_str::SmolStr;

use motarjim_ast::Attribute;

/// HTML void elements that cannot have children.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link",
    "meta", "param", "source", "track", "wbr",
];

/// Returns `true` if the given tag name is a void HTML element.
#[must_use]
pub fn is_void_element(tag_name: &str) -> bool {
    VOID_ELEMENTS.contains(&tag_name)
}

/// Extracts the tag name from a raw HTML tag string (e.g., `"<div"` -> `"div"`).
#[must_use]
pub fn extract_tag_name(raw: &str) -> &str {
    let raw = raw.trim();
    if let Some(stripped) = raw.strip_prefix("</") {
        stripped
    } else if let Some(stripped) = raw.strip_prefix('<') {
        stripped
    } else {
        raw
    }
}

/// Finds the offset of `>` in the given tag section, respecting quotes.
/// Returns the byte offset of the `>` character.
pub(crate) fn find_tag_close_offset(section: &str) -> usize {
    let mut in_single = false;
    let mut in_double = false;
    for (i, c) in section.char_indices() {
        match c {
            '"' if !in_single => in_double = !in_double,
            '\'' if !in_double => in_single = !in_single,
            '>' if !in_single && !in_double => return i,
            _ => {}
        }
    }
    0
}

/// Parses attribute name-value pairs from a string.
///
/// The input string should be everything between the tag name and the closing `>`,
/// for example: ` class="foo" id="bar"`.
#[must_use]
pub fn parse_attributes_from_str(input: &str) -> SmallVec<[Attribute; 8]> {
    let mut attrs: SmallVec<[Attribute; 8]> = SmallVec::new();
    let bytes = input.as_bytes();
    let len = input.len();
    let mut pos = 0;

    while pos < len {
        // Skip whitespace
        while pos < len && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= len {
            break;
        }

        // Parse attribute name
        let name_start = pos;
        while pos < len
            && !bytes[pos].is_ascii_whitespace()
            && bytes[pos] != b'='
            && bytes[pos] != b'>'
        {
            pos += 1;
        }
        let name = &input[name_start..pos];
        if name.is_empty() {
            break;
        }

        // Skip whitespace before '='
        while pos < len && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        let value = if pos < len && bytes[pos] == b'=' {
            pos += 1; // consume '='
            // Skip whitespace after '='
            while pos < len && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }

            // Parse quoted or unquoted value
            if pos < len && (bytes[pos] == b'"' || bytes[pos] == b'\'') {
                let quote = bytes[pos];
                pos += 1; // consume opening quote
                let val_start = pos;
                while pos < len && bytes[pos] != quote {
                    pos += 1;
                }
                let val = &input[val_start..pos];
                if pos < len {
                    pos += 1; // consume closing quote
                }
                val.to_string()
            } else {
                let val_start = pos;
                while pos < len
                    && !bytes[pos].is_ascii_whitespace()
                    && bytes[pos] != b'>'
                {
                    pos += 1;
                }
                input[val_start..pos].to_string()
            }
        } else {
            // Boolean attribute (no value)
            String::new()
        };

        attrs.push(Attribute::new(SmolStr::from(name), SmolStr::from(value)));
    }

    attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_attributes_from_str_fn() {
        let attrs = parse_attributes_from_str(" class=\"foo\" id=\"bar\"");
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].name.as_str(), "class");
        assert_eq!(attrs[0].value.as_str(), "foo");
        assert_eq!(attrs[1].name.as_str(), "id");
        assert_eq!(attrs[1].value.as_str(), "bar");
    }

    #[test]
    fn test_parse_attributes_single_quotes_fn() {
        let attrs = parse_attributes_from_str(" type='text' ");
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].name.as_str(), "type");
        assert_eq!(attrs[0].value.as_str(), "text");
    }

    #[test]
    fn test_parse_attributes_no_value_fn() {
        let attrs = parse_attributes_from_str(" disabled checked");
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].name.as_str(), "disabled");
        assert_eq!(attrs[0].value.as_str(), "");
        assert_eq!(attrs[1].name.as_str(), "checked");
        assert_eq!(attrs[1].value.as_str(), "");
    }

    #[test]
    fn test_extract_tag_name_fn() {
        assert_eq!(extract_tag_name("<div"), "div");
        assert_eq!(extract_tag_name("</div"), "div");
        assert_eq!(extract_tag_name("br"), "br");
    }

    #[test]
    fn test_is_void_element_fn() {
        assert!(is_void_element("br"));
        assert!(is_void_element("img"));
        assert!(is_void_element("input"));
        assert!(is_void_element("hr"));
        assert!(!is_void_element("div"));
        assert!(!is_void_element("span"));
    }
}
