use crate::types::*;

/// A specificity value following CSS cascade rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    /// Number of ID selectors.
    pub ids: u32,
    /// Number of class, attribute, and pseudo-class selectors.
    pub classes: u32,
    /// Number of type selectors and pseudo-elements.
    pub types: u32,
}

impl Specificity {
    /// Zero specificity.
    pub const ZERO: Self = Self {
        ids: 0,
        classes: 0,
        types: 0,
    };

    /// Compute specificity for a selector.
    #[must_use]
    pub fn of(selector: &Selector) -> Self {
        match selector {
            Selector::Simple(s) => Self::of_simple(s),
            Selector::Compound(ss) => ss
                .iter()
                .map(Self::of_simple)
                .fold(Self::ZERO, |a, b| a + b),
            Selector::Complex(cx) => {
                let mut total = Self::ZERO;
                for (sel, _) in &cx.sequence {
                    total = total + Self::of(sel);
                }
                total + Self::of(&cx.final_selector)
            }
            Selector::List(l) => l.iter().map(Self::of).max().unwrap_or(Self::ZERO),
        }
    }

    /// Returns the specificity of a single simple selector.
    const fn of_simple(sel: &SimpleSelector) -> Self {
        match sel {
            SimpleSelector::Id(_) => Self {
                ids: 1,
                classes: 0,
                types: 0,
            },
            SimpleSelector::Class(_)
            | SimpleSelector::Attribute(_)
            | SimpleSelector::PseudoClass(_) => Self {
                ids: 0,
                classes: 1,
                types: 0,
            },
            SimpleSelector::Type(_) | SimpleSelector::PseudoElement(_) => Self {
                ids: 0,
                classes: 0,
                types: 1,
            },
            SimpleSelector::Universal => Self::ZERO,
        }
    }
}

impl std::ops::Add for Specificity {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            ids: self.ids + other.ids,
            classes: self.classes + other.classes,
            types: self.types + other.types,
        }
    }
}

/// A matched selector with specificity.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchedSelector {
    /// The matched selector.
    pub selector: Selector,
    /// The specificity of the selector.
    pub specificity: Specificity,
}

impl MatchedSelector {
    /// Creates a new matched selector.
    #[must_use]
    pub fn new(selector: Selector) -> Self {
        let specificity = Specificity::of(&selector);
        Self {
            selector,
            specificity,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use crate::*;

    #[test]
    fn test_specificity_type() {
        let s = parse_selector("div").unwrap();
        assert_eq!(
            Specificity::of(&s),
            Specificity {
                ids: 0,
                classes: 0,
                types: 1
            }
        );
    }

    #[test]
    fn test_specificity_class() {
        let s = parse_selector(".class").unwrap();
        assert_eq!(
            Specificity::of(&s),
            Specificity {
                ids: 0,
                classes: 1,
                types: 0
            }
        );
    }

    #[test]
    fn test_specificity_id() {
        let s = parse_selector("#id").unwrap();
        assert_eq!(
            Specificity::of(&s),
            Specificity {
                ids: 1,
                classes: 0,
                types: 0
            }
        );
    }

    #[test]
    fn test_specificity_compound() {
        let s = parse_selector("div#id.class").unwrap();
        assert_eq!(
            Specificity::of(&s),
            Specificity {
                ids: 1,
                classes: 1,
                types: 1
            }
        );
    }
}
