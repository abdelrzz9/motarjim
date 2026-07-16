use crate::*;
use motarjim_ast_ir::IrTree;
use motarjim_diag::DiagnosticBag;
use std::collections::HashSet;

/// Validates an [`IrTree`] for structural integrity.
///
/// Checks for:
/// - Duplicate node IDs
/// - Orphan nodes (non-root nodes without a parent)
/// - Invalid parent references
/// - Cycles in the parent-child graph
/// - Invalid heading levels (> 6)
#[derive(Debug, Clone)]
pub struct IrValidator;

impl IrValidator {
    /// Creates a new IR validator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates the given [`IrTree`] and emits diagnostics for any violations.
    pub fn validate(&self, tree: &IrTree, diagnostics: &mut DiagnosticBag) {
        // 1. Check for duplicate node IDs
        let mut seen_ids = HashSet::new();
        for node in &tree.nodes {
            if !seen_ids.insert(node.id) {
                diagnostics.push_warning(
                    motarjim_diag::codes::IR_HEADING_LEVEL_INVALID,
                    format!("Duplicate IR node ID: {}", node.id.0),
                );
            }
        }

        // 2. Build child→parent lookup for orphan/invalid-parent checks
        let node_ids: HashSet<motarjim_ast_html::NodeId> =
            tree.nodes.iter().map(|n| n.id).collect();

        for node in &tree.nodes {
            if node.id == tree.root_id {
                // Root must not have a parent
                if node.parent.is_some() {
                    diagnostics.push_warning(
                        motarjim_diag::codes::IR_HEADING_LEVEL_INVALID,
                        format!("Root node {} has a parent reference", node.id.0),
                    );
                }
                continue;
            }

            // Non-root nodes must have a parent
            if node.parent.is_none() {
                diagnostics.push_warning(
                    motarjim_diag::codes::IR_HEADING_LEVEL_INVALID,
                    format!(
                        "Orphan node {} has no parent (root is {})",
                        node.id.0, tree.root_id.0
                    ),
                );
                continue;
            }

            // Parent reference must point to a valid node
            if let Some(parent_id) = node.parent {
                if !node_ids.contains(&parent_id) {
                    diagnostics.push_warning(
                        motarjim_diag::codes::IR_HEADING_LEVEL_INVALID,
                        format!(
                            "Node {} references non-existent parent {}",
                            node.id.0, parent_id.0
                        ),
                    );
                }
            }
        }

        // 3. Check for cycles via DFS from each node following parent links
        let mut visited = HashSet::new();
        for node in &tree.nodes {
            if visited.contains(&node.id) {
                continue;
            }
            let mut path = HashSet::new();
            let mut current = Some(node.id);
            while let Some(id) = current {
                if !path.insert(id) {
                    diagnostics.push_warning(
                        motarjim_diag::codes::IR_HEADING_LEVEL_INVALID,
                        format!("Cycle detected involving node {}", id.0),
                    );
                    break;
                }
                visited.insert(id);
                // Find the node and follow its parent
                current = tree
                    .nodes
                    .iter()
                    .find(|n| n.id == id)
                    .and_then(|n| n.parent);
            }
        }

        // 4. Validate heading levels
        for node in &tree.nodes {
            if let SemanticIr::Heading { level } = node.semantic {
                if level == 0 || level > 6 {
                    diagnostics.push_warning(
                        motarjim_diag::codes::IR_HEADING_LEVEL_INVALID,
                        format!(
                            "Invalid heading level {level} on node {}; valid range is 1–6",
                            node.id.0
                        ),
                    );
                }
            }
        }
    }
}

