use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr, TargetIr};
use motarjim_ast::NodeId;
use motarjim_ast_html::ComputedStyle;
use motarjim_gen_flutter::FlutterGenerator;

fn main() {
    let tree = IrTree {
        nodes: vec![
            IrNode {
                id: NodeId(0),
                semantic: SemanticIr::Root,
                layout: LayoutIr::FlexColumn,
                target: TargetIr::Flutter {
                    widget: smol_str::SmolStr::new_inline("Column"),
                    properties: vec![],
                },
                computed_style: ComputedStyle::default(),
                children: smallvec::smallvec![NodeId(1)],
                parent: None,
                text: None,
            },
            IrNode {
                id: NodeId(1),
                semantic: SemanticIr::Text,
                layout: LayoutIr::Flow,
                target: TargetIr::Flutter {
                    widget: smol_str::SmolStr::new_inline("Text"),
                    properties: vec![(
                        smol_str::SmolStr::new_inline("data"),
                        "Hello from Flutter!".into(),
                    )],
                },
                computed_style: ComputedStyle::default(),
                children: smallvec::smallvec![],
                parent: Some(NodeId(0)),
                text: None,
            },
        ],
        root_id: NodeId(0),
        target_hints: vec![],
    };

    let gen = FlutterGenerator::new();
    let output = gen.generate(&tree);

    println!("{}", output);
}
