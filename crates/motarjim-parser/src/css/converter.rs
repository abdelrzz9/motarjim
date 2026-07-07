//! AST converter from Lightning CSS to Motarjim.
//!
//! This module converts Lightning CSS AST types into Motarjim's own AST types.
//! No Lightning CSS types escape this module.
//!
//! The conversion strategy is to use Lightning CSS's own CSS serialization
//! and re-parse the output into Motarjim types. This creates a clean
//! decoupling layer between the two AST representations.

use smallvec::SmallVec;
use smol_str::SmolStr;

use motarjim_ast_css::{
    AtRule, AttributeOperator, CharsetRule, Combinator, CssRule, CssStylesheet, Declaration,
    FontFaceRule, ImportRule, Keyframe, KeyframesRule, MediaCondition, MediaQuery, MediaRule,
    NamespaceRule, PageRule, PseudoClass, PseudoElement, Selector, SimpleSelector, StyleRule,
    SupportsRule,
};
use motarjim_span::{SourceLocation, SourceSpan};
use crate::css::error::CssError;

use lightningcss::rules::Location;
use lightningcss::traits::ToCss;

/// Converts a Lightning CSS `Location` (0-based) into a Motarjim `SourceSpan`.
fn to_span(loc: &Location) -> Option<SourceSpan> {
    // Lightning CSS uses 0-based line/column; Motarjim uses 1-based.
    let start = SourceLocation::new(loc.line + 1, loc.column + 1, 0);
    Some(SourceSpan::from_location(start))
}

/// Converts a Lightning CSS `StyleSheet` into a Motarjim `CssStylesheet`.
///
/// # Errors
///
/// Returns `CssError` if the conversion encounters issues.
pub fn convert_stylesheet(
    source: &str,
    sheet: &lightningcss::stylesheet::StyleSheet<'_, '_>,
) -> Result<CssStylesheet, CssError> {
    let mut rules = Vec::new();
    let mut diagnostics = Vec::new();

    for lc_rule in &sheet.rules.0 {
        match convert_rule(source, lc_rule) {
            Ok(Some(rule)) => rules.push(rule),
            Ok(None) => {}
            Err(err) => {
                diagnostics.extend(err.diagnostics());
            }
        }
    }

    if !diagnostics.is_empty() {
        let mut bag = motarjim_diag::DiagnosticBag::new();
        for diag in diagnostics {
            bag.push(diag);
        }
        return Err(CssError::from_bag(bag));
    }

    Ok(CssStylesheet {
        rules,
        source_path: None,
    })
}

/// Converts a Lightning CSS `CssRule` into a Motarjim `CssRule`.
fn convert_rule(
    source: &str,
    rule: &lightningcss::rules::CssRule<'_>,
) -> Result<Option<CssRule>, CssError> {
    match rule {
        lightningcss::rules::CssRule::Style(style_rule) => {
            let span = to_span(&style_rule.loc);
            convert_style_rule(style_rule, span).map(Some)
        }
        lightningcss::rules::CssRule::Media(media_rule) => {
            let span = to_span(&media_rule.loc);
            convert_media_rule(source, media_rule, span).map(Some)
        }
        lightningcss::rules::CssRule::Import(import_rule) => {
            let span = to_span(&import_rule.loc);
            convert_import_rule(import_rule, span).map(Some)
        }
        lightningcss::rules::CssRule::Keyframes(keyframes_rule) => {
            let span = to_span(&keyframes_rule.loc);
            convert_keyframes_rule(keyframes_rule, span).map(Some)
        }
        lightningcss::rules::CssRule::FontFace(font_face_rule) => {
            let span = to_span(&font_face_rule.loc);
            convert_font_face_rule(font_face_rule, span).map(Some)
        }
        lightningcss::rules::CssRule::Supports(supports_rule) => {
            let span = to_span(&supports_rule.loc);
            convert_supports_rule(source, supports_rule, span).map(Some)
        }
        lightningcss::rules::CssRule::Page(page_rule) => {
            let span = to_span(&page_rule.loc);
            convert_page_rule(page_rule, span).map(Some)
        }
        lightningcss::rules::CssRule::Namespace(namespace_rule) => {
            let span = to_span(&namespace_rule.loc);
            convert_namespace_rule(namespace_rule, span).map(Some)
        }
        lightningcss::rules::CssRule::Nesting(nesting_rule) => {
            let span = to_span(&nesting_rule.loc);
            convert_nesting_rule(nesting_rule, span).map(Some)
        }
        lightningcss::rules::CssRule::Ignored => Ok(None),
        lightningcss::rules::CssRule::Unknown(unknown) => {
            let span = to_span(&unknown.loc);
            convert_unknown_rule(unknown, span).map(Some)
        }
        lightningcss::rules::CssRule::NestedDeclarations(_) => {
            // Nested declarations are an intermediate artifact of CSS nesting
            // that don't map directly to Motarjim's AST. Skip them for now.
            Ok(None)
        }
        other => {
            let rule_str = other
                .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
                .unwrap_or_default();
            let name = variant_name(other);
            Ok(Some(CssRule::Other(AtRule {
                name: SmolStr::from(name),
                prelude: rule_str,
                block: None,
                span: None,
            })))
        }
    }
}