impl Default for IrValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_ast_html::NodeId;
    use motarjim_ast_ir::{IrNode, LayoutIr, SemanticIr, TargetIr};
    use smallvec::SmallVec;
    use smol_str::SmolStr;

    fn make_ir_node(
        id: u32,
        semantic: SemanticIr,
        layout: LayoutIr,
        parent: Option<u32>,
        children: &[u32],
    ) -> IrNode {
        IrNode {
            id: NodeId(id),
            semantic,
            layout,
            target: TargetIr::Generic {
                platform: SmolStr::new_inline("default"),
                node: SmolStr::new_inline("Test"),
            },
            computed_style: motarjim_ast_html::ComputedStyle::default(),
            children: children.iter().map(|&id| NodeId(id)).collect(),
            parent: parent.map(NodeId),
            text: None,
            responsive: Vec::new(),
            events: Vec::new(),
            text_direction: None,
        }
    }

    fn make_tree(root_id: u32, nodes: Vec<IrNode>) -> IrTree {
        IrTree {
            nodes,
            root_id: NodeId(root_id),
            target_hints: Vec::new(),
        }
    }

    #[test]
    fn test_valid_tree_no_diagnostics() {
        let tree = make_tree(
            0,
            vec![
                make_ir_node(0, SemanticIr::Root, LayoutIr::FlexColumn, None, &[1]),
                make_ir_node(1, SemanticIr::Button, LayoutIr::Static, Some(0), &[]),
            ],
        );
        let mut diag = DiagnosticBag::new();
        IrValidator::new().validate(&tree, &mut diag);
        assert!(!diag.has_warnings());
    }

    #[test]
    fn test_duplicate_id_detected() {
        let tree = make_tree(
            0,
            vec![
                make_ir_node(0, SemanticIr::Root, LayoutIr::FlexColumn, None, &[1]),
                make_ir_node(
                    0,
                    SemanticIr::Button,
                    LayoutIr::Static,
                    Some(0),
                    &[],
                ),
            ],
        );
        let mut diag = DiagnosticBag::new();
        IrValidator::new().validate(&tree, &mut diag);
        assert!(diag.has_warnings());
    }

    #[test]
    fn test_orphan_detected() {
        let tree = make_tree(
            0,
            vec![
                make_ir_node(0, SemanticIr::Root, LayoutIr::FlexColumn, None, &[]),
                // Node 1 has no parent — orphan
                make_ir_node(1, SemanticIr::Button, LayoutIr::Static, None, &[]),
            ],
        );
        let mut diag = DiagnosticBag::new();
        IrValidator::new().validate(&tree, &mut diag);
        assert!(diag.has_warnings());
    }

    #[test]
    fn test_invalid_parent_detected() {
        let tree = make_tree(
            0,
            vec![make_ir_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                None,
                &[],
            )],
        );
        // Manually create a node with an invalid parent
        let mut tree = tree;
        tree.nodes.push(IrNode {
            id: NodeId(1),
            semantic: SemanticIr::Button,
            layout: LayoutIr::Static,
            target: TargetIr::Generic {
                platform: SmolStr::new_inline("default"),
                node: SmolStr::new_inline("Test"),
            },
            computed_style: motarjim_ast_html::ComputedStyle::default(),
            children: SmallVec::new(),
            parent: Some(NodeId(999)), // Non-existent parent
            text: None,
            responsive: Vec::new(),
            events: Vec::new(),
            text_direction: None,
        });

        let mut diag = DiagnosticBag::new();
        IrValidator::new().validate(&tree, &mut diag);
        assert!(diag.has_warnings());
    }

    #[test]
    fn test_cycle_detected() {
        // Node 0 parent → Node 1, Node 1 parent → Node 0
        let tree = make_tree(
            0,
            vec![
                make_ir_node(0, SemanticIr::Root, LayoutIr::FlexColumn, Some(1), &[]),
                make_ir_node(1, SemanticIr::Button, LayoutIr::Static, Some(0), &[]),
            ],
        );
        let mut diag = DiagnosticBag::new();
        IrValidator::new().validate(&tree, &mut diag);
        assert!(diag.has_warnings());
    }

    #[test]
    fn test_invalid_heading_level() {
        let tree = make_tree(
            0,
            vec![make_ir_node(
                0,
                SemanticIr::Heading { level: 9 },
                LayoutIr::Stack,
                None,
                &[],
            )],
        );
        let mut diag = DiagnosticBag::new();
        IrValidator::new().validate(&tree, &mut diag);
        assert!(diag.has_warnings());
    }

    #[test]
    fn test_valid_heading_levels() {
        let tree = make_tree(
            0,
            vec![
                make_ir_node(
                    0,
                    SemanticIr::Heading { level: 1 },
                    LayoutIr::Stack,
                    None,
                    &[],
                ),
                make_ir_node(
                    1,
                    SemanticIr::Heading { level: 6 },
                    LayoutIr::Stack,
                    Some(0),
                    &[],
                ),
            ],
        );
        let mut diag = DiagnosticBag::new();
        IrValidator::new().validate(&tree, &mut diag);
        assert!(!diag.has_warnings());
    }
}
