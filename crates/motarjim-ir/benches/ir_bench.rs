use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use motarjim_ast::{Attribute, Document, Element, HtmlNode, NodeId, NodeType};
use motarjim_ast_html::ComputedStyle;
use motarjim_diag::DiagnosticBag;
use motarjim_ir::IrBuilder;
use smallvec::SmallVec;

fn make_doc_small() -> Document {
    let mut doc = Document::new();
    let mut element = Element::new("div");
    element
        .attributes
        .push(Attribute::new("class", "container"));
    doc.nodes.push(HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Element,
        element: Some(element),
        value: None,
        children: smallvec::smallvec![NodeId(1)],
        parent: None,
        depth: 0,
        document_type: None,
    });
    doc.nodes.push(HtmlNode {
        id: NodeId(1),
        node_type: NodeType::Text,
        element: None,
        value: Some("Hello World".into()),
        children: SmallVec::new(),
        parent: Some(NodeId(0)),
        depth: 1,
        document_type: None,
    });
    doc.root_id = NodeId(0);
    doc
}

fn make_doc_medium() -> Document {
    let mut doc = Document::new();
    doc.nodes.push(HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Element,
        element: Some(Element::new("div")),
        value: None,
        children: smallvec::smallvec![NodeId(1), NodeId(2), NodeId(3)],
        parent: None,
        depth: 0,
        document_type: None,
    });
    doc.nodes.push(HtmlNode {
        id: NodeId(1),
        node_type: NodeType::Text,
        element: None,
        value: Some("Title".into()),
        children: SmallVec::new(),
        parent: Some(NodeId(0)),
        depth: 1,
        document_type: None,
    });
    let mut btn = Element::new("button");
    btn.attributes
        .push(Attribute::new("aria-label", "Click me"));
    doc.nodes.push(HtmlNode {
        id: NodeId(2),
        node_type: NodeType::Element,
        element: Some(btn),
        value: None,
        children: smallvec::smallvec![NodeId(4)],
        parent: Some(NodeId(0)),
        depth: 1,
        document_type: None,
    });
    doc.nodes.push(HtmlNode {
        id: NodeId(3),
        node_type: NodeType::Element,
        element: Some(Element::new("img")),
        value: None,
        children: SmallVec::new(),
        parent: Some(NodeId(0)),
        depth: 1,
        document_type: None,
    });
    doc.nodes.push(HtmlNode {
        id: NodeId(4),
        node_type: NodeType::Text,
        element: None,
        value: Some("Submit".into()),
        children: SmallVec::new(),
        parent: Some(NodeId(2)),
        depth: 2,
        document_type: None,
    });
    doc.root_id = NodeId(0);
    doc
}

fn make_styles(doc: &Document) -> HashMap<NodeId, ComputedStyle> {
    doc.nodes
        .iter()
        .map(|n| (n.id, ComputedStyle::default()))
        .collect()
}

fn bench_build_ir_small(c: &mut Criterion) {
    let doc = make_doc_small();
    let styles = make_styles(&doc);
    let builder = IrBuilder::new();

    c.bench_function("build_ir_small", |b| {
        b.iter(|| {
            let mut diag = DiagnosticBag::new();
            builder.build(black_box(&doc), black_box(&styles), black_box(&mut diag))
        });
    });
}

fn bench_build_ir_medium(c: &mut Criterion) {
    let doc = make_doc_medium();
    let styles = make_styles(&doc);
    let builder = IrBuilder::new();

    c.bench_function("build_ir_medium", |b| {
        b.iter(|| {
            let mut diag = DiagnosticBag::new();
            builder.build(black_box(&doc), black_box(&styles), black_box(&mut diag))
        });
    });
}

criterion_group!(benches, bench_build_ir_small, bench_build_ir_medium);
criterion_main!(benches);
