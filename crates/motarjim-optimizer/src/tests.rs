use super::*;
use crate::helpers::{collapse_whitespace, get_text_content};

use motarjim_ast::ir::LayoutIr;
use motarjim_ast_html::{FlexDirection, JustifyContent};

fn default_context() -> PassContext {
    PassContext::default()
}

/// Helper to build an IrNode for testing.
fn make_node(
    id: u32,
    semantic: SemanticIr,
    layout: LayoutIr,
    children: SmallVec<[NodeId; 4]>,
    parent: Option<NodeId>,
) -> IrNode {
    IrNode {
        id: NodeId(id),
        semantic,
        layout,
        target: TargetIr::Generic {
            platform: smol_str::SmolStr::new_inline("flutter"),
            node: smol_str::SmolStr::new_inline("Container"),
        },
        computed_style: ComputedStyle::default(),
        children,
        parent,
        text: None,
    }
}

/// Helper to build a text IrNode with the given text content.
fn make_text_node(
    id: u32,
    text: &str,
    children: SmallVec<[NodeId; 4]>,
    parent: Option<NodeId>,
) -> IrNode {
    IrNode {
        id: NodeId(id),
        semantic: SemanticIr::Text,
        layout: LayoutIr::Flow,
        target: TargetIr::Flutter {
            widget: smol_str::SmolStr::new_inline("Text"),
            properties: vec![(smol_str::SmolStr::new_inline("data"), text.to_string())],
        },
        computed_style: ComputedStyle::default(),
        children,
        parent,
        text: None,
    }
}

/// Helper to create a simple tree with root + children.
fn simple_tree(nodes: Vec<IrNode>) -> IrTree {
    IrTree {
        nodes,
        root_id: NodeId(0),
        target_hints: Vec::new(),
    }
}

// ─── RemoveEmptyNodes tests ──────────────────────────────────────────

#[test]
fn test_remove_empty_text_node() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_text_node(1, "", SmallVec::new(), Some(NodeId(0))),
    ]);

    let pass = RemoveEmptyNodes::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_removed, 1);

    if let Some(root) = tree.nodes.first() {
        assert!(root.children.is_empty());
    }
}

#[test]
fn test_remove_whitespace_only_text_node() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_text_node(1, "   \n  \t  ", SmallVec::new(), Some(NodeId(0))),
    ]);

    let pass = RemoveEmptyNodes::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_removed, 1);
    if let Some(root) = tree.nodes.first() {
        assert!(root.children.is_empty());
    }
}

#[test]
fn test_remove_empty_container() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_node(
            1,
            SemanticIr::Container,
            LayoutIr::FlexColumn,
            SmallVec::new(),
            Some(NodeId(0)),
        ),
    ]);

    let pass = RemoveEmptyNodes::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_removed, 1);
    if let Some(root) = tree.nodes.first() {
        assert!(root.children.is_empty());
    }
}

#[test]
fn test_remove_display_none_node() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_node(
            1,
            SemanticIr::Button,
            LayoutIr::Static,
            SmallVec::new(),
            Some(NodeId(0)),
        ),
    ]);
    if let Some(btn) = tree.nodes.get_mut(1) {
        btn.computed_style.display = DisplayType::None;
    }

    let pass = RemoveEmptyNodes::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_removed, 1);
    if let Some(root) = tree.nodes.first() {
        assert!(root.children.is_empty());
    }
}

#[test]
fn test_keep_non_empty_text_node() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_text_node(1, "Hello", SmallVec::new(), Some(NodeId(0))),
    ]);

    let pass = RemoveEmptyNodes::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(
        stats.nodes_removed, 0,
        "non-empty text node should not be removed"
    );
    if let Some(text_node) = tree.nodes.get(1) {
        assert_eq!(text_node.semantic, SemanticIr::Text);
    }
}

// ─── MergeAdjacentText tests ─────────────────────────────────────────

#[test]
fn test_merge_two_adjacent_text_nodes() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1), NodeId(2)],
            None,
        ),
        make_text_node(1, "Hello ", SmallVec::new(), Some(NodeId(0))),
        make_text_node(2, "World", SmallVec::new(), Some(NodeId(0))),
    ]);

    let pass = MergeAdjacentText::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert!(stats.nodes_removed >= 1);

    if let Some(root) = tree.nodes.first() {
        assert_eq!(root.children.len(), 1);
        if let Some(remaining) = root
            .children
            .first()
            .and_then(|&id| tree.nodes.get(id.0 as usize))
        {
            assert_eq!(get_text_content(remaining).as_deref(), Some("Hello World"));
        }
    }
}

