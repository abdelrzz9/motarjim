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
            platform: SmolStr::new_inline("compose"),
            node: SmolStr::new_inline("Composable"),
        },
        computed_style: ComputedStyle::default(),
        children: children.into_iter().map(NodeId).collect(),
        parent: parent.map(NodeId),
        text: None,
        responsive: Vec::new(),
        events: Vec::new(),
        text_direction: None,
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
fn test_generates_imports() {
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
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("import androidx.compose.material3.*"));
    assert!(output.contains("@Composable"));
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
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("fun GeneratedPage()"));
    assert!(output.contains("Text(\"text\")"));
}

#[test]
fn test_button_composable() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Button, LayoutIr::Static, vec![], Some(0)),
        ],
        0,
    );
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Button(onClick"));
    assert!(output.contains("Text(\"Button\")"));
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
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Column("));
    assert!(output.contains("Row("));
}

#[test]
fn test_image_composable() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Image, LayoutIr::Static, vec![], Some(0)),
        ],
        0,
    );
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Image("));
}

#[test]
fn test_heading_composable() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(
                1,
                SemanticIr::Heading { level: 2 },
                LayoutIr::Static,
                vec![],
                Some(0),
            ),
        ],
        0,
    );
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("text = \"text\""));
    assert!(output.contains("fontSize = 28.sp"));
}

#[test]
fn test_card_composable() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Card, LayoutIr::Stack, vec![2], Some(0)),
            make_node(2, SemanticIr::Text, LayoutIr::Static, vec![], Some(1)),
        ],
        0,
    );
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Card("));
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
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Icon("));
    assert!(output.contains("HorizontalDivider"));
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
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("empty page"));
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
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("Box("));
    assert!(output.contains("text = \"text\""));
}

#[test]
fn test_dialog_composable() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Dialog, LayoutIr::Stack, vec![2], Some(0)),
            make_node(
                2,
                SemanticIr::Text,
                LayoutIr::Static,
                vec![],
                Some(1),
            ),
        ],
        0,
    );
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("AlertDialog(") || output.contains("Dialog("));
}

#[test]
fn test_table_composable() {
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
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    // Compose falls back to Text for unsupported table widgets
    assert!(!output.is_empty());
}

#[test]
fn test_scroll_composable() {
    let tree = make_tree(
        vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Scroll, LayoutIr::Scroll, vec![2], Some(0)),
            make_node(2, SemanticIr::Text, LayoutIr::Static, vec![], Some(1)),
        ],
        0,
    );
    let gen = ComposeGenerator::new();
    let output = gen.generate(&tree);
    assert!(output.contains("LazyColumn(") || output.contains("Column("));
}
