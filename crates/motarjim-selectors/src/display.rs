use std::fmt;

use crate::types::*;

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simple(s) => write!(f, "{s}"),
            Self::Compound(ss) => {
                for s in ss {
                    write!(f, "{s}")?;
                }
                Ok(())
            }
            Self::Complex(cx) => {
                for (sel, comb) in &cx.sequence {
                    write!(f, "{sel} {comb} ")?;
                }
                write!(f, "{}", cx.final_selector)
            }
            Self::List(l) => {
                for (i, sel) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{sel}")?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for SimpleSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(t) => write!(f, "{t}"),
            Self::Universal => write!(f, "*"),
            Self::Class(c) => write!(f, ".{c}"),
            Self::Id(id) => write!(f, "#{id}"),
            Self::Attribute(a) => write!(f, "{a}"),
            Self::PseudoClass(pc) => write!(f, "{pc}"),
            Self::PseudoElement(pe) => write!(f, "::{pe}"),
        }
    }
}

impl fmt::Display for AttributeSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(val) = &self.value {
            write!(f, "[{}{}\"{}\"]", self.name, val.operator, val.value)
        } else {
            write!(f, "[{}]", self.name)
        }
    }
}

impl fmt::Display for AttributeOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equals => write!(f, "="),
            Self::Includes => write!(f, "~="),
            Self::DashMatch => write!(f, "|="),
            Self::Prefix => write!(f, "^="),
            Self::Suffix => write!(f, "$="),
            Self::Substring => write!(f, "*="),
        }
    }
}

impl fmt::Display for PseudoClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hover => write!(f, ":hover"),
            Self::Focus => write!(f, ":focus"),
            Self::Active => write!(f, ":active"),
            Self::Visited => write!(f, ":visited"),
            Self::Link => write!(f, ":link"),
            Self::FirstChild => write!(f, ":first-child"),
            Self::LastChild => write!(f, ":last-child"),
            Self::NthChild(n) => write!(f, ":nth-child({n})"),
            Self::NthLastChild(n) => write!(f, ":nth-last-child({n})"),
            Self::FirstOfType => write!(f, ":first-of-type"),
            Self::LastOfType => write!(f, ":last-of-type"),
            Self::Not(s) => write!(f, ":not({s})"),
            Self::Is(s) => write!(f, ":is({s})"),
            Self::Where(s) => write!(f, ":where({s})"),
            Self::Has(s) => write!(f, ":has({s})"),
            Self::Root => write!(f, ":root"),
            Self::Empty => write!(f, ":empty"),
            Self::Disabled => write!(f, ":disabled"),
            Self::Enabled => write!(f, ":enabled"),
            Self::Checked => write!(f, ":checked"),
            Self::Other(s) => write!(f, ":{s}"),
        }
    }
}

impl fmt::Display for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Descendant => write!(f, " "),
            Self::Child => write!(f, ">"),
            Self::NextSibling => write!(f, "+"),
            Self::SubsequentSibling => write!(f, "~"),
        }
    }
}
