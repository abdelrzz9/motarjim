use criterion::{black_box, criterion_group, criterion_main, Criterion};
use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr, TargetIr};
use motarjim_ast::NodeId;
use motarjim_ast_html::ComputedStyle;
use motarjim_gen_compose::ComposeGenerator;
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
    }
}

fn make_small_tree() -> IrTree {
    IrTree {
        nodes: vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(1, SemanticIr::Text, LayoutIr::Static, vec![], Some(0)),
        ],
        root_id: NodeId(0),
        target_hints: vec![],
    }
}

fn make_medium_tree() -> IrTree {
    IrTree {
        nodes: vec![
            make_node(0, SemanticIr::Root, LayoutIr::FlexColumn, vec![1], None),
            make_node(
                1,
                SemanticIr::Column,
                LayoutIr::FlexColumn,
                vec![2, 3, 4],
                Some(0),
            ),
            make_node(2, SemanticIr::Button, LayoutIr::Static, vec![], Some(1)),
            make_node(3, SemanticIr::Text, LayoutIr::Static, vec![], Some(1)),
            make_node(4, SemanticIr::Row, LayoutIr::FlexRow, vec![5, 6], Some(1)),
            make_node(5, SemanticIr::Image, LayoutIr::Static, vec![], Some(4)),
            make_node(6, SemanticIr::Icon, LayoutIr::Static, vec![], Some(4)),
        ],
        root_id: NodeId(0),
        target_hints: vec![],
    }
}

fn bench_generate_compose_small(c: &mut Criterion) {
    let tree = make_small_tree();
    let gen = ComposeGenerator::new();

    c.bench_function("generate_compose_small", |b| {
        b.iter(|| gen.generate(black_box(&tree)));
    });
}

fn bench_generate_compose_medium(c: &mut Criterion) {
    let tree = make_medium_tree();
    let gen = ComposeGenerator::new();

    c.bench_function("generate_compose_medium", |b| {
        b.iter(|| gen.generate(black_box(&tree)));
    });
}

criterion_group!(
    benches,
    bench_generate_compose_small,
    bench_generate_compose_medium
);
criterion_main!(benches);
