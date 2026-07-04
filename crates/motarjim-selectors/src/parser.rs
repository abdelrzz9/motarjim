use crate::error::SelectorParseError;
use crate::types::*;

/// Parse a single selector string into a `Selector` tree.
pub fn parse_selector(input: &str) -> Result<Selector, SelectorParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(SelectorParseError::Empty);
    }

    // Check for comma-separated list
    if has_comma_outside_parens(input) {
        let mut selectors = Vec::new();
        for part in split_on_commas(input) {
            let part = part.trim();
            if !part.is_empty() {
                selectors.push(parse_selector(part)?);
            }
        }
        if selectors.is_empty() {
            return Err(SelectorParseError::Empty);
        }
        return Ok(if selectors.len() == 1 {
            selectors.into_iter().next().expect("selectors.len() == 1")
        } else {
            Selector::List(selectors)
        });
    }

    // Parse complex selector with combinators
    parse_complex_or_compound(input)
}

/// Returns `true` if the input contains a comma outside of parentheses or brackets.
fn has_comma_outside_parens(input: &str) -> bool {
    let mut depth = 0i32;
    for c in input.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => return true,
            _ => {}
        }
    }
    false
}

/// Splits a selector string on commas that are outside parentheses or brackets.
fn split_on_commas(input: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, c) in input.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                result.push(&input[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(&input[start..]);
    result
}

/// Parses a complex selector (with combinators) or a compound selector.
fn parse_complex_or_compound(input: &str) -> Result<Selector, SelectorParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(SelectorParseError::Empty);
    }

    // Scan left-to-right, accumulating simple selectors and handling combinators
    let mut seq: Vec<(Selector, Combinator)> = Vec::new();
    let mut current_start = 0;
    let mut i = 0;
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0; // byte position

    while pos < input.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' => {
                // Whitespace: might be a descendant combinator or just spacing
                let ws_start = pos;
                pos += 1;
                i += 1;
                // Skip remaining whitespace
                while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\n') {
                    pos += 1;
                    i += 1;
                }
                if i < chars.len() && !matches!(chars[i], '>' | '+' | '~' | ',' | ')' | ']') {
                    // This whitespace is a descendant combinator
                    let left = &input[current_start..ws_start];
                    if left.trim().is_empty() {
                        return Err(SelectorParseError::InvalidCombinator);
                    }
                    let sel = parse_simple_or_compound(left.trim())?;
                    seq.push((sel, Combinator::Descendant));
                    current_start = pos;
                }
            }
            '>' | '+' | '~' => {
                let left = &input[current_start..pos];
                if left.trim().is_empty() {
                    return Err(SelectorParseError::InvalidCombinator);
                }
                let sel = parse_simple_or_compound(left.trim())?;
                let comb = match c {
                    '>' => Combinator::Child,
                    '+' => Combinator::NextSibling,
                    '~' => Combinator::SubsequentSibling,
                    _ => unreachable!(),
                };
                seq.push((sel, comb));
                pos += 1;
                i += 1;
                // Skip whitespace after combinator
                while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\n') {
                    pos += 1;
                    i += 1;
                }
                current_start = pos;
            }
            _ => {
                pos += c.len_utf8();
                i += 1;
            }
        }
    }

    if seq.is_empty() {
        return parse_simple_or_compound(input);
    }

    let final_sel = parse_simple_or_compound(input[current_start..].trim())?;
    Ok(Selector::Complex(ComplexSelector {
        sequence: seq,
        final_selector: Box::new(final_sel),
    }))
}

/// Parses a simple or compound selector from the input string.
fn parse_simple_or_compound(input: &str) -> Result<Selector, SelectorParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(SelectorParseError::Empty);
    }

    let mut simple_selectors = Vec::new();
    let mut pos = 0;
    let chars: Vec<char> = input.chars().collect();

    while pos < chars.len() {
        let c = chars[pos];
        match c {
            '*' => {
                simple_selectors.push(SimpleSelector::Universal);
                pos += 1;
            }
            '.' => {
                pos += 1;
                let len = ident_len(&chars[pos..]);
                if len == 0 {
                    return Err(SelectorParseError::UnexpectedToken(".".to_string()));
                }
                let name: String = chars[pos..pos + len].iter().collect();
                simple_selectors.push(SimpleSelector::Class(name));
                pos += len;
            }
            '#' => {
                pos += 1;
                let len = ident_len(&chars[pos..]);
                if len == 0 {
                    return Err(SelectorParseError::UnexpectedToken("#".to_string()));
                }
                let name: String = chars[pos..pos + len].iter().collect();
                simple_selectors.push(SimpleSelector::Id(name));
                pos += len;
            }
            ':' => {
                let (pseudo, consumed) = parse_pseudo(&chars[pos..])?;
                simple_selectors.push(pseudo);
                pos += consumed;
            }
            '[' => {
                let end = find_bracket_end(&chars[pos..])?;
                let inner: String = chars[pos + 1..pos + end].iter().collect();
                let attr = parse_attribute_selector(&inner)?;
                simple_selectors.push(SimpleSelector::Attribute(attr));
                pos += end + 1;
            }
            'a'..='z' | 'A'..='Z' | '_' | '-' => {
                let len = ident_len(&chars[pos..]);
                if len == 0 {
                    return Err(SelectorParseError::UnexpectedToken(c.to_string()));
                }
                let name: String = chars[pos..pos + len].iter().collect();
                // Only allow type selector at start or after pseudo/attribute
                if simple_selectors.is_empty()
                    || simple_selectors.iter().all(|s| {
                        matches!(
                            s,
                            SimpleSelector::PseudoClass(_) | SimpleSelector::PseudoElement(_)
                        )
                    })
                {
                    simple_selectors.push(SimpleSelector::Type(name));
                } else {
                    return Err(SelectorParseError::UnexpectedToken(name));
                }
                pos += len;
            }
            _ => {
                return Err(SelectorParseError::UnexpectedToken(c.to_string()));
            }
        }
    }

    if simple_selectors.is_empty() {
        return Err(SelectorParseError::Empty);
    }

    Ok(if simple_selectors.len() == 1 {
        Selector::Simple(
            simple_selectors
                .into_iter()
                .next()
                .expect("simple_selectors.len() == 1"),
        )
    } else {
        Selector::Compound(simple_selectors)
    })
}