/// Converts a Lightning CSS style rule.
fn convert_style_rule(
    rule: &lightningcss::rules::style::StyleRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    let selectors = convert_selectors(&rule.selectors)?;
    let declarations = convert_declaration_block(&rule.declarations);

    Ok(CssRule::Style(StyleRule {
        selectors,
        declarations,
        span,
    }))
}

/// Converts a Lightning CSS selector list into Motarjim selectors by
/// serializing to CSS and re-parsing.
fn convert_selectors(
    selector_list: &lightningcss::selector::SelectorList<'_>,
) -> Result<Vec<Selector>, CssError> {
    let selector_text = selector_list
        .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
        .unwrap_or_default();

    Ok(parse_selector_list(&selector_text))
}

/// Parses a CSS selector list string into Motarjim selectors.
fn parse_selector_list(text: &str) -> Vec<Selector> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    // Split by comma to get individual selectors
    let mut selectors = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0;

    for (i, ch) in text.char_indices() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let sel_text = text[start..i].trim();
                if !sel_text.is_empty() {
                    let (simple_selectors, combinators) = parse_selector_string(sel_text);
                    if !simple_selectors.is_empty() {
                        selectors.push(Selector {
                            simple_selectors,
                            combinators,
                            span: None,
                        });
                    }
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    // Last selector
    let sel_text = text[start..].trim();
    if !sel_text.is_empty() {
        let (simple_selectors, combinators) = parse_selector_string(sel_text);
        if !simple_selectors.is_empty() {
            selectors.push(Selector {
                simple_selectors,
                combinators,
                span: None,
            });
        }
    }

    selectors
}

/// Parses a single selector string into simple selectors and combinators.
fn parse_selector_string(text: &str) -> (Vec<SimpleSelector>, Vec<Combinator>) {
    let text = text.trim();
    let mut simple_selectors = Vec::new();
    let mut combinators = Vec::new();
    let mut in_bracket: i32 = 0;
    let mut in_paren: i32 = 0;
    let mut buffer = String::new();

    for ch in text.chars() {
        match ch {
            '[' => {
                in_bracket += 1;
                buffer.push(ch);
            }
            ']' => {
                in_bracket = in_bracket.saturating_sub(1);
                buffer.push(ch);
            }
            '(' => {
                in_paren += 1;
                buffer.push(ch);
            }
            ')' => {
                in_paren = in_paren.saturating_sub(1);
                buffer.push(ch);
            }
            ' ' | '\t' | '\n' if in_bracket == 0 && in_paren == 0 => {
                // Flush buffer — this is a descendant combinator
                flush_compound_selectors(&mut buffer, &mut simple_selectors);
                // Skip whitespace and note descendant combinator
            }
            '>' | '+' | '~' if in_bracket == 0 && in_paren == 0 => {
                flush_compound_selectors(&mut buffer, &mut simple_selectors);
                let combinator = match ch {
                    '>' => Combinator::Child,
                    '+' => Combinator::AdjacentSibling,
                    '~' => Combinator::GeneralSibling,
                    _ => unreachable!(),
                };
                combinators.push(combinator);
            }
            _ => {
                buffer.push(ch);
            }
        }
    }

    // Flush remaining buffer
    flush_compound_selectors(&mut buffer, &mut simple_selectors);

    (simple_selectors, combinators)
}

/// Flushes the buffer of a compound selector by splitting it into individual
/// simple selectors (type, class, id, pseudo, attribute).
fn flush_compound_selectors(buffer: &mut String, result: &mut Vec<SimpleSelector>) {
    if buffer.is_empty() {
        return;
    }
    let s = std::mem::take(buffer);
    parse_compound_selector(&s, result);
}

