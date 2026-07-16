//! Layout strategy types for the Motarjim compiler.
#![allow(missing_docs)]

use smol_str::SmolStr;

/// A responsive breakpoint range defined by optional min/max width in pixels.
///
/// This preserves the raw CSS values rather than classifying into lossy
/// categories like Mobile/Tablet/Desktop. Generators or tooling can derive
/// categories if they want.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct BreakpointRange {
    /// Minimum viewport width in pixels (inclusive). `None` means no lower bound.
    pub min_width: Option<u32>,
    /// Maximum viewport width in pixels (inclusive). `None` means no upper bound.
    pub max_width: Option<u32>,
}

impl BreakpointRange {
    /// Creates a new breakpoint range with both bounds.
    #[must_use]
    pub const fn new(min_width: Option<u32>, max_width: Option<u32>) -> Self {
        Self {
            min_width,
            max_width,
        }
    }

    /// Creates a breakpoint range from a max-width condition only.
    #[must_use]
    pub const fn max(px: u32) -> Self {
        Self {
            min_width: None,
            max_width: Some(px),
        }
    }

    /// Creates a breakpoint range from a min-width condition only.
    #[must_use]
    pub const fn min(px: u32) -> Self {
        Self {
            min_width: Some(px),
            max_width: None,
        }
    }

    /// Returns true if the given viewport width falls within this range.
    #[must_use]
    pub fn matches(&self, viewport_width: u32) -> bool {
        if let Some(min) = self.min_width {
            if viewport_width < min {
                return false;
            }
        }
        if let Some(max) = self.max_width {
            if viewport_width > max {
                return false;
            }
        }
        true
    }

    /// Classifies this breakpoint range into a human-readable category.
    #[must_use]
    pub fn classify(&self) -> BreakpointCategory {
        match (self.min_width, self.max_width) {
            (None, Some(max)) if max <= 480 => BreakpointCategory::Mobile,
            (Some(min), Some(max)) if min >= 480 && max <= 768 => BreakpointCategory::Tablet,
            (Some(min), Some(max)) if min >= 768 && max <= 1200 => BreakpointCategory::Desktop,
            (Some(min), None) if min >= 1200 => BreakpointCategory::Wide,
            _ => BreakpointCategory::Custom,
        }
    }
}

/// A human-readable breakpoint category derived from [`BreakpointRange`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum BreakpointCategory {
    Mobile,
    Tablet,
    Desktop,
    Wide,
    Custom,
}

impl std::fmt::Display for BreakpointCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mobile => write!(f, "mobile"),
            Self::Tablet => write!(f, "tablet"),
            Self::Desktop => write!(f, "desktop"),
            Self::Wide => write!(f, "wide"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// A responsive variant for a node, produced by the responsive inferrer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct ResponsiveVariant {
    /// The breakpoint range this variant applies to.
    pub breakpoint: BreakpointRange,
    /// Style property overrides that apply at this breakpoint.
    pub style_override: Vec<(SmolStr, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_range_matches() {
        let bp = BreakpointRange::new(None, Some(768));
        assert!(bp.matches(375));
        assert!(bp.matches(768));
        assert!(!bp.matches(1024));

        let bp2 = BreakpointRange::min(768);
        assert!(bp2.matches(1024));
        assert!(!bp2.matches(375));

        let bp3 = BreakpointRange::new(Some(480), Some(768));
        assert!(bp3.matches(480));
        assert!(bp3.matches(768));
        assert!(!bp3.matches(375));
        assert!(!bp3.matches(1024));
    }

    #[test]
    fn test_breakpoint_range_classify() {
        assert_eq!(
            BreakpointRange::max(480).classify(),
            BreakpointCategory::Mobile
        );
        assert_eq!(
            BreakpointRange::new(Some(480), Some(768)).classify(),
            BreakpointCategory::Tablet
        );
        assert_eq!(
            BreakpointRange::new(Some(768), Some(1200)).classify(),
            BreakpointCategory::Desktop
        );
        assert_eq!(
            BreakpointRange::min(1200).classify(),
            BreakpointCategory::Wide
        );
    }

    #[test]
    fn test_responsive_variant() {
        let rv = ResponsiveVariant {
            breakpoint: BreakpointRange::max(768),
            style_override: vec![(SmolStr::new_inline("padding"), "10px".to_string())],
        };
        assert_eq!(rv.style_override.len(), 1);
        assert!(rv.breakpoint.matches(375));
    }
}