/// Returns the length of an identifier starting at the beginning of `chars`.
fn ident_len(chars: &[char]) -> usize {
    let mut len = 0;
    for c in chars {
        if c.is_alphanumeric() || *c == '_' || *c == '-' {
            len += 1;
        } else {
            break;
        }
    }
    len
}

/// Parses a pseudo-class or pseudo-element selector from `chars`.
fn parse_pseudo(chars: &[char]) -> Result<(SimpleSelector, usize), SelectorParseError> {
    if chars.is_empty() || chars[0] != ':' {
        return Err(SelectorParseError::UnexpectedToken(String::new()));
    }

    let mut consumed = 1; // first ':'
    let is_pseudo_element = chars.get(1) == Some(&':');

    if is_pseudo_element {
        consumed += 1; // second ':'
    }

    let name_len = ident_len(&chars[consumed..]);
    if name_len == 0 {
        return Err(SelectorParseError::UnexpectedToken(":".to_string()));
    }

    let name: String = chars[consumed..consumed + name_len].iter().collect();
    consumed += name_len;

    if is_pseudo_element {
        return Ok((SimpleSelector::PseudoElement(name), consumed));
    }

    // Check for function syntax
    if chars.get(consumed) == Some(&'(') {
        consumed += 1; // '('
        let mut depth = 1i32;
        let arg_start = consumed;
        let mut arg_end = consumed;
        while consumed < chars.len() {
            match chars[consumed] {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            consumed += 1;
            if depth > 0 {
                arg_end = consumed;
            }
        }
        if depth != 0 {
            return Err(SelectorParseError::UnmatchedParen);
        }
        let args: String = chars[arg_start..arg_end].iter().collect();
        consumed += 1; // ')'

        let pc = match name.as_str() {
            "not" => PseudoClass::Not(Box::new(parse_selector(args.trim())?)),
            "is" => PseudoClass::Is(Box::new(parse_selector(args.trim())?)),
            "where" => PseudoClass::Where(Box::new(parse_selector(args.trim())?)),
            "has" => PseudoClass::Has(Box::new(parse_selector(args.trim())?)),
            "nth-child" => PseudoClass::NthChild(args.trim().to_string()),
            "nth-last-child" => PseudoClass::NthLastChild(args.trim().to_string()),
            _ => PseudoClass::Other(format!("{name}({args})")),
        };
        return Ok((SimpleSelector::PseudoClass(pc), consumed));
    }

    let pc = match name.as_str() {
        "hover" => PseudoClass::Hover,
        "focus" => PseudoClass::Focus,
        "active" => PseudoClass::Active,
        "visited" => PseudoClass::Visited,
        "link" => PseudoClass::Link,
        "first-child" => PseudoClass::FirstChild,
        "last-child" => PseudoClass::LastChild,
        "first-of-type" => PseudoClass::FirstOfType,
        "last-of-type" => PseudoClass::LastOfType,
        "root" => PseudoClass::Root,
        "empty" => PseudoClass::Empty,
        "disabled" => PseudoClass::Disabled,
        "enabled" => PseudoClass::Enabled,
        "checked" => PseudoClass::Checked,
        _ => PseudoClass::Other(name),
    };
    Ok((SimpleSelector::PseudoClass(pc), consumed))
}

/// Finds the position of the matching closing bracket in `chars`.
fn find_bracket_end(chars: &[char]) -> Result<usize, SelectorParseError> {
    let mut depth = 0i32;
    for (i, c) in chars.iter().enumerate() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            _ => {}
        }
    }
    Err(SelectorParseError::UnmatchedBracket)
}