/// Parses a compound selector (e.g. `a.foo:hover`) into a list of simple selectors.
fn parse_compound_selector(text: &str, result: &mut Vec<SimpleSelector>) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    let mut remaining = text;

    // Try to extract a type selector at the start if it exists.
    if extracted_type_selector(&mut remaining, result) {
        // type was extracted; continue with the rest
    }

    // Now parse the rest: classes, ids, pseudo-classes, pseudo-elements, attributes
    let mut i = 0;
    let chars: Vec<char> = remaining.chars().collect();
    while i < chars.len() {
        match chars[i] {
            '#' => {
                // ID selector
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
                    i += 1;
                }
                let id: String = chars[(start + 1)..i].iter().collect();
                result.push(SimpleSelector::Id(SmolStr::from(id)));
            }
            '.' => {
                // Class selector
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
                    i += 1;
                }
                let class: String = chars[(start + 1)..i].iter().collect();
                result.push(SimpleSelector::Class(SmolStr::from(class)));
            }
            '[' => {
                // Attribute selector
                let start = i;
                let mut depth = 1;
                i += 1;
                while i < chars.len() && depth > 0 {
                    if chars[i] == '[' { depth += 1; }
                    if chars[i] == ']' { depth -= 1; }
                    i += 1;
                }
                let attr_str: String = chars[start..i].iter().collect();
                if let Some(sel) = parse_attribute_selector(&attr_str) {
                    result.push(sel);
                }
            }
            ':' => {
                if i + 1 < chars.len() && chars[i + 1] == ':' {
                    // Pseudo-element ::
                    let start = i;
                    i += 2;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
                        i += 1;
                    }
                    let name: String = chars[(start + 2)..i].iter().collect();
                    result.push(SimpleSelector::PseudoElement(parse_pseudo_element(&name)));
                } else {
                    // Pseudo-class :
                    let start = i;
                    i += 1;
                    // Handle functional pseudo-classes like :nth-child(2n+1)
                    while i < chars.len() {
                        if chars[i] == '(' {
                            let mut depth = 1;
                            i += 1;
                            while i < chars.len() && depth > 0 {
                                if chars[i] == '(' { depth += 1; }
                                if chars[i] == ')' { depth -= 1; }
                                i += 1;
                            }
                            break;
                        }
                        if !(chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
                            break;
                        }
                        i += 1;
                    }
                    let name: String = chars[(start + 1)..i].iter().collect();
                    result.push(SimpleSelector::PseudoClass(parse_pseudo_class(&name)));
                }
            }
            _ => i += 1,
        }
    }
}

/// Attempts to extract a type selector from the beginning of a compound selector.
/// Returns true if a type selector was found and pushed, and advances `text` past it.
fn extracted_type_selector(text: &mut &str, result: &mut Vec<SimpleSelector>) -> bool {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    let first = trimmed.chars().next().unwrap();
    // A type selector starts with *, a letter, _, -, or \
    if !(first == '*' || first.is_alphabetic() || first == '_' || first == '-' || first == '\\') {
        return false;
    }

    // Count bytes of the type selector
    let type_len = type_selector_len(trimmed);
    if type_len == 0 {
        return false;
    }

    let type_name: String = trimmed[..type_len].chars().collect();
    *text = trimmed[type_len..].trim_start();
    if type_name == "*" {
        result.push(SimpleSelector::Universal);
    } else {
        result.push(SimpleSelector::Type(SmolStr::from(type_name)));
    }
    true
}

/// Returns the byte length of a type selector starting at the given position.
fn type_selector_len(s: &str) -> usize {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return 0;
    }
    // Handle universal selector *
    if chars[0] == '*' {
        return '*'.len_utf8();
    }
    let mut i = 0;
    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_' || chars[i] == '\\') {
        i += 1;
        if i < chars.len() && chars[i] == '|' {
            i += 1; // skip | for namespace prefix
            break;
        }
    }
    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_' || chars[i] == '\\') {
        i += 1;
    }
    // Convert char index back to byte offset
    let mut byte_len = 0;
    for (idx, ch) in s.char_indices() {
        if idx >= i {
            break;
        }
        byte_len = idx + ch.len_utf8();
        if idx + 1 >= i {
            break;
        }
    }
    byte_len
}

