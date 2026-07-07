//! CSS selector AST types for the Motarjim compiler.

use smol_str::SmolStr;

use motarjim_span::SourceSpan;

/// A parsed CSS selector, consisting of a sequence of simple selectors separated
/// by combinators.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Selector {
    /// The simple selectors in this selector.
    pub simple_selectors: Vec<SimpleSelector>,
    /// The combinators between simple selector groups.
    pub combinators: Vec<Combinator>,
    /// The source location of this selector.
    pub span: Option<SourceSpan>,
}

impl Selector {
    /// Checks if all simple selectors match the given element (combinators ignored).
    #[must_use]
    pub fn matches(&self, element: &motarjim_ast_html::Element) -> bool {
        self.simple_selectors.iter().all(|s| s.matches(element))
    }

    /// Returns the specificity as a 3-tuple (id, class, type).
    #[must_use]
    pub fn specificity(&self) -> (u32, u32, u32) {
        let mut ids = 0u32;
        let mut classes = 0u32;
        let mut types = 0u32;
        for sel in &self.simple_selectors {
            match sel {
                SimpleSelector::Id(_) => ids += 1,
                SimpleSelector::Class(_)
                | SimpleSelector::Attribute { .. }
                | SimpleSelector::PseudoClass(_) => classes += 1,
                SimpleSelector::Type(_) | SimpleSelector::Universal => types += 1,
                SimpleSelector::PseudoElement(_) => {}
            }
        }
        (ids, classes, types)
    }
}

/// A simple selector component.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum SimpleSelector {
    /// The universal selector `*`.
    Universal,
    /// A type/element selector.
    Type(SmolStr),
    /// A class selector.
    Class(SmolStr),
    /// An ID selector.
    Id(SmolStr),
    /// An attribute selector.
    Attribute {
        /// The attribute name.
        name: SmolStr,
        /// The optional comparison operator.
        operator: Option<AttributeOperator>,
        /// The optional value to compare against.
        value: Option<SmolStr>,
        /// Whether the comparison is case-sensitive.
        case_sensitive: bool,
        /// The source location of this attribute selector.
        span: Option<SourceSpan>,
    },
    /// A pseudo-class selector.
    PseudoClass(PseudoClass),
    /// A pseudo-element selector.
    PseudoElement(PseudoElement),
}

impl SimpleSelector {
    /// Checks if this simple selector matches the given element.
    #[must_use]
    pub fn matches(&self, element: &motarjim_ast_html::Element) -> bool {
        match self {
            Self::Universal => true,
            Self::Type(name) => element.tag_name.as_str() == name.as_str(),
            Self::Class(name) => element.has_class(name.as_str()),
            Self::Id(name) => element
                .id
                .as_ref()
                .is_some_and(|id| id.as_str() == name.as_str()),
            Self::Attribute {
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
                    Some(AttributeOperator::Includes) => value.as_ref().is_some_and(|v| {
                        attr_val.split_whitespace().any(|part| part == v.as_str())
                    }),
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
            Self::PseudoClass(_) | Self::PseudoElement(_) => true,
        }
    }
}

/// Attribute selector operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum AttributeOperator {
    /// Exact match `=`
    Equals,
    /// Whitespace-separated includes `~=`
    Includes,
    /// Hyphen-separated dash match `|=`
    DashMatch,
    /// Prefix match `^=`
    PrefixMatch,
    /// Suffix match `$=`
    SuffixMatch,
    /// Substring match `*=`
    SubstringMatch,
}

/// A combinator between simple selector groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Combinator {
    /// Descendant combinator (space).
    Descendant,
    /// Child combinator (`>`).
    Child,
    /// Adjacent sibling combinator (`+`).
    AdjacentSibling,
    /// General sibling combinator (`~`).
    GeneralSibling,
    /// Column combinator (`||`).
    Column,
}

/// A pseudo-class selector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PseudoClass {
    /// `:hover`
    Hover,
    /// `:active`
    Active,
    /// `:focus`
    Focus,
    /// `:visited`
    Visited,
    /// `:link`
    Link,
    /// `:disabled`
    Disabled,
    /// `:enabled`
    Enabled,
    /// `:checked`
    Checked,
    /// `:first-child`
    FirstChild,
    /// `:last-child`
    LastChild,
    /// `:nth-child(n)`
    NthChild(String),
    /// `:nth-last-child(n)`
    NthLastChild(String),
    /// `:first-of-type`
    FirstOfType,
    /// `:last-of-type`
    LastOfType,
    /// `:not(…)`
    Not(Vec<SimpleSelector>),
    /// `:has(…)`
    Has(Vec<SimpleSelector>),
    /// `:root`
    Root,
    /// `:empty`
    Empty,
    /// `:target`
    Target,
    /// A custom pseudo-class.
    Custom(SmolStr),
}