/// Parses an attribute selector (e.g. `[attr]`, `[attr=value]`).
fn parse_attribute_selector(input: &str) -> Result<AttributeSelector, SelectorParseError> {
    let input = input.trim();
    let (name, rest) =
        if let Some(pos) = input.find(|c: char| c.is_whitespace() || "~^$*|!=".contains(c)) {
            (&input[..pos], input[pos..].trim())
        } else {
            return Ok(AttributeSelector {
                name: input.to_string(),
                value: None,
            });
        };

    if rest.is_empty() {
        return Ok(AttributeSelector {
            name: name.to_string(),
            value: None,
        });
    }

    let (operator, after_op) = if let Some(r) = rest.strip_prefix("~=") {
        (AttributeOperator::Includes, r)
    } else if let Some(r) = rest.strip_prefix("|=") {
        (AttributeOperator::DashMatch, r)
    } else if let Some(r) = rest.strip_prefix("^=") {
        (AttributeOperator::Prefix, r)
    } else if let Some(r) = rest.strip_prefix("$=") {
        (AttributeOperator::Suffix, r)
    } else if let Some(r) = rest.strip_prefix("*=") {
        (AttributeOperator::Substring, r)
    } else if let Some(r) = rest.strip_prefix('=') {
        (AttributeOperator::Equals, r)
    } else {
        return Err(SelectorParseError::InvalidAttributeOperator);
    };

    let value = after_op
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();
    Ok(AttributeSelector {
        name: name.to_string(),
        value: Some(AttributeValue { operator, value }),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use crate::*;

    #[test]
    fn test_type_selector() {
        let s = parse_selector("div").unwrap();
        assert_eq!(s, Selector::Simple(SimpleSelector::Type("div".to_string())));
    }

    #[test]
    fn test_class_selector() {
        let s = parse_selector(".container").unwrap();
        assert_eq!(
            s,
            Selector::Simple(SimpleSelector::Class("container".to_string()))
        );
    }

    #[test]
    fn test_id_selector() {
        let s = parse_selector("#main").unwrap();
        assert_eq!(s, Selector::Simple(SimpleSelector::Id("main".to_string())));
    }

    #[test]
    fn test_universal_selector() {
        let s = parse_selector("*").unwrap();
        assert_eq!(s, Selector::Simple(SimpleSelector::Universal));
    }

    #[test]
    fn test_compound_selector() {
        let s = parse_selector("div.container").unwrap();
        assert!(matches!(s, Selector::Compound(_)));
        if let Selector::Compound(ss) = s {
            assert_eq!(ss.len(), 2);
            assert_eq!(ss[0], SimpleSelector::Type("div".to_string()));
            assert_eq!(ss[1], SimpleSelector::Class("container".to_string()));
        }
    }

    #[test]
    fn test_id_with_class() {
        let s = parse_selector("#main.active").unwrap();
        assert!(matches!(s, Selector::Compound(_)));
    }

    #[test]
    fn test_descendant_combinator() {
        let s = parse_selector("div p").unwrap();
        assert!(matches!(s, Selector::Complex(_)));
    }

    #[test]
    fn test_child_combinator() {
        let s = parse_selector("div > p").unwrap();
        assert!(matches!(s, Selector::Complex(_)));
    }

    #[test]
    fn test_adjacent_sibling() {
        let s = parse_selector("div + p").unwrap();
        assert!(matches!(s, Selector::Complex(_)));
    }

    #[test]
    fn test_general_sibling() {
        let s = parse_selector("div ~ p").unwrap();
        assert!(matches!(s, Selector::Complex(_)));
    }

    #[test]
    fn test_selector_list() {
        let s = parse_selector("div, span").unwrap();
        assert!(matches!(s, Selector::List(_)));
        if let Selector::List(list) = s {
            assert_eq!(list.len(), 2);
        }
    }

    #[test]
    fn test_pseudo_class() {
        let s = parse_selector("div:hover").unwrap();
        assert!(matches!(s, Selector::Compound(_)));
    }

    #[test]
    fn test_pseudo_class_with_args() {
        let s = parse_selector("div:nth-child(2)").unwrap();
        assert!(matches!(s, Selector::Compound(_)));
    }

    #[test]
    fn test_attribute_selector() {
        let s = parse_selector("[disabled]").unwrap();
        assert!(matches!(s, Selector::Simple(_)));
    }

    #[test]
    fn test_attribute_with_value() {
        let s = parse_selector("[type=text]").unwrap();
        assert!(matches!(s, Selector::Simple(_)));
    }

    #[test]
    fn test_pseudo_element() {
        let s = parse_selector("div::before").unwrap();
        assert!(matches!(s, Selector::Compound(_)));
    }

    #[test]
    fn test_empty_selector() {
        assert_eq!(parse_selector(""), Err(SelectorParseError::Empty));
    }

    #[test]
    fn test_not_selector() {
        let s = parse_selector(":not(.hidden)").unwrap();
        assert!(matches!(s, Selector::Simple(_)));
    }

    #[test]
    fn test_display_roundtrip() {
        let input = "div.container#main.active";
        let s = parse_selector(input).unwrap();
        let output = s.to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_complex_selector_chain() {
        let s = parse_selector("div > p > span").unwrap();
        assert!(matches!(s, Selector::Complex(_)));
    }
}