/// Parses a simple selector fragment from a CSS selector string.
fn parse_simple_selector(fragment: &str) -> Option<SimpleSelector> {
    let frag = fragment.trim();
    if frag.is_empty() {
        return None;
    }

    if frag == "*" {
        return Some(SimpleSelector::Universal);
    }

    // Pseudo-elements (::)
    if let Some(rest) = frag.strip_prefix("::") {
        return Some(SimpleSelector::PseudoElement(parse_pseudo_element(rest)));
    }

    // Pseudo-classes (:)
    if let Some(rest) = frag.strip_prefix(':') {
        return Some(SimpleSelector::PseudoClass(parse_pseudo_class(rest)));
    }

    // Attribute selectors
    if frag.starts_with('[') && frag.ends_with(']') {
        return parse_attribute_selector(frag);
    }

    // ID selectors
    if let Some(rest) = frag.strip_prefix('#') {
        return Some(SimpleSelector::Id(SmolStr::from(rest)));
    }

    // Class selectors
    if let Some(rest) = frag.strip_prefix('.') {
        return Some(SimpleSelector::Class(SmolStr::from(rest)));
    }

    // Type/universal selectors
    if frag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '*') {
        if frag == "*" {
            Some(SimpleSelector::Universal)
        } else {
            Some(SimpleSelector::Type(SmolStr::from(frag)))
        }
    } else {
        None
    }
}

/// Parses a pseudo-class name string.
fn parse_pseudo_class(name: &str) -> PseudoClass {
    if let Some(paren_idx) = name.find('(') {
        let base = &name[..paren_idx];
        let args = &name[paren_idx + 1..name.len() - 1];

        match base {
            "nth-child" => return PseudoClass::NthChild(args.to_string()),
            "nth-last-child" => return PseudoClass::NthLastChild(args.to_string()),
            "not" => {
                let inner = args.trim();
                if let Some(s) = parse_simple_selector(inner) {
                    return PseudoClass::Not(vec![s]);
                }
                return PseudoClass::Custom(SmolStr::from(name));
            }
            "has" => {
                let inner = args.trim();
                if let Some(s) = parse_simple_selector(inner) {
                    return PseudoClass::Has(vec![s]);
                }
                return PseudoClass::Custom(SmolStr::from(name));
            }
            _ => {
                return PseudoClass::Custom(SmolStr::from(name));
            }
        }
    }

    match name {
        "hover" => PseudoClass::Hover,
        "active" => PseudoClass::Active,
        "focus" => PseudoClass::Focus,
        "visited" => PseudoClass::Visited,
        "link" => PseudoClass::Link,
        "disabled" => PseudoClass::Disabled,
        "enabled" => PseudoClass::Enabled,
        "checked" => PseudoClass::Checked,
        "first-child" => PseudoClass::FirstChild,
        "last-child" => PseudoClass::LastChild,
        "first-of-type" => PseudoClass::FirstOfType,
        "last-of-type" => PseudoClass::LastOfType,
        "root" => PseudoClass::Root,
        "empty" => PseudoClass::Empty,
        "target" => PseudoClass::Target,
        other => PseudoClass::Custom(SmolStr::from(other)),
    }
}

/// Parses a pseudo-element name string.
fn parse_pseudo_element(name: &str) -> PseudoElement {
    match name {
        "before" => PseudoElement::Before,
        "after" => PseudoElement::After,
        "first-line" => PseudoElement::FirstLine,
        "first-letter" => PseudoElement::FirstLetter,
        "placeholder" => PseudoElement::Placeholder,
        "selection" => PseudoElement::Selection,
        other => PseudoElement::Custom(SmolStr::from(other)),
    }
}

/// Parses an attribute selector.
fn parse_attribute_selector(frag: &str) -> Option<SimpleSelector> {
    let inner = frag.strip_prefix('[')?.strip_suffix(']')?;
    let inner = inner.trim();

    let operators = ["~=", "|=", "^=", "$=", "*=", "="];
    for op in &operators {
        if let Some(idx) = inner.find(op) {
            let name = inner[..idx].trim();
            let value = inner[idx + op.len()..].trim();
            let value = value
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                .unwrap_or(value);

            let operator = match *op {
                "=" => Some(AttributeOperator::Equals),
                "~=" => Some(AttributeOperator::Includes),
                "|=" => Some(AttributeOperator::DashMatch),
                "^=" => Some(AttributeOperator::PrefixMatch),
                "$=" => Some(AttributeOperator::SuffixMatch),
                "*=" => Some(AttributeOperator::SubstringMatch),
                _ => None,
            };

            let case_sensitive = !value.ends_with(" i") && !value.ends_with(" I");

            return Some(SimpleSelector::Attribute {
                name: SmolStr::from(name),
                operator,
                value: Some(SmolStr::from(value)),
                case_sensitive,
                span: None,
            });
        }
    }

    Some(SimpleSelector::Attribute {
        name: SmolStr::from(inner),
        operator: None,
        value: None,
        case_sensitive: true,
        span: None,
    })
}