#[test]
fn test_merge_three_adjacent_text_nodes() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1), NodeId(2), NodeId(3)],
            None,
        ),
        make_text_node(1, "A", SmallVec::new(), Some(NodeId(0))),
        make_text_node(2, "B", SmallVec::new(), Some(NodeId(0))),
        make_text_node(3, "C", SmallVec::new(), Some(NodeId(0))),
    ]);

    let pass = MergeAdjacentText::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert!(stats.nodes_removed >= 1);

    if let Some(root) = tree.nodes.first() {
        assert_eq!(root.children.len(), 1);
        if let Some(remaining) = root
            .children
            .first()
            .and_then(|&id| tree.nodes.get(id.0 as usize))
        {
            assert_eq!(get_text_content(remaining).as_deref(), Some("ABC"));
        }
    }
}

#[test]
fn test_merge_does_not_merge_non_text_siblings() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1), NodeId(2)],
            None,
        ),
        make_text_node(1, "Hello", SmallVec::new(), Some(NodeId(0))),
        make_node(
            2,
            SemanticIr::Button,
            LayoutIr::Static,
            SmallVec::new(),
            Some(NodeId(0)),
        ),
    ]);

    let pass = MergeAdjacentText::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_removed, 0);

    if let Some(root) = tree.nodes.first() {
        assert_eq!(root.children.len(), 2);
    }
}

// ─── CollapseWhitespace tests ────────────────────────────────────────

#[test]
fn test_collapse_whitespace_in_text_node() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_text_node(1, "Hello    World\n  Foo", SmallVec::new(), Some(NodeId(0))),
    ]);

    let pass = CollapseWhitespace::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_modified, 1);

    if let Some(text_node) = tree.nodes.get(1) {
        assert_eq!(
            get_text_content(text_node).as_deref(),
            Some("Hello World Foo")
        );
    }
}

#[test]
fn test_collapse_whitespace_no_change() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_text_node(1, "Hello World", SmallVec::new(), Some(NodeId(0))),
    ]);

    let pass = CollapseWhitespace::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_modified, 0);

    if let Some(text_node) = tree.nodes.get(1) {
        assert_eq!(get_text_content(text_node).as_deref(), Some("Hello World"));
    }
}

#[test]
fn test_collapse_whitespace_trims_edges() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_text_node(1, "   Hello World   ", SmallVec::new(), Some(NodeId(0))),
    ]);

    let pass = CollapseWhitespace::new();
    let ctx = default_context();
    let _ = pass.run(&mut tree, &ctx);

    if let Some(text_node) = tree.nodes.get(1) {
        assert_eq!(get_text_content(text_node).as_deref(), Some("Hello World"));
    }
}

// ─── RemoveUnusedStyles tests ────────────────────────────────────────

#[test]
fn test_remove_flex_styles_from_non_flex() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_node(
            1,
            SemanticIr::Button,
            LayoutIr::Static,
            SmallVec::new(),
            Some(NodeId(0)),
        ),
    ]);
    if let Some(btn) = tree.nodes.get_mut(1) {
        btn.computed_style.flex_direction = Some(FlexDirection::Row);
        btn.computed_style.justify_content = Some(JustifyContent::Center);
    }

    let pass = RemoveUnusedStyles::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_modified, 1);

    if let Some(button) = tree.nodes.get(1) {
        assert_eq!(button.computed_style.flex_direction, None);
        assert_eq!(button.computed_style.justify_content, None);
    }
}

#[test]
fn test_remove_grid_styles_from_non_grid() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_node(
            1,
            SemanticIr::Section,
            LayoutIr::Stack,
            SmallVec::new(),
            Some(NodeId(0)),
        ),
    ]);
    if let Some(node) = tree.nodes.get_mut(1) {
        node.computed_style.grid_template_columns = Some("1fr 1fr".to_string());
    }

    let pass = RemoveUnusedStyles::new();
    let ctx = default_context();
    let _ = pass.run(&mut tree, &ctx);

    if let Some(node) = tree.nodes.get(1) {
        assert_eq!(node.computed_style.grid_template_columns, None);
    }
}