/// A pseudo-element selector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PseudoElement {
    /// `::before`
    Before,
    /// `::after`
    After,
    /// `::first-line`
    FirstLine,
    /// `::first-letter`
    FirstLetter,
    /// `::placeholder`
    Placeholder,
    /// `::selection`
    Selection,
    /// A custom pseudo-element.
    Custom(SmolStr),
}

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_ast_html::{Attribute, Element};

    #[test]
    fn test_simple_selectors() {
        let el_div = Element::new("div");
        assert!(SimpleSelector::Universal.matches(&el_div));
        assert!(SimpleSelector::Type(SmolStr::new_inline("div")).matches(&el_div));
        assert!(!SimpleSelector::Type(SmolStr::new_inline("span")).matches(&el_div));

        let mut el = Element::new("div");
        el.classes.push(SmolStr::new_inline("container"));
        assert!(SimpleSelector::Class(SmolStr::new_inline("container")).matches(&el));
        assert!(!SimpleSelector::Class(SmolStr::new_inline("foo")).matches(&Element::new("div")));

        el.id = Some(SmolStr::new_inline("main"));
        assert!(SimpleSelector::Id(SmolStr::new_inline("main")).matches(&el));
        assert!(!SimpleSelector::Id(SmolStr::new_inline("other")).matches(&Element::new("div")));
    }

    #[test]
    fn test_attribute_selectors() {
        let mut el = Element::new("a");
        el.attributes
            .push(Attribute::new("href", "https://example.com"));

        let eq = SimpleSelector::Attribute {
            name: SmolStr::new_inline("href"),
            operator: Some(AttributeOperator::Equals),
            value: Some(SmolStr::new_inline("https://example.com")),
            case_sensitive: true,
            span: None,
        };
        assert!(eq.matches(&el));

        let exists = SimpleSelector::Attribute {
            name: SmolStr::new_inline("disabled"),
            operator: None,
            value: None,
            case_sensitive: true,
            span: None,
        };
        assert!(!exists.matches(&el));

        el.attributes.push(Attribute::new("class", "item featured"));
        let includes = SimpleSelector::Attribute {
            name: SmolStr::new_inline("class"),
            operator: Some(AttributeOperator::Includes),
            value: Some(SmolStr::new_inline("featured")),
            case_sensitive: true,
            span: None,
        };
        assert!(includes.matches(&el));
    }

    #[test]
    fn test_selector_matches_and_specificity() {
        let sel = Selector {
            simple_selectors: vec![
                SimpleSelector::Type(SmolStr::new_inline("div")),
                SimpleSelector::Class(SmolStr::new_inline("container")),
            ],
            combinators: vec![],
            span: None,
        };
        let mut el = Element::new("div");
        el.classes.push(SmolStr::new_inline("container"));
        assert!(sel.matches(&el));
        assert!(!sel.matches(&Element::new("span")));

        assert_eq!(
            Selector {
                simple_selectors: vec![SimpleSelector::Id(SmolStr::new_inline("main"))],
                combinators: vec![],
                span: None,
            }
            .specificity(),
            (1, 0, 0)
        );
        assert_eq!(
            Selector {
                simple_selectors: vec![SimpleSelector::Type(SmolStr::new_inline("div"))],
                combinators: vec![],
                span: None,
            }
            .specificity(),
            (0, 0, 1)
        );
    }

    #[test]
    fn test_pseudo_class_variants() {
        assert!(matches!(PseudoClass::Hover, PseudoClass::Hover));
        assert!(matches!(
            PseudoClass::NthChild("2n+1".to_string()),
            PseudoClass::NthChild(_)
        ));
        assert!(matches!(
            PseudoClass::Not(vec![SimpleSelector::Type(SmolStr::new_inline("span"))]),
            PseudoClass::Not(_)
        ));
    }

    #[test]
    fn test_pseudo_element_variants() {
        assert!(matches!(PseudoElement::Before, PseudoElement::Before));
        assert!(matches!(
            PseudoElement::Custom(SmolStr::new_inline("backdrop")),
            PseudoElement::Custom(_)
        ));
    }
}