/// Converts a Lightning CSS `DeclarationBlock` into Motarjim declarations.
fn convert_declaration_block(
    block: &lightningcss::declaration::DeclarationBlock<'_>,
) -> SmallVec<[Declaration; 4]> {
    let mut result = SmallVec::new();

    // Normal declarations
    for prop in &block.declarations {
        let property_name = prop.property_id().name().to_string();
        let value = prop
            .value_to_css_string(lightningcss::stylesheet::PrinterOptions::default())
            .unwrap_or_default();

        result.push(Declaration {
            property: SmolStr::from(property_name),
            value,
            important: false,
            span: None,
        });
    }

    // Important declarations
    for prop in &block.important_declarations {
        let property_name = prop.property_id().name().to_string();
        let value = prop
            .value_to_css_string(lightningcss::stylesheet::PrinterOptions::default())
            .unwrap_or_default();

        result.push(Declaration {
            property: SmolStr::from(property_name),
            value,
            important: true,
            span: None,
        });
    }

    result
}

/// Converts a Lightning CSS media rule.
fn convert_media_rule(
    source: &str,
    rule: &lightningcss::rules::media::MediaRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    let query = convert_media_list(&rule.query);
    let mut rules = Vec::new();

    for child in &rule.rules.0 {
        if let Ok(Some(converted)) = convert_rule(source, child) {
            rules.push(converted);
        }
    }

    Ok(CssRule::Media(MediaRule {
        query,
        rules,
        span,
    }))
}

/// Converts a Lightning CSS `MediaList` into Motarjim's `MediaQuery`.
fn convert_media_list(list: &lightningcss::media_query::MediaList<'_>) -> MediaQuery {
    let mut conditions = Vec::new();

    for query in &list.media_queries {
        // Handle media type
        match &query.media_type {
            lightningcss::media_query::MediaType::All => {
                conditions.push(MediaCondition::All);
            }
            lightningcss::media_query::MediaType::Print => {
                conditions.push(MediaCondition::Print);
            }
            lightningcss::media_query::MediaType::Screen => {
                conditions.push(MediaCondition::Screen);
            }
            _ => {
                conditions.push(MediaCondition::All);
            }
        }

        // Handle qualifier
        if let Some(qualifier) = &query.qualifier {
            match qualifier {
                lightningcss::media_query::Qualifier::Only => {
                    let last = conditions.pop().unwrap_or(MediaCondition::All);
                    conditions.push(MediaCondition::Only(Box::new(last)));
                }
                lightningcss::media_query::Qualifier::Not => {
                    let last = conditions.pop().unwrap_or(MediaCondition::All);
                    conditions.push(MediaCondition::Not(Box::new(last)));
                }
            }
        }

        // Handle condition
        if let Some(condition) = &query.condition {
            let cond = convert_media_condition(condition);
            // If we already have conditions, merge with AND
            if !conditions.is_empty() {
                let mut all_conditions = Vec::new();
                all_conditions.extend(conditions.drain(..));
                all_conditions.push(cond);
                conditions.push(MediaCondition::And(all_conditions));
            } else {
                conditions.push(cond);
            }
        }
    }

    MediaQuery { conditions }
}

/// Converts a Lightning CSS `MediaCondition` into Motarjim's `MediaCondition`.
fn convert_media_condition(
    condition: &lightningcss::media_query::MediaCondition<'_>,
) -> MediaCondition {
    use lightningcss::media_query::{MediaCondition as LcCondition, Operator};

    match condition {
        LcCondition::Not(inner) => {
            MediaCondition::Not(Box::new(convert_media_condition(inner)))
        }
        LcCondition::Operation { operator, conditions } => {
            let converted: Vec<MediaCondition> =
                conditions.iter().map(|c| convert_media_condition(c)).collect();
            match operator {
                Operator::And => MediaCondition::And(converted),
                Operator::Or => MediaCondition::Or(converted),
            }
        }
        LcCondition::Feature(feature) => convert_media_feature(feature),
        LcCondition::Unknown(_) => MediaCondition::All,
    }
}