#[test]
fn test_keep_flex_styles_on_flex_container() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_node(
            1,
            SemanticIr::Flex,
            LayoutIr::FlexColumn,
            SmallVec::new(),
            Some(NodeId(0)),
        ),
    ]);
    if let Some(node) = tree.nodes.get_mut(1) {
        node.computed_style.flex_direction = Some(FlexDirection::Row);
        node.computed_style.justify_content = Some(JustifyContent::Center);
    }

    let pass = RemoveUnusedStyles::new();
    let ctx = default_context();
    let _ = pass.run(&mut tree, &ctx);

    if let Some(node) = tree.nodes.get(1) {
        assert_eq!(node.computed_style.flex_direction, Some(FlexDirection::Row));
        assert_eq!(
            node.computed_style.justify_content,
            Some(JustifyContent::Center)
        );
    }
}

// ─── FlattenNestedContainers tests ────────────────────────────────────

#[test]
fn test_flatten_nested_container() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_node(
            1,
            SemanticIr::Container,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(2)],
            Some(NodeId(0)),
        ),
        make_node(
            2,
            SemanticIr::Container,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(3)],
            Some(NodeId(1)),
        ),
        make_text_node(3, "deep", SmallVec::new(), Some(NodeId(2))),
    ]);

    let pass = FlattenNestedContainers::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert!(stats.nodes_removed >= 1);

    if let Some(container) = tree.nodes.get(1) {
        assert!(container.children.contains(&NodeId(3)));
    }
}

#[test]
fn test_flatten_does_not_remove_non_container_child() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_node(
            1,
            SemanticIr::Container,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(2)],
            Some(NodeId(0)),
        ),
        make_text_node(2, "hello", SmallVec::new(), Some(NodeId(1))),
    ]);

    let pass = FlattenNestedContainers::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_removed, 0);
}

// ─── InlineConstantValues tests ──────────────────────────────────────

#[test]
fn test_inline_constant_container() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_node(
            1,
            SemanticIr::Section,
            LayoutIr::Stack,
            smallvec::smallvec![NodeId(2)],
            Some(NodeId(0)),
        ),
        make_text_node(2, "content", SmallVec::new(), Some(NodeId(1))),
    ]);

    let pass = InlineConstantValues::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert!(stats.nodes_removed >= 1);

    if let Some(root) = tree.nodes.first() {
        assert!(root.children.contains(&NodeId(2)));
    }
}

#[test]
fn test_does_not_inline_root() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_text_node(1, "content", SmallVec::new(), Some(NodeId(0))),
    ]);

    let pass = InlineConstantValues::new();
    let ctx = default_context();
    pass.run(&mut tree, &ctx).unwrap();
    let stats = pass.statistics().snapshot();
    assert_eq!(stats.nodes_removed, 0);
}

// ─── PassManager tests ───────────────────────────────────────────────

#[test]
fn test_pass_manager_empty() {
    let manager = PassManager::new();
    assert!(manager.is_empty());
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_pass_manager_register_and_run() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_text_node(1, "", SmallVec::new(), Some(NodeId(0))),
    ]);

    let mut manager = PassManager::new();
    manager.register(Box::new(RemoveEmptyNodes::new()));

    assert_eq!(manager.len(), 1);
    let results = manager.run_all(&mut tree);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].nodes_removed, 1);
}

#[test]
fn test_register_default_passes() {
    let mut manager = PassManager::new();
    register_default_passes(&mut manager);
    assert_eq!(manager.len(), 6);
}

#[test]
fn test_pass_manager_pass_count() {
    let mut manager = PassManager::new();
    register_default_passes(&mut manager);
    assert_eq!(manager.pass_count(), 6);
}

#[test]
fn test_pass_manager_statistics() {
    let mut manager = PassManager::new();
    register_default_passes(&mut manager);
    let stats = manager.statistics();
    assert!(stats.contains_key("RemoveEmptyNodes"));
    assert!(stats.contains_key("CollapseWhitespace"));
    assert!(stats.contains_key("MergeAdjacentText"));
    assert!(stats.contains_key("RemoveUnusedStyles"));
    assert!(stats.contains_key("FlattenNestedContainers"));
    assert!(stats.contains_key("InlineConstantValues"));
}

