use std::collections::HashMap;

use motarjim_ast::style::ComputedStyle;
use motarjim_ast::{Document, Element, HtmlNode, NodeId, NodeType};
use motarjim_diag::DiagnosticBag;
use motarjim_ir::IrBuilder;

fn main() {
    let mut doc = Document::new();

    doc.nodes.push(HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Document,
        element: None,
        value: None,
        children: smallvec::smallvec![NodeId(1)],
        parent: None,
        depth: 0,
        document_type: None,
    });

    doc.nodes.push(HtmlNode {
        id: NodeId(1),
        node_type: NodeType::Element,
        element: Some(Element::new("div")),
        value: None,
        children: smallvec::smallvec![],
        parent: Some(NodeId(0)),
        depth: 1,
        document_type: None,
    });

    let mut styles = HashMap::new();
    styles.insert(NodeId(0), ComputedStyle::default());
    styles.insert(NodeId(1), ComputedStyle::default());

    let diags = DiagnosticBag::new();
    let builder = IrBuilder::new();
    let ir = builder.build(&doc, &styles, &diags);

    println!("Built IR tree with {} node(s)", ir.nodes.len());
    for node in &ir.nodes {
        println!(
            "  [{}] semantic={:?}  children={}",
            node.id.0,
            node.semantic,
            node.children.len()
        );
    }
}