/// Converts a Lightning CSS `MediaFeature` into Motarjim's `MediaCondition`.
fn convert_media_feature(
    feature: &lightningcss::media_query::MediaFeature<'_>,
) -> MediaCondition {
    use lightningcss::media_query::MediaFeature as LcFeature;

    match feature {
        LcFeature::Boolean { name } => {
            let name_str = name
                .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
                .unwrap_or_default();
            MediaCondition::Feature {
                name: SmolStr::from(name_str),
                value: None,
            }
        }
        LcFeature::Plain { name, value: _ } => {
            let name_str = name
                .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
                .unwrap_or_default();
            let name_lower = name_str.to_ascii_lowercase();
            // Serialize the value to get a string representation
            let value_str = feature
                .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
                .ok()
                .and_then(|s| {
                    // Extract value after :
                    s.split(':').nth(1).map(|v| v.trim().to_string())
                });

            match name_lower.as_str() {
                "min-width" => MediaCondition::MinWidth(value_str.unwrap_or_default()),
                "max-width" => MediaCondition::MaxWidth(value_str.unwrap_or_default()),
                "min-height" => MediaCondition::MinHeight(value_str.unwrap_or_default()),
                "max-height" => MediaCondition::MaxHeight(value_str.unwrap_or_default()),
                "orientation" => MediaCondition::Orientation(value_str.unwrap_or_default()),
                "prefers-color-scheme" => {
                    MediaCondition::PrefersColorScheme(value_str.unwrap_or_default())
                }
                _ => MediaCondition::Feature {
                    name: SmolStr::from(name_lower),
                    value: value_str,
                },
            }
        }
        LcFeature::Range { name, .. } => {
            let name_str = name
                .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
                .unwrap_or_default();
            let name_lower = name_str.to_ascii_lowercase();
            let value_str = feature
                .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
                .ok();

            match name_lower.as_str() {
                "min-width" => {
                    MediaCondition::MinWidth(value_str.unwrap_or_default())
                }
                "max-width" => {
                    MediaCondition::MaxWidth(value_str.unwrap_or_default())
                }
                "min-height" => {
                    MediaCondition::MinHeight(value_str.unwrap_or_default())
                }
                "max-height" => {
                    MediaCondition::MaxHeight(value_str.unwrap_or_default())
                }
                _ => MediaCondition::Feature {
                    name: SmolStr::from(name_lower),
                    value: value_str,
                },
            }
        }
        LcFeature::Interval { .. } => {
            let text = feature
                .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
                .unwrap_or_default();
            MediaCondition::Feature {
                name: SmolStr::new_inline("unknown"),
                value: Some(text),
            }
        }
    }
}

/// Converts a Lightning CSS import rule.
fn convert_import_rule(
    rule: &lightningcss::rules::import::ImportRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    let url = rule.url.to_string();
    let media = if rule.media.media_queries.is_empty() {
        None
    } else {
        Some(convert_media_list(&rule.media))
    };

    Ok(CssRule::Import(ImportRule {
        url: SmolStr::from(url),
        media,
        span,
    }))
}

/// Converts a Lightning CSS keyframes rule.
fn convert_keyframes_rule(
    rule: &lightningcss::rules::keyframes::KeyframesRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    use lightningcss::rules::keyframes::KeyframesName;

    let name = match &rule.name {
        KeyframesName::Ident(ident) => ident.to_string(),
        KeyframesName::Custom(s) => s.to_string(),
    };

    let mut keyframes = Vec::new();

    for kf in &rule.keyframes {
        let mut selectors: SmallVec<[SmolStr; 2]> = SmallVec::new();
        for sel in &kf.selectors {
            let sel_str = sel
                .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
                .unwrap_or_default();
            selectors.push(SmolStr::from(sel_str));
        }

        let declarations = convert_declaration_block(&kf.declarations);

        keyframes.push(Keyframe {
            selectors,
            declarations,
            span: None,
        });
    }

    Ok(CssRule::Keyframes(KeyframesRule {
        name: SmolStr::from(name),
        keyframes,
        span,
    }))
}

/// Converts a Lightning CSS font-face rule.
fn convert_font_face_rule(
    rule: &lightningcss::rules::font_face::FontFaceRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    let mut declarations = SmallVec::new();

    for prop in &rule.properties {
        let css_text = prop
            .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
            .unwrap_or_default();
        if let Some(decl) = parse_font_face_declaration(&css_text) {
            declarations.push(decl);
        }
    }

    Ok(CssRule::FontFace(FontFaceRule {
        declarations,
        span,
    }))
}

/// Parses a font-face property CSS string into a Motarjim Declaration.
fn parse_font_face_declaration(css_text: &str) -> Option<Declaration> {
    let css_text = css_text.trim();
    if css_text.is_empty() {
        return None;
    }

    // The format is "property: value" or "property: value !important"
    if let Some(colon_idx) = css_text.find(':') {
        let property = css_text[..colon_idx].trim();
        let mut value = css_text[colon_idx + 1..].trim().to_string();

        let important = if let Some(imp_idx) = value.rfind("!important") {
            value = value[..imp_idx].trim().to_string();
            true
        } else {
            false
        };

        Some(Declaration {
            property: SmolStr::from(property),
            value,
            important,
            span: None,
        })
    } else {
        None
    }
}

