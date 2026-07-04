#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery, missing_docs)]

//! Layout strategy types for the Motarjim compiler.

use smol_str::SmolStr;

/// Layout strategies inferred from CSS computed styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LayoutStrategy {
    FlexRow,
    FlexColumn,
    Grid,
    Stack,
    ZStack,
    Scroll,
    Absolute,
    Relative,
    Static,
    Sticky,
    Fixed,
    Flow,
    Inline,
    InlineBlock,
    None,
}

/// Layout constraints for a node.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct LayoutConstraints {
    pub min_width: Option<f64>,
    pub max_width: Option<f64>,
    pub min_height: Option<f64>,
    pub max_height: Option<f64>,
    pub aspect_ratio: Option<f64>,
}

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
    fn test_layout_strategy_variants() {
        let strategies = [
            LayoutStrategy::FlexRow, LayoutStrategy::FlexColumn, LayoutStrategy::Grid,
            LayoutStrategy::Stack, LayoutStrategy::Scroll, LayoutStrategy::None,
        ];
        assert_eq!(strategies.len(), 6);
    }

    #[test]
    fn test_layout_constraints() {
        let lc = LayoutConstraints {
            min_width: None, max_width: None, min_height: None, max_height: None,
            aspect_ratio: None,
        };
        assert!(lc.min_width.is_none() && lc.aspect_ratio.is_none());

        let lc2 = LayoutConstraints {
            min_width: Some(100.0), max_width: Some(1200.0),
            min_height: Some(50.0), max_height: Some(800.0),
            aspect_ratio: Some(16.0 / 9.0),
        };
        assert_eq!(lc2.min_width, Some(100.0));
    }

    #[test]
    fn test_breakpoint_variants() {
        assert!(matches!(Breakpoint::Mobile, Breakpoint::Mobile));
        assert!(matches!(Breakpoint::Custom(768), Breakpoint::Custom(768)));
    }

    #[test]
    fn test_responsive_variant() {
        let rv = ResponsiveVariant {
            breakpoint: Breakpoint::Mobile,
            style_override: vec![
                (SmolStr::new_inline("padding"), "10px".to_string()),
            ],
        };
        assert_eq!(rv.style_override.len(), 1);
    }
}
