use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr, TargetIr};
use motarjim_ast::NodeId;
use motarjim_ast_html::ComputedStyle;
use motarjim_optimizer::{register_default_passes, PassManager};

fn make_text_node(id: u32, text: &str) -> IrNode {
    IrNode {
        id: NodeId(id),
        semantic: SemanticIr::Text,
        layout: LayoutIr::Flow,
        target: TargetIr::Flutter {
            widget: smol_str::SmolStr::new_inline("Text"),
            properties: vec![(smol_str::SmolStr::new_inline("data"), text.to_string())],
        },
        computed_style: ComputedStyle::default(),
        children: smallvec::smallvec![],
        parent: None,
        text: None,
        responsive: Vec::new(),
        events: Vec::new(),
        text_direction: None,
    }
}

fn make_container(
    id: u32,
    children: smallvec::SmallVec<[NodeId; 4]>,
    parent: Option<NodeId>,
) -> IrNode {
    IrNode {
        id: NodeId(id),
        semantic: SemanticIr::Container,
        layout: LayoutIr::FlexColumn,
        target: TargetIr::Flutter {
            widget: smol_str::SmolStr::new_inline("Container"),
            properties: vec![],
        },
        computed_style: ComputedStyle::default(),
        children,
        parent,
        text: None,
        responsive: Vec::new(),
        events: Vec::new(),
        text_direction: None,
    }
}

fn main() {
    let mut tree = IrTree {
        nodes: vec![
            make_container(0, smallvec::smallvec![NodeId(1), NodeId(2)], None),
            make_container(1, smallvec::smallvec![NodeId(3)], Some(NodeId(0))),
            make_text_node(2, "  Hello   World  "),
            make_text_node(3, "Nested"),
        ],
        root_id: NodeId(0),
        target_hints: vec![],
    };

    let mut pm = PassManager::new();
    register_default_passes(&mut pm);

    println!("Before: {} node(s)", tree.nodes.len());
    println!("Passes registered: {}", pm.pass_count());

    let results = pm.run_all(&mut tree);

    println!("\nAfter:  {} node(s)", tree.nodes.len());
    for r in &results {
        println!(
            "  {}: visited={}, modified={}, removed={}, took={}ns",
            r.pass_name, r.nodes_visited, r.nodes_modified, r.nodes_removed, r.duration_ns
        );
    }
}
