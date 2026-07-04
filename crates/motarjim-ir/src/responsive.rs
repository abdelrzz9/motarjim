use crate::*;

/// Infers responsive breakpoint information from styles.
#[derive(Debug, Clone)]
pub struct ResponsiveInferrer {}

impl ResponsiveInferrer {
    /// Creates a new responsive inferrer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Infers responsive variants for a node.
    #[must_use]
    pub const fn infer(&self, _node: &HtmlNode, _style: &ComputedStyle) -> Vec<ResponsiveVariant> {
        Vec::new()
    }
}

impl Default for ResponsiveInferrer {
    fn default() -> Self {
        Self::new()
    }
}
