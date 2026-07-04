use motarjim_ast::{Attribute, Document, Element, HtmlNode, NodeId, NodeType};

fn main() {
    let mut doc = Document::new();

    doc.nodes.push(HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Document,
        element: None,
        value: None,
        children: smallvec::smallvec![NodeId(1), NodeId(2)],
        parent: None,
        depth: 0,
        document_type: None,
    });

    let mut div = Element::new("div");
    div.attributes
        .push(Attribute::new("class", "container"));

    doc.nodes.push(HtmlNode {
        id: NodeId(1),
        node_type: NodeType::Element,
        element: Some(div),
        value: None,
        children: smallvec::smallvec![],
        parent: Some(NodeId(0)),
        depth: 1,
        document_type: None,
    });

    doc.nodes.push(HtmlNode {
        id: NodeId(2),
        node_type: NodeType::Text,
        element: None,
        value: Some("Hello, world!".into()),
        children: smallvec::smallvec![],
        parent: Some(NodeId(0)),
        depth: 1,
        document_type: None,
    });

    println!("Document has {} node(s)", doc.nodes.len());

    for node in &doc.nodes {
        let tag = node
            .element
            .as_ref()
            .map(|e| e.tag_name.as_str())
            .unwrap_or(match node.node_type {
                NodeType::Document => "document",
                NodeType::Text => "text",
                NodeType::Comment => "comment",
                NodeType::DocumentType => "doctype",
                NodeType::Fragment => "fragment",
                NodeType::Element => "element",
            });
        let text = node.value.as_deref().unwrap_or("");
        println!("  [{:?}] <{}> {}", node.id, tag, text);
    }
}
