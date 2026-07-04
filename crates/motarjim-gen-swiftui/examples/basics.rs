use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr, TargetIr};
use motarjim_ast::style::ComputedStyle;
use motarjim_ast::NodeId;
use motarjim_gen_swiftui::SwiftUIGenerator;

fn main() {
    let tree = IrTree {
        nodes: vec![
            IrNode {
                id: NodeId(0),
                semantic: SemanticIr::Root,
                layout: LayoutIr::FlexColumn,
                target: TargetIr::SwiftUI {
                    view: smol_str::SmolStr::new_inline("VStack"),
                    properties: vec![],
                },
                computed_style: ComputedStyle::default(),
                children: smallvec::smallvec![NodeId(1)],
                parent: None,
            },
            IrNode {
                id: NodeId(1),
                semantic: SemanticIr::Text,
                layout: LayoutIr::Flow,
                target: TargetIr::SwiftUI {
                    view: smol_str::SmolStr::new_inline("Text"),
                    properties: vec![(
                        smol_str::SmolStr::new_inline("content"),
                        "Hello from SwiftUI!".into(),
                    )],
                },
                computed_style: ComputedStyle::default(),
                children: smallvec::smallvec![],
                parent: Some(NodeId(0)),
            },
        ],
        root_id: NodeId(0),
        target_hints: vec![],
    };

    let gen = SwiftUIGenerator::new();
    let output = gen.generate(&tree);

    println!("{}", output);
}
