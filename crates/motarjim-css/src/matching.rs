use crate::*;

/// Checks whether any selector on `rule` matches `element`.
pub(crate) fn rule_matches_element(rule: &StyleRule, element: &Element) -> bool {
    rule.selectors
        .iter()
        .any(|sel| selector_matches_element(sel, element))
}

/// Check whether a single `Selector` matches an element.
fn selector_matches_element(selector: &Selector, element: &Element) -> bool {
    if selector.combinators.is_empty() {
        // Simple compound selector (no combinators): all simple selectors must match.
        return selector
            .simple_selectors
            .iter()
            .all(|s| simple_selector_matches(s, element));
    }

    // For selectors with combinators we do a limited matching — match the last
    // selector group and accept combinators as a simple conjunction.
    // Full combinator-aware matching (walking the DOM tree) is available via the
    // `motarjim-selectors` crate.
    selector
        .simple_selectors
        .iter()
        .all(|s| simple_selector_matches(s, element))
}

/// Check whether a single `SimpleSelector` matches an element.
fn simple_selector_matches(sel: &SimpleSelector, element: &Element) -> bool {
    match sel {
        SimpleSelector::Universal => true,
        SimpleSelector::Type(name) => element.tag_name.as_str() == name.as_str(),
        SimpleSelector::Class(name) => element.has_class(name.as_str()),
        SimpleSelector::Id(name) => element
            .id
            .as_ref()
            .is_some_and(|id| id.as_str() == name.as_str()),
        SimpleSelector::Attribute {
            name,
            operator,
            value,
            case_sensitive: _,
            span: _,
        } => {
            let attr_val = match element.get_attribute(name.as_str()) {
                Some(v) => v,
                None => return false,
            };
            match operator {
                None => true,
                Some(AttributeOperator::Equals) => {
                    value.as_ref().is_some_and(|v| attr_val == v.as_str())
                }
                Some(AttributeOperator::Includes) => value
                    .as_ref()
                    .is_some_and(|v| attr_val.split_whitespace().any(|part| part == v.as_str())),
                Some(AttributeOperator::DashMatch) => value.as_ref().is_some_and(|v| {
                    attr_val == v.as_str() || attr_val.starts_with(&format!("{}-", v.as_str()))
                }),
                Some(AttributeOperator::PrefixMatch) => value
                    .as_ref()
                    .is_some_and(|v| attr_val.starts_with(v.as_str())),
                Some(AttributeOperator::SuffixMatch) => value
                    .as_ref()
                    .is_some_and(|v| attr_val.ends_with(v.as_str())),
                Some(AttributeOperator::SubstringMatch) => value
                    .as_ref()
                    .is_some_and(|v| attr_val.contains(v.as_str())),
            }
        }
        SimpleSelector::PseudoClass(_) | SimpleSelector::PseudoElement(_) => {
            // Pseudo-classes and pseudo-elements are conservatively treated as matching.
            true
        }
    }
}

/// Compute the *maximum* specificity among all selectors in a rule.
pub(crate) fn rule_max_specificity(rule: &StyleRule) -> (u32, u32, u32) {
    rule.selectors
        .iter()
        .map(motarjim_ast::Selector::specificity)
        .max()
        .unwrap_or((0, 0, 0))
}

// ---------------------------------------------------------------------------
// StyleResolver
// ---------------------------------------------------------------------------