/// Converts a Lightning CSS supports rule.
fn convert_supports_rule(
    source: &str,
    rule: &lightningcss::rules::supports::SupportsRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    let condition = rule
        .condition
        .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
        .unwrap_or_default();

    let mut rules = Vec::new();
    for child in &rule.rules.0 {
        if let Ok(Some(converted)) = convert_rule(source, child) {
            rules.push(converted);
        }
    }

    Ok(CssRule::Supports(SupportsRule {
        condition,
        rules,
        span,
    }))
}

/// Converts a Lightning CSS page rule.
fn convert_page_rule(
    rule: &lightningcss::rules::page::PageRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    // Extract a pseudo-class from the page selectors if any
    let pseudo = rule.selectors.first().and_then(|sel| {
        sel.pseudo_classes.first().map(|pc| {
            SmolStr::from(format!("{:?}", pc).to_ascii_lowercase())
        })
    });
    let declarations = convert_declaration_block(&rule.declarations);

    Ok(CssRule::Page(PageRule {
        pseudo,
        declarations,
        span,
    }))
}

/// Converts a Lightning CSS namespace rule.
fn convert_namespace_rule(
    rule: &lightningcss::rules::namespace::NamespaceRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    let prefix = rule.prefix.as_ref().map(|p| SmolStr::from(p.to_string()));
    let url = SmolStr::from(rule.url.to_string());

    Ok(CssRule::Namespace(NamespaceRule {
        prefix,
        url,
        span,
    }))
}

/// Converts a Lightning CSS nesting rule.
fn convert_nesting_rule(
    rule: &lightningcss::rules::nesting::NestingRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    convert_style_rule(&rule.style, span)
}

/// Converts an unknown at-rule into a generic Motarjim at-rule.
fn convert_unknown_rule(
    rule: &lightningcss::rules::unknown::UnknownAtRule<'_>,
    span: Option<SourceSpan>,
) -> Result<CssRule, CssError> {
    let name = SmolStr::from(rule.name.to_string());

    // Serialize the entire rule, then extract parts from the CSS output.
    let full_css = rule
        .to_css_string(lightningcss::stylesheet::PrinterOptions::default())
        .unwrap_or_default();
    let (prelude, block) = extract_at_rule_parts(&full_css, &name);

    // Check if this is a @charset rule
    if name.as_str() == "charset" {
        let encoding = prelude.trim_matches('"').to_string();
        return Ok(CssRule::Charset(CharsetRule {
            encoding: SmolStr::from(encoding),
            span,
        }));
    }

    Ok(CssRule::Other(AtRule {
        name,
        prelude,
        block,
        span,
    }))
}

/// Extracts the prelude and optional block from a serialized at-rule string.
///
/// Input format: `@name prelude;` or `@name prelude { block }`
fn extract_at_rule_parts(css: &str, name: &str) -> (String, Option<String>) {
    let after_at = css.strip_prefix('@').unwrap_or(css);
    let body = after_at.strip_prefix(name).unwrap_or(after_at).trim();

    if let Some(block_start) = body.find('{') {
        let prelude = body[..block_start].trim().to_string();
        let block = body[block_start..].to_string();
        (prelude, Some(block))
    } else {
        let prelude = body.trim_end_matches(';').trim().to_string();
        (prelude, None)
    }
}

/// Returns a static name for a CSS rule variant.
fn variant_name(
    rule: &lightningcss::rules::CssRule<'_>,
) -> &'static str {
    match rule {
        lightningcss::rules::CssRule::Style(_) => "style",
        lightningcss::rules::CssRule::Media(_) => "media",
        lightningcss::rules::CssRule::Import(_) => "import",
        lightningcss::rules::CssRule::Keyframes(_) => "keyframes",
        lightningcss::rules::CssRule::FontFace(_) => "font-face",
        lightningcss::rules::CssRule::Supports(_) => "supports",
        lightningcss::rules::CssRule::Page(_) => "page",
        lightningcss::rules::CssRule::Namespace(_) => "namespace",
        lightningcss::rules::CssRule::Nesting(_) => "nesting",
        lightningcss::rules::CssRule::CounterStyle(_) => "counter-style",
        lightningcss::rules::CssRule::Viewport(_) => "viewport",
        lightningcss::rules::CssRule::CustomMedia(_) => "custom-media",
        lightningcss::rules::CssRule::LayerStatement(_) => "layer",
        lightningcss::rules::CssRule::LayerBlock(_) => "layer",
        lightningcss::rules::CssRule::Property(_) => "property",
        lightningcss::rules::CssRule::Container(_) => "container",
        lightningcss::rules::CssRule::Scope(_) => "scope",
        lightningcss::rules::CssRule::StartingStyle(_) => "starting-style",
        lightningcss::rules::CssRule::ViewTransition(_) => "view-transition",
        lightningcss::rules::CssRule::MozDocument(_) => "-moz-document",
        lightningcss::rules::CssRule::FontPaletteValues(_) => "font-palette-values",
        lightningcss::rules::CssRule::FontFeatureValues(_) => "font-feature-values",
        lightningcss::rules::CssRule::NestedDeclarations(_) => "nested-declarations",
        lightningcss::rules::CssRule::Ignored => "ignored",
        lightningcss::rules::CssRule::Unknown(_) => "unknown",
        lightningcss::rules::CssRule::Custom(_) => "custom",
    }
}

