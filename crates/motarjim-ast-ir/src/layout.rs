//! Layout strategy types for the Motarjim compiler.
#![allow(missing_docs)]

use smol_str::SmolStr;

/// A responsive breakpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Breakpoint {
    Mobile,
    Tablet,
    Desktop,
    Wide,
    Custom(u32),
}

/// A responsive variant for a node.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct ResponsiveVariant {
    pub breakpoint: Breakpoint,
    pub style_override: Vec<(SmolStr, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_variants() {
        assert!(matches!(Breakpoint::Mobile, Breakpoint::Mobile));
        assert!(matches!(Breakpoint::Custom(768), Breakpoint::Custom(768)));
    }

    #[test]
    fn test_responsive_variant() {
        let rv = ResponsiveVariant {
            breakpoint: Breakpoint::Mobile,
            style_override: vec![(SmolStr::new_inline("padding"), "10px".to_string())],
        };
        assert_eq!(rv.style_override.len(), 1);
    }
}
