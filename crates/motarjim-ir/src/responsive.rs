use crate::*;
use motarjim_ast_ir::{BreakpointRange, ResponsiveVariant};

/// Infers responsive breakpoint information from styles and media query breakpoints.
///
/// The inferrer receives the breakpoint ranges extracted from `@media` rules
/// by the CSS resolver and determines which breakpoints apply to each node
/// based on its computed style.
#[derive(Debug, Clone)]
pub struct ResponsiveInferrer {}

impl ResponsiveInferrer {
    /// Creates a new responsive inferrer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Infers responsive variants for a node based on the given breakpoint ranges.
    ///
    /// For each breakpoint range that has style differences from the base style,
    /// a [`ResponsiveVariant`] is produced. Currently, this produces variants for
    /// all breakpoint ranges that exist in the stylesheet — future implementations
    /// can refine this by tracking which styles came from which `@media` rules.
    #[must_use]
    pub fn infer(
        &self,
        _node: &HtmlNode,
        _style: &ComputedStyle,
        breakpoints: &[BreakpointRange],
    ) -> Vec<ResponsiveVariant> {
        breakpoints
            .iter()
            .map(|bp| ResponsiveVariant {
                breakpoint: *bp,
                style_override: Vec::new(),
            })
            .collect()
    }
}

impl Default for ResponsiveInferrer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::SmallVec;

    #[test]
    fn test_responsive_empty_breakpoints() {
        let inferrer = ResponsiveInferrer::new();
        let node = HtmlNode {
            id: NodeId(0),
            node_type: NodeType::Element,
            element: Some(Element::new("div")),
            value: None,
            children: SmallVec::new(),
            parent: None,
            depth: 0,
            document_type: None,
        };
        let variants = inferrer.infer(&node, &ComputedStyle::default(), &[]);
        assert!(variants.is_empty());
    }

    #[test]
    fn test_responsive_with_breakpoints() {
        let inferrer = ResponsiveInferrer::new();
        let node = HtmlNode {
            id: NodeId(0),
            node_type: NodeType::Element,
            element: Some(Element::new("div")),
            value: None,
            children: SmallVec::new(),
            parent: None,
            depth: 0,
            document_type: None,
        };
        let breakpoints = vec![
            BreakpointRange::max(768),
            BreakpointRange::min(768),
        ];
        let variants = inferrer.infer(&node, &ComputedStyle::default(), &breakpoints);
        assert_eq!(variants.len(), 2);
        assert!(variants[0].breakpoint.matches(375));
        assert!(variants[1].breakpoint.matches(1024));
    }
}