#[cfg(test)]
mod converter_tests {
    use super::*;

    #[test]
    fn test_parse_simple_selector_type() {
        assert_eq!(
            parse_simple_selector("div"),
            Some(SimpleSelector::Type(SmolStr::new_inline("div")))
        );
    }

    #[test]
    fn test_parse_simple_selector_class() {
        assert_eq!(
            parse_simple_selector(".container"),
            Some(SimpleSelector::Class(SmolStr::new_inline("container")))
        );
    }

    #[test]
    fn test_parse_simple_selector_id() {
        assert_eq!(
            parse_simple_selector("#header"),
            Some(SimpleSelector::Id(SmolStr::new_inline("header")))
        );
    }

    #[test]
    fn test_parse_simple_selector_universal() {
        assert_eq!(parse_simple_selector("*"), Some(SimpleSelector::Universal));
    }

    #[test]
    fn test_pseudo_class() {
        let r = parse_simple_selector(":hover");
        assert!(matches!(r, Some(SimpleSelector::PseudoClass(PseudoClass::Hover))));
    }

    #[test]
    fn test_pseudo_element() {
        let r = parse_simple_selector("::before");
        assert!(matches!(r, Some(SimpleSelector::PseudoElement(PseudoElement::Before))));
    }

    #[test]
    fn test_attribute_equals() {
        let r = parse_attribute_selector("[type=text]");
        assert!(matches!(
            r,
            Some(SimpleSelector::Attribute {
                name,
                operator: Some(AttributeOperator::Equals),
                value: Some(v),
                ..
            }) if name.as_str() == "type" && v.as_str() == "text"
        ));
    }

    #[test]
    fn test_attribute_exists() {
        let r = parse_attribute_selector("[disabled]");
        assert!(matches!(
            r,
            Some(SimpleSelector::Attribute {
                name,
                operator: None,
                value: None,
                ..
            }) if name.as_str() == "disabled"
        ));
    }

    #[test]
    fn test_nth_child() {
        match parse_pseudo_class("nth-child(2n+1)") {
            PseudoClass::NthChild(s) => assert_eq!(s, "2n+1"),
            _ => panic!("Expected NthChild"),
        }
    }

    #[test]
    fn test_not_selector() {
        match parse_pseudo_class("not(.hidden)") {
            PseudoClass::Not(selectors) => {
                assert_eq!(selectors.len(), 1);
                assert!(matches!(selectors[0], SimpleSelector::Class(_)));
            }
            _ => panic!("Expected Not"),
        }
    }

    #[test]
    fn test_parse_selector_list_single() {
        let selectors = parse_selector_list("div");
        assert_eq!(selectors.len(), 1);
    }

    #[test]
    fn test_parse_selector_list_multiple() {
        let selectors = parse_selector_list("div, span, .container");
        assert_eq!(selectors.len(), 3);
    }

    #[test]
    fn test_parse_selector_string_empty() {
        let (sels, combs) = parse_selector_string("");
        assert!(sels.is_empty());
        assert!(combs.is_empty());
    }

    #[test]
    fn test_parse_selector_descendant() {
        let (sels, combs) = parse_selector_string("div span");
        assert_eq!(sels.len(), 2);
    }

    #[test]
    fn test_parse_selector_child() {
        let (sels, combs) = parse_selector_string("div > span");
        assert_eq!(sels.len(), 2);
        assert_eq!(combs.len(), 1);
        assert!(matches!(combs[0], Combinator::Child));
    }

    #[test]
    fn test_parse_font_face_declaration() {
        let d = parse_font_face_declaration("font-family: MyFont").unwrap();
        assert_eq!(d.property.as_str(), "font-family");
        assert_eq!(d.value, "MyFont");
        assert!(!d.important);
    }

    #[test]
    fn test_parse_font_face_declaration_important() {
        let d = parse_font_face_declaration("src: url('font.woff2') !important").unwrap();
        assert_eq!(d.property.as_str(), "src");
        assert!(d.important);
    }
}