#[test]
fn test_pass_result_contains_cost() {
    let mut tree = simple_tree(vec![
        make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            smallvec::smallvec![NodeId(1)],
            None,
        ),
        make_text_node(1, "", SmallVec::new(), Some(NodeId(0))),
    ]);

    let mut manager = PassManager::new();
    manager.register(Box::new(RemoveEmptyNodes::new()));

    let results = manager.run_all(&mut tree);
    assert_eq!(results[0].cost, PassCost::ON);
    assert!(results[0].duration_ns > 0);
}

#[test]
fn test_pass_statistics_snapshot() {
    let stats = PassStatistics::new();
    let snap = stats.snapshot();
    assert_eq!(snap.nodes_visited, 0);
    assert_eq!(snap.nodes_modified, 0);
    assert_eq!(snap.nodes_removed, 0);
    assert_eq!(snap.memory_freed, 0);
    assert_eq!(snap.duration_ns, 0);
}

#[test]
fn test_pass_statistics_reset() {
    let stats = PassStatistics::new();
    stats.nodes_visited.fetch_add(42, Ordering::Relaxed);
    stats.reset();
    assert_eq!(stats.nodes_visited.load(Ordering::SeqCst), 0);
}

#[test]
fn test_pass_cost_equality() {
    assert_eq!(PassCost::O1, PassCost::O1);
    assert_ne!(PassCost::O1, PassCost::ON);
}

#[test]
fn test_cancel_token() {
    let token = CancelToken::new();
    assert!(!token.is_cancelled());
    token.cancel();
    assert!(token.is_cancelled());
}

// ─── Integration: multiple passes ────────────────────────────────────

#[test]
fn test_full_pipeline_on_complex_tree() {
    let mut tree = IrTree {
        nodes: vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1), NodeId(2), NodeId(3), NodeId(4)],
                None,
            ),
            make_node(
                1,
                SemanticIr::Container,
                LayoutIr::FlexColumn,
                SmallVec::new(),
                Some(NodeId(0)),
            ),
            make_text_node(2, "Hello    World", SmallVec::new(), Some(NodeId(0))),
            make_text_node(3, "Foo", SmallVec::new(), Some(NodeId(0))),
            make_node(
                4,
                SemanticIr::Section,
                LayoutIr::Stack,
                smallvec::smallvec![NodeId(5)],
                Some(NodeId(0)),
            ),
            make_node(
                5,
                SemanticIr::Container,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(6)],
                Some(NodeId(4)),
            ),
            make_text_node(6, "Bar", SmallVec::new(), Some(NodeId(5))),
        ],
        root_id: NodeId(0),
        target_hints: Vec::new(),
    };

    let mut manager = PassManager::new();
    register_default_passes(&mut manager);
    let results = manager.run_all(&mut tree);

    let total_removed: usize = results.iter().map(|r| r.nodes_removed).sum();
    assert!(
        total_removed >= 3,
        "Expected at least 3 removals, got {total_removed}"
    );

    if let Some(root) = tree.nodes.first() {
        let has_text_child = root.children.iter().any(|&cid| {
            tree.nodes
                .get(cid.0 as usize)
                .is_some_and(|n| matches!(n.semantic, SemanticIr::Text))
        });
        assert!(
            has_text_child,
            "root should have text children after passes"
        );
    }
}

// ─── collapse_whitespace unit tests ──────────────────────────────────

#[test]
fn test_collapse_whitespace_simple() {
    assert_eq!(collapse_whitespace("a   b"), "a b");
}

#[test]
fn test_collapse_whitespace_newlines_and_tabs() {
    assert_eq!(collapse_whitespace("a\n\tb\n\nc"), "a b c");
}

#[test]
fn test_collapse_whitespace_trim() {
    assert_eq!(collapse_whitespace("  hello  "), "hello");
}

#[test]
fn test_collapse_whitespace_empty() {
    assert_eq!(collapse_whitespace(""), "");
}

#[test]
fn test_collapse_whitespace_all_whitespace() {
    assert_eq!(collapse_whitespace("   \n  \t  "), "");
}

#[test]
fn test_collapse_whitespace_no_change_needed() {
    assert_eq!(collapse_whitespace("hello world"), "hello world");
}
