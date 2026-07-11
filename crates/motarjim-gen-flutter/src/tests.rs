use super::*;
use motarjim_ast::ir::{LayoutIr, SemanticIr, TargetIr};
use motarjim_ast::NodeId;
use motarjim_ast_html::ComputedStyle;
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
            platform: SmolStr::new_inline("flutter"),
            node: SmolStr::new_inline("Widget"),
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
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("import 'package:flutter/material.dart';"));
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
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("class GeneratedPage"));
    assert!(output.contains("Scaffold"));
    assert!(output.contains("Text"));
}

#[test]
fn test_button_widget() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Button, LayoutIr::Static, vec![], Some(0)),
        ],
        0,
    );
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("ElevatedButton"));
    assert!(output.contains("onPressed"));
}

#[test]
fn test_column_and_row() {
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
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Column("));
    assert!(output.contains("Row("));
}

#[test]
fn test_image_widget() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Image, LayoutIr::Static, vec![], Some(0)),
        ],
        0,
    );
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Image.network"));
}

#[test]
fn test_heading_widget() {
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
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    // heading with no text children falls back to "text"
    assert!(output.contains("'text',"));
    assert!(output.contains("fontSize: 32.0"));
}

#[test]
fn test_list_view() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::List, LayoutIr::Stack, vec![2, 3], Some(0)),
            make_node(2, SemanticIr::ListItem, LayoutIr::Static, vec![], Some(1)),
            make_node(3, SemanticIr::ListItem, LayoutIr::Static, vec![], Some(1)),
        ],
        0,
    );
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("ListView"));
    assert!(output.contains("ListTile"));
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
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Icon"));
    assert!(output.contains("Divider"));
}

#[test]
fn test_card_widget() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Card, LayoutIr::Stack, vec![2], Some(0)),
            make_node(2, SemanticIr::Text, LayoutIr::Static, vec![], Some(1)),
        ],
        0,
    );
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Card("));
    assert!(output.contains("child:"));
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
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("SizedBox.shrink"));
}

#[test]
fn test_text_field_widget() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Input, LayoutIr::Static, vec![], Some(0)),
        ],
        0,
    );
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("TextField"));
    assert!(output.contains("InputDecoration"));
}

#[test]
fn test_hero_section() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(
                1,
                SemanticIr::HeroSection,
                LayoutIr::Stack,
                vec![2],
                Some(0),
            ),
            make_node(
                2,
                SemanticIr::Heading { level: 1 },
                LayoutIr::Static,
                vec![],
                Some(1),
            ),
        ],
        0,
    );
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("SizedBox"));
    assert!(output.contains("'text',"));
}

#[test]
fn test_table_cell_widget() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Table, LayoutIr::Table, vec![2], Some(0)),
            make_node(
                2,
                SemanticIr::TableRow,
                LayoutIr::Table,
                vec![3],
                Some(1),
            ),
            make_node(3, SemanticIr::TableCell, LayoutIr::Table, vec![], Some(2)),
        ],
        0,
    );
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("TableCell("), "Expected TableCell widget, got:\n{output}");
    assert!(output.contains("TableRow("), "Expected TableRow widget, got:\n{output}");
}

#[test]
fn test_table_with_multiple_cells() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Table, LayoutIr::Table, vec![2, 3], Some(0)),
            make_node(
                2,
                SemanticIr::TableRow,
                LayoutIr::Table,
                vec![4, 5],
                Some(1),
            ),
            make_node(3, SemanticIr::TableRow, LayoutIr::Table, vec![6], Some(1)),
            make_node(4, SemanticIr::TableCell, LayoutIr::Table, vec![], Some(2)),
            make_node(5, SemanticIr::TableCell, LayoutIr::Table, vec![], Some(2)),
            make_node(6, SemanticIr::TableCell, LayoutIr::Table, vec![], Some(3)),
        ],
        0,
    );
    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);
    // Should have 3 TableCell widgets
    assert_eq!(output.matches("TableCell(").count(), 3);
    // Should have 2 TableRow widgets
    assert_eq!(output.matches("TableRow(").count(), 2);
}
