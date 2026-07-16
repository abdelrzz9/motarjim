use criterion::{black_box, criterion_group, criterion_main, Criterion};
use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr, TargetIr};
use motarjim_ast::NodeId;
use motarjim_ast_html::ComputedStyle;
use motarjim_optimizer::{
    register_default_passes, CollapseWhitespace, MergeAdjacentText, PassManager, RemoveEmptyNodes,
};
use smallvec::SmallVec;
use smol_str::SmolStr;

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
            platform: SmolStr::new_inline("default"),
            node: SmolStr::new_inline("Widget"),
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
            widget: SmolStr::new_inline("Text"),
            properties: vec![(SmolStr::new_inline("data"), text.to_string())],
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

fn make_small_tree() -> IrTree {
    IrTree {
        nodes: vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1), NodeId(2)],
                None,
            ),
            make_text_node(1, "Hello    World", SmallVec::new(), Some(NodeId(0))),
            make_text_node(2, "", SmallVec::new(), Some(NodeId(0))),
        ],
        root_id: NodeId(0),
        target_hints: vec![],
    }
}

fn make_medium_tree() -> IrTree {
    IrTree {
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
        target_hints: vec![],
    }
}

fn bench_optimize_small(c: &mut Criterion) {
    let tree = make_small_tree();

    c.bench_function("optimize_small", |b| {
        b.iter(|| {
            let mut t = black_box(tree.clone());
            let mut pm = PassManager::new();
            pm.register(Box::new(RemoveEmptyNodes::default()));
            pm.register(Box::new(CollapseWhitespace::default()));
            pm.register(Box::new(MergeAdjacentText::default()));
            pm.run_all(&mut t)
        });
    });
}

fn bench_optimize_medium(c: &mut Criterion) {
    let tree = make_medium_tree();

    c.bench_function("optimize_medium", |b| {
        b.iter(|| {
            let mut t = black_box(tree.clone());
            let mut pm = PassManager::new();
            register_default_passes(&mut pm);
            pm.run_all(&mut t)
        });
    });
}

criterion_group!(benches, bench_optimize_small, bench_optimize_medium);
criterion_main!(benches);
