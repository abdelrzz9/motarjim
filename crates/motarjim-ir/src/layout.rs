use crate::*;
use smallvec::SmallVec;

/// Infers layout strategies from CSS computed styles.
#[derive(Debug, Clone)]
pub struct LayoutInferrer {}

impl LayoutInferrer {
    /// Creates a new layout inferrer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Infers the layout strategy from a node's computed style.
    #[must_use]
    pub fn infer(&self, style: &ComputedStyle) -> LayoutIr {
        if style.display == DisplayType::None {
            return LayoutIr::None;
        }

        match style.position {
            PositionType::Absolute => return LayoutIr::Absolute,
            PositionType::Fixed => return LayoutIr::Fixed,
            PositionType::Sticky => return LayoutIr::Sticky,
            PositionType::Relative | PositionType::Static => {}
        }

        if style.display == DisplayType::Flex {
            return match style.flex_direction {
                Some(FlexDirection::Row) | Some(FlexDirection::RowReverse) => LayoutIr::FlexRow,
                Some(FlexDirection::Column) | Some(FlexDirection::ColumnReverse) => {
                    LayoutIr::FlexColumn
                }
                None => LayoutIr::FlexRow,
            };
        }

        if style.display == DisplayType::Grid {
            return LayoutIr::Grid;
        }

        if let Some(ref overflow) = style.overflow {
            if *overflow == Overflow::Scroll || *overflow == Overflow::Auto {
                return LayoutIr::Scroll;
            }
        }

        match style.display {
            DisplayType::Block | DisplayType::FlowRoot => LayoutIr::Stack,
            DisplayType::Flow => LayoutIr::Flow,
            DisplayType::Inline => LayoutIr::Inline,
            DisplayType::InlineBlock => LayoutIr::InlineBlock,
            DisplayType::Table | DisplayType::TableRow | DisplayType::TableCell => LayoutIr::Table,
            DisplayType::ListItem => LayoutIr::Stack,
            DisplayType::Contents => LayoutIr::None,
            DisplayType::Flex | DisplayType::Grid | DisplayType::None => LayoutIr::None,
        }
    }

    /// Promotes layout variants that require tree context (children positions).
    ///
    /// - **ZStack**: A container with `position: relative` (or static) that has 2+
    ///   children with `position: absolute` is promoted from `Stack` to `ZStack`.
    /// - **LazyList**: A `Scroll` container that is a `<ul>`, `<ol>`, or has 5+
    ///   children is promoted to `LazyList`.
    pub fn promote_tree_aware(
        &self,
        ir_nodes: &mut [IrNode],
        html_nodes: &[motarjim_ast::HtmlNode],
    ) {
        // Build a NodeId → html_node index map for quick lookup
        let html_index: std::collections::HashMap<motarjim_ast_html::NodeId, usize> = html_nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id, i))
            .collect();

        for i in 0..ir_nodes.len() {
            let node_id = ir_nodes[i].id;
            let current_layout = ir_nodes[i].layout.clone();
            let children: SmallVec<[motarjim_ast_html::NodeId; 4]> =
                ir_nodes[i].children.clone();

            // ZStack detection: Stack container with 2+ absolute children
            if current_layout == LayoutIr::Stack && children.len() >= 2 {
                let absolute_count = children
                    .iter()
                    .filter(|child_id| {
                        ir_nodes
                            .iter()
                            .find(|n| &n.id == *child_id)
                            .is_some_and(|n| n.computed_style.position == PositionType::Absolute)
                    })
                    .count();
                if absolute_count >= 2 {
                    ir_nodes[i].layout = LayoutIr::ZStack;
                    continue;
                }
            }

            // LazyList detection: Scroll container that is a list element or has many children
            if current_layout == LayoutIr::Scroll {
                let is_list = html_index
                    .get(&node_id)
                    .and_then(|&idx| html_nodes.get(idx))
                    .and_then(|html_node| html_node.element.as_ref())
                    .is_some_and(|element| {
                        matches!(
                            element.tag_name.as_str(),
                            "ul" | "ol" | "datalist"
                        )
                    });

                if is_list || children.len() >= 5 {
                    ir_nodes[i].layout = LayoutIr::LazyList;
                }
            }
        }
    }
}

impl Default for LayoutInferrer {
    fn default() -> Self {
        Self::new()
    }
}