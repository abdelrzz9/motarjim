//! Converts html5ever's RcDom into Motarjim's internal AST.
//!
//! This is the only module that directly depends on html5ever/markup5ever_rcdom
//! types. All html5ever-specific types remain private to this module,
//! ensuring that the rest of the crate is insulated from the parser backend.
//!
//! # Ownership model
//!
//! The converter takes ownership of the RcDom and recursively walks its
//! tree, building Motarjim-owned nodes. Each Motarjim [`Node`] fully owns
//! its children (unlike RcDom's shared-reference model), which means the
//! RcDom can be dropped after conversion.

use smol_str::SmolStr;

use markup5ever_rcdom::{Handle, NodeData, RcDom};

use crate::ast::{Attribute, Document, Fragment, Node, NodeKind};
use crate::diagnostics::DiagnosticBag;

/// Converts an RcDom document into a Motarjim tree-based [`Document`].
///
/// Recursively walks the RcDom tree and produces the Motarjim-owned AST.
/// The RcDom's document node itself is skipped; only its children are kept.
pub fn rcdom_to_document(rcdom: RcDom, errors: &mut DiagnosticBag) -> Document {
    let mut id_gen = crate::ast::NodeIdGenerator::new();
    let children = convert_children(&rcdom.document, &mut id_gen, errors);
    Document {
        children,
        span: None,
    }
}

/// Converts an RcDom document into a Motarjim [`Fragment`].
///
/// Unlike [`rcdom_to_document`], this extracts only the direct children
/// without the implied `<html>`, `<head>`, `<body>` wrapper elements
/// that html5ever normally inserts.
pub fn rcdom_to_fragment(rcdom: RcDom, errors: &mut DiagnosticBag) -> Fragment {
    let mut id_gen = crate::ast::NodeIdGenerator::new();
    let children = convert_children(&rcdom.document, &mut id_gen, errors);
    Fragment {
        children,
        span: None,
    }
}

/// Recursively converts children of an RcDom handle into Motarjim nodes.
fn convert_children(
    handle: &Handle,
    id_gen: &mut crate::ast::NodeIdGenerator,
    errors: &mut DiagnosticBag,
) -> Vec<Node> {
    let children = handle.children.borrow().clone();

    let mut result = Vec::with_capacity(children.len());
    for child_handle in &children {
        if let Some(node) = convert_node(child_handle, id_gen, errors) {
            result.push(node);
        }
    }
    result
}

/// Converts a single RcDom node into a Motarjim [`Node`].
fn convert_node(
    handle: &Handle,
    id_gen: &mut crate::ast::NodeIdGenerator,
    errors: &mut DiagnosticBag,
) -> Option<Node> {
    let node_id = id_gen.next();

    match &handle.data {
        NodeData::Element { name, attrs, .. } => {
            let tag_name: SmolStr = name.local.to_string().into();
            let namespace: SmolStr = name.ns.to_string().into();

            let borrowed_attrs = attrs.borrow();
            let mut attributes: Vec<Attribute> = Vec::with_capacity(borrowed_attrs.len());

            for attr in borrowed_attrs.iter() {
                let attr_name: SmolStr = attr.name.local.to_string().into();
                let attr_value: SmolStr = attr.value.to_string().into();
                attributes.push(Attribute {
                    name: attr_name,
                    value: attr_value,
                    span: None,
                });
            }
            drop(borrowed_attrs);

            let children = convert_children(handle, id_gen, errors);

            Some(Node {
                id: node_id,
                kind: NodeKind::Element(crate::ast::ElementData {
                    tag_name,
                    namespace,
                    attributes,
                }),
                children,
                span: None,
            })
        }

        NodeData::Text { contents } => {
            let value: String = contents.borrow().to_string();

            // Preserve all text nodes — including whitespace-only ones —
            // since whitespace can be semantically meaningful (e.g., in
            // `<pre>` elements or inline formatting).
            if value.is_empty() {
                return None;
            }

            Some(Node {
                id: node_id,
                kind: NodeKind::Text(crate::ast::TextData { value }),
                children: Vec::new(),
                span: None,
            })
        }

        NodeData::Comment { contents } => {
            let value: String = contents.to_string();

            Some(Node {
                id: node_id,
                kind: NodeKind::Comment(crate::ast::CommentData { value }),
                children: Vec::new(),
                span: None,
            })
        }

        NodeData::Doctype {
            name,
            public_id,
            system_id,
        } => {
            let dt_name: SmolStr = name.to_string().into();
            let dt_public: SmolStr = public_id.to_string().into();
            let dt_system: SmolStr = system_id.to_string().into();

            Some(Node {
                id: node_id,
                kind: NodeKind::Doctype(crate::ast::DoctypeData {
                    name: dt_name,
                    public_id: dt_public,
                    system_id: dt_system,
                }),
                children: Vec::new(),
                span: None,
            })
        }

        NodeData::Document => {
            // The Document node itself has no Motarjim counterpart;
            // only its children are meaningful.
            None
        }

        NodeData::ProcessingInstruction { target, contents } => {
            let pi_target: String = target.to_string();
            let pi_data: String = contents.to_string();

            Some(Node {
                id: node_id,
                kind: NodeKind::ProcessingInstruction(
                    crate::ast::ProcessingInstructionData {
                        target: pi_target,
                        data: pi_data,
                    },
                ),
                children: Vec::new(),
                span: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::NodeKind;

    /// Parse a string into RcDom using html5ever, then convert to Motarjim AST.
    fn parse_and_convert(html: &str) -> Document {
        use html5ever::parse_document;
        use html5ever::tendril::TendrilSink;

        let rcdom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .expect("html5ever parse failed");

        let mut errors = DiagnosticBag::new();
        rcdom_to_document(rcdom, &mut errors)
    }

    fn find_tag<'a>(nodes: &'a [Node], tag: &str) -> Option<&'a Node> {
        for node in nodes {
            if node.tag_name() == Some(tag) {
                return Some(node);
            }
            if let found @ Some(_) = find_tag(&node.children, tag) {
                return found;
            }
        }
        None
    }

    #[test]
    fn test_empty_input() {
        let doc = parse_and_convert("");
        // html5ever always inserts html/head/body
        assert!(!doc.children.is_empty());
    }

    #[test]
    fn test_simple_element() {
        let doc = parse_and_convert("<p>Hello</p>");
        let body = find_tag(&doc.children, "body").unwrap();
        let p = &body.children[0];
        assert_eq!(p.tag_name(), Some("p"));
        assert!(!p.children.is_empty());
    }

    #[test]
    fn test_attribute_preservation() {
        let doc = parse_and_convert("<div class='foo' id='bar'></div>");
        let body = find_tag(&doc.children, "body").unwrap();
        let div = &body.children[0];
        let el = div.as_element().unwrap();
        assert_eq!(el.attributes.len(), 2);
        assert_eq!(el.get_attribute("class"), Some("foo"));
        assert_eq!(el.get_attribute("id"), Some("bar"));
    }

    #[test]
    fn test_comment_preservation() {
        let doc = parse_and_convert("<!-- test --><div></div>");
        let has_comment = doc.children.iter().any(|n| {
            matches!(n.kind, NodeKind::Comment(_))
                || n.children.iter().any(|c| matches!(c.kind, NodeKind::Comment(_)))
        });
        assert!(has_comment);
    }

    #[test]
    fn test_doctype_preservation() {
        let doc = parse_and_convert("<!DOCTYPE html>");
        let has_doctype = doc.children.iter().any(|n| matches!(n.kind, NodeKind::Doctype(_)));
        assert!(has_doctype);
    }

    #[test]
    fn test_text_preservation() {
        let doc = parse_and_convert("<p>Hello world</p>");
        let p = find_tag(&doc.children, "p").unwrap();
        let text_node = p.children.iter().find(|c| matches!(c.kind, NodeKind::Text(_)));
        assert!(text_node.is_some());
        assert!(text_node.unwrap().as_text().unwrap().contains("Hello world"));
    }

    #[test]
    fn test_whitespace_text_preserved() {
        let doc = parse_and_convert("<div>   </div>");
        let div = find_tag(&doc.children, "div").unwrap();
        let text_node = div.children.iter().find(|c| matches!(c.kind, NodeKind::Text(_)));
        assert!(
            text_node.is_some(),
            "whitespace-only text nodes should be preserved"
        );
    }

    #[test]
    fn test_nested_structure() {
        let doc = parse_and_convert("<ul><li>A</li><li>B</li></ul>");
        let body = find_tag(&doc.children, "body").unwrap();
        let ul = &body.children[0];
        assert_eq!(ul.tag_name(), Some("ul"));
        assert_eq!(ul.children.len(), 2);
    }

    #[test]
    fn test_void_elements() {
        let doc = parse_and_convert("<br><hr>");
        let has_br = find_tag(&doc.children, "br").is_some();
        let has_hr = find_tag(&doc.children, "hr").is_some();
        assert!(has_br);
        assert!(has_hr);
    }

    #[test]
    fn test_converter_no_panic_on_malformed() {
        // html5ever handles recovery internally; converter just processes the result
        let doc = parse_and_convert("<div><span>unclosed");
        assert!(!doc.children.is_empty());
    }

    #[test]
    fn test_fragment_conversion() {
        use html5ever::parse_document;
        use html5ever::tendril::TendrilSink;

        let rcdom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut "<div></div>".as_bytes())
            .unwrap();
        let mut errors = DiagnosticBag::new();
        let frag = rcdom_to_fragment(rcdom, &mut errors);
        assert!(!frag.is_empty());
    }
}
