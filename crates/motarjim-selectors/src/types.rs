/// A parsed CSS selector.
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    /// A single simple selector (e.g., `div`, `.class`, `#id`)
    Simple(SimpleSelector),
    /// A compound selector (e.g., `div.class`, `a#id.name`)
    Compound(Vec<SimpleSelector>),
    /// A complex selector with combinators (e.g., `div > p`, `div p`)
    Complex(ComplexSelector),
    /// A list of selectors (e.g., `div, span`)
    List(Vec<Self>),
}

/// Simple selector variants.
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleSelector {
    /// Type selector, e.g., `div`, `span`
    Type(String),
    /// Universal selector `*`
    Universal,
    /// Class selector, e.g., `.container`
    Class(String),
    /// ID selector, e.g., `#main`
    Id(String),
    /// Attribute selector, e.g., `[attr]`, `[attr=value]`
    Attribute(AttributeSelector),
    /// Pseudo-class, e.g., `:hover`, `:nth-child(2)`
    PseudoClass(PseudoClass),
    /// Pseudo-element, e.g., `::before`, `::after`
    PseudoElement(String),
}

/// An attribute selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSelector {
    /// Attribute name.
    pub name: String,
    /// Optional comparison operator and value.
    pub value: Option<AttributeValue>,
}

/// Attribute comparison value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeValue {
    /// The comparison operator.
    pub operator: AttributeOperator,
    /// The value to compare against.
    pub value: String,
}

/// Attribute comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOperator {
    /// `=` exactly equals
    Equals,
    /// `~=` contains word
    Includes,
    /// `|=` starts with followed by hyphen
    DashMatch,
    /// `^=` starts with
    Prefix,
    /// `$=` ends with
    Suffix,
    /// `*=` contains substring
    Substring,
}

/// Pseudo-class variants.
#[derive(Debug, Clone, PartialEq)]
pub enum PseudoClass {
    /// `:hover`
    Hover,
    /// `:focus`
    Focus,
    /// `:active`
    Active,
    /// `:visited`
    Visited,
    /// `:link`
    Link,
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
    /// `:not(selector)`
    Not(Box<Selector>),
    /// `:is(selector)`
    Is(Box<Selector>),
    /// `:where(selector)`
    Where(Box<Selector>),
    /// `:has(selector)`
    Has(Box<Selector>),
    /// `:root`
    Root,
    /// `:empty`
    Empty,
    /// `:disabled`
    Disabled,
    /// `:enabled`
    Enabled,
    /// `:checked`
    Checked,
    /// Other pseudo-class
    Other(String),
}

/// A combinator between two selector parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant combinator (space): `div p`
    Descendant,
    /// Child combinator (`>`): `div > p`
    Child,
    /// Next sibling combinator (`+`): `div + p`
    NextSibling,
    /// Subsequent sibling combinator (`~`): `div ~ p`
    SubsequentSibling,
}

impl Combinator {
    /// Returns the string length of this combinator in source.
    #[must_use]
    pub const fn char_len(&self) -> usize {
        match self {
            Self::Descendant => 1,
            Self::Child => 1,
            Self::NextSibling => 1,
            Self::SubsequentSibling => 1,
        }
    }
}

/// A complex selector with combinators.
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
    /// The sequence of selector-combinator pairs.
    pub sequence: Vec<(Selector, Combinator)>,
    /// The final selector.
    pub final_selector: Box<Selector>,
}
