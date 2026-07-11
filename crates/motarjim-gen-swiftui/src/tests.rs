use super::*;
use motarjim_ast::ir::{LayoutIr, SemanticIr, TargetIr};
use motarjim_ast::NodeId;
use motarjim_ast_html::{AlignItems, ComputedStyle};
use smol_str::SmolStr;

fn make_node(
    id: u32,
    semantic: SemanticIr,
    layout: LayoutIr,
    children: Vec<u32>,
    parent: Option<u32>,
) -> IrNode {
    IrNode {
        id: NodeId(id),
        semantic,
        layout,
        target: TargetIr::Generic {
            platform: SmolStr::new_inline("swiftui"),
            node: SmolStr::new_inline("View"),
        },
        computed_style: ComputedStyle::default(),
        children: children.into_iter().map(NodeId).collect(),
        parent: parent.map(NodeId),
        text: None,
    }
}

fn make_tree(nodes: Vec<IrNode>, root_id: u32) -> IrTree {
    IrTree {
        nodes,
        root_id: NodeId(root_id),
        target_hints: vec![],
    }
}

fn make_node_with_style(
    id: u32,
    semantic: SemanticIr,
    layout: LayoutIr,
    children: Vec<u32>,
    parent: Option<u32>,
    style: ComputedStyle,
) -> IrNode {
    IrNode {
        id: NodeId(id),
        semantic,
        layout,
        target: TargetIr::Generic {
            platform: SmolStr::new_inline("swiftui"),
            node: SmolStr::new_inline("View"),
        },
        computed_style: style,
        children: children.into_iter().map(NodeId).collect(),
        parent: parent.map(NodeId),
        text: None,
    }
}

#[test]
fn test_generates_import() {
    let tree = make_tree(
        vec![make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            vec![],
            None,
        )],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("import SwiftUI"));
}

#[test]
fn test_root_with_text() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Text, LayoutIr::Static, vec![], Some(0)),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("struct GeneratedPage: View"));
    assert!(output.contains("Text(\"text\")"));
}

#[test]
fn test_button_view() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Button, LayoutIr::Static, vec![], Some(0)),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Button("));
}

#[test]
fn test_vstack_and_hstack() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(
                1,
                SemanticIr::Column,
                LayoutIr::FlexColumn,
                vec![2],
                Some(0),
            ),
            make_node(2, SemanticIr::Row, LayoutIr::FlexRow, vec![], Some(1)),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("VStack"));
    assert!(output.contains("HStack"));
}

#[test]
fn test_image_view() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Image, LayoutIr::Static, vec![], Some(0)),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("AsyncImage"));
}

#[test]
fn test_heading_view() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(
                1,
                SemanticIr::Heading { level: 1 },
                LayoutIr::Static,
                vec![],
                Some(0),
            ),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Text(\"text\")"));
    assert!(output.contains("largeTitle"));
}

#[test]
fn test_list_view() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::List, LayoutIr::Stack, vec![2], Some(0)),
            make_node(2, SemanticIr::ListItem, LayoutIr::Static, vec![], Some(1)),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("List"));
}

#[test]
fn test_icon_and_divider() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1, 2], None),
            make_node(1, SemanticIr::Icon, LayoutIr::Static, vec![], Some(0)),
            make_node(2, SemanticIr::Divider, LayoutIr::Static, vec![], Some(0)),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Image(systemName: \"star.fill\")"));
    assert!(output.contains("Divider()"));
}

#[test]
fn test_empty_tree() {
    let tree = make_tree(
        vec![make_node(
            0,
            SemanticIr::Root,
            LayoutIr::FlexColumn,
            vec![],
            None,
        )],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Color.clear"));
}

#[test]
fn test_card_with_content() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Card, LayoutIr::Stack, vec![2], Some(0)),
            make_node(
                2,
                SemanticIr::Heading { level: 2 },
                LayoutIr::Static,
                vec![],
                Some(1),
            ),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains(".cornerRadius(12)"));
    assert!(output.contains("Text(\"text\")"));
}

#[test]
fn test_hstack_alignment_top() {
    let style = ComputedStyle {
        align_items: Some(AlignItems::FlexStart),
        ..Default::default()
    };
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node_with_style(
                1,
                SemanticIr::Row,
                LayoutIr::FlexRow,
                vec![],
                Some(0),
                style,
            ),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(
        output.contains("HStack(alignment: .top)"),
        "Expected HStack(alignment: .top), got:\n{output}"
    );
}

#[test]
fn test_hstack_alignment_bottom() {
    let style = ComputedStyle {
        align_items: Some(AlignItems::FlexEnd),
        ..Default::default()
    };
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node_with_style(
                1,
                SemanticIr::Row,
                LayoutIr::FlexRow,
                vec![],
                Some(0),
                style,
            ),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(
        output.contains("HStack(alignment: .bottom)"),
        "Expected HStack(alignment: .bottom), got:\n{output}"
    );
}

#[test]
fn test_hstack_alignment_center() {
    let style = ComputedStyle {
        align_items: Some(AlignItems::Center),
        ..Default::default()
    };
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node_with_style(
                1,
                SemanticIr::Row,
                LayoutIr::FlexRow,
                vec![],
                Some(0),
                style,
            ),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(
        output.contains("HStack(alignment: .center)"),
        "Expected HStack(alignment: .center), got:\n{output}"
    );
}

#[test]
fn test_vstack_alignment_leading() {
    let style = ComputedStyle {
        align_items: Some(AlignItems::FlexStart),
        ..Default::default()
    };
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node_with_style(
                1,
                SemanticIr::Column,
                LayoutIr::FlexColumn,
                vec![],
                Some(0),
                style,
            ),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(
        output.contains("VStack(alignment: .leading)"),
        "Expected VStack(alignment: .leading), got:\n{output}"
    );
}

#[test]
fn test_vstack_alignment_trailing() {
    let style = ComputedStyle {
        align_items: Some(AlignItems::FlexEnd),
        ..Default::default()
    };
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node_with_style(
                1,
                SemanticIr::Column,
                LayoutIr::FlexColumn,
                vec![],
                Some(0),
                style,
            ),
        ],
        0,
    );
    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);
    assert!(
        output.contains("VStack(alignment: .trailing)"),
        "Expected VStack(alignment: .trailing), got:\n{output}"
    );
}
