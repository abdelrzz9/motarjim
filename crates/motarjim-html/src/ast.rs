//! Motarjim-owned HTML AST types.
//!
//! These types are completely independent of html5ever and represent
//! the compiler's internal view of an HTML document as a tree of nodes.
//!
//! The tree structure (`Node` contains `Vec<Node>` children) differs from
//! the arena-based representation in `motarjim_ast_html` which uses a flat
//! `Vec<HtmlNode>` with `NodeId` references for children.

use smol_str::SmolStr;

use crate::span::SourceSpan;

/// A unique identifier for a node in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u32);

impl NodeId {
    /// The root node identifier (always 0).
    pub const ROOT: Self = Self(0);
}

/// An HTML attribute (name-value pair).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// The attribute name.
    pub name: SmolStr,
    /// The attribute value.
    pub value: SmolStr,
    /// The source location of this attribute, if available.
    pub span: Option<SourceSpan>,
}

impl Attribute {
    /// Creates a new attribute without source location.
    pub fn new(name: impl Into<SmolStr>, value: impl Into<SmolStr>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            span: None,
        }
    }

    /// Creates a new attribute with a source location.
    pub fn with_span(
        name: impl Into<SmolStr>,
        value: impl Into<SmolStr>,
        span: SourceSpan,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            span: Some(span),
        }
    }
}

/// The kind of an HTML tree node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// An HTML element node with a tag name, namespace, and attributes.
    Element(ElementData),
    /// A text content node.
    Text(TextData),
    /// An HTML comment.
    Comment(CommentData),
    /// A DOCTYPE declaration.
    Doctype(DoctypeData),
    /// A processing instruction (e.g., `<?xml ...?>`).
    ProcessingInstruction(ProcessingInstructionData),
}

/// Data for an HTML element node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementData {
    /// The tag name (e.g., "div", "span").
    pub tag_name: SmolStr,
    /// The namespace URI (e.g., `http://www.w3.org/1999/xhtml`).
    pub namespace: SmolStr,
    /// The element's attributes.
    pub attributes: Vec<Attribute>,
}

impl ElementData {
    /// Returns the value of the first attribute with the given name.
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|a| a.name.as_str() == name)
            .map(|a| a.value.as_str())
    }
}

/// Data for a text node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextData {
    /// The text content.
    pub value: String,
}

impl TextData {
    /// Returns `true` if the text consists only of whitespace.
    pub fn is_whitespace(&self) -> bool {
        self.value.chars().all(char::is_whitespace)
    }
}

/// Data for an HTML comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentData {
    /// The comment text.
    pub value: String,
}

/// Data for a DOCTYPE declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctypeData {
    /// The document type name.
    pub name: SmolStr,
    /// The public identifier.
    pub public_id: SmolStr,
    /// The system identifier.
    pub system_id: SmolStr,
}

/// Data for a processing instruction (e.g., `<?xml ...?>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessingInstructionData {
    /// The target (e.g., "xml").
    pub target: String,
    /// The data/content of the instruction.
    pub data: String,
}

/// A single node in the HTML tree.
///
/// Nodes form a recursive tree structure where each node owns its children.
/// This is in contrast to the arena-based representation used elsewhere
/// in the Motarjim pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The unique identifier for this node.
    pub id: NodeId,
    /// The kind of this node (element, text, comment, doctype).
    pub kind: NodeKind,
    /// The children of this node.
    pub children: Vec<Node>,
    /// The source location of this node, if available.
    pub span: Option<SourceSpan>,
}

impl Node {
    /// Creates a new element node.
    pub fn element(
        id: NodeId,
        tag_name: impl Into<SmolStr>,
        namespace: impl Into<SmolStr>,
        attributes: Vec<Attribute>,
    ) -> Self {
        Self {
            id,
            kind: NodeKind::Element(ElementData {
                tag_name: tag_name.into(),
                namespace: namespace.into(),
                attributes,
            }),
            children: Vec::new(),
            span: None,
        }
    }

    /// Creates a new text node.
    pub fn text(id: NodeId, value: impl Into<String>) -> Self {
        Self {
            id,
            kind: NodeKind::Text(TextData {
                value: value.into(),
            }),
            children: Vec::new(),
            span: None,
        }
    }

    /// Creates a new comment node.
    pub fn comment(id: NodeId, value: impl Into<String>) -> Self {
        Self {
            id,
            kind: NodeKind::Comment(CommentData {
                value: value.into(),
            }),
            children: Vec::new(),
            span: None,
        }
    }

    /// Creates a new doctype node.
    pub fn doctype(
        id: NodeId,
        name: impl Into<SmolStr>,
        public_id: impl Into<SmolStr>,
        system_id: impl Into<SmolStr>,
    ) -> Self {
        Self {
            id,
            kind: NodeKind::Doctype(DoctypeData {
                name: name.into(),
                public_id: public_id.into(),
                system_id: system_id.into(),
            }),
            children: Vec::new(),
            span: None,
        }
    }

    /// Creates a new processing instruction node.
    pub fn processing_instruction(
        id: NodeId,
        target: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        Self {
            id,
            kind: NodeKind::ProcessingInstruction(ProcessingInstructionData {
                target: target.into(),
                data: data.into(),
            }),
            children: Vec::new(),
            span: None,
        }
    }

    /// Returns the tag name if this is an element node.
    pub fn tag_name(&self) -> Option<&str> {
        match &self.kind {
            NodeKind::Element(e) => Some(e.tag_name.as_str()),
            _ => None,
        }
    }

    /// Returns the element data if this is an element node.
    pub fn as_element(&self) -> Option<&ElementData> {
        match &self.kind {
            NodeKind::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Returns mutable element data if this is an element node.
    pub fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        match &mut self.kind {
            NodeKind::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the text content if this is a text node.
    pub fn as_text(&self) -> Option<&str> {
        match &self.kind {
            NodeKind::Text(t) => Some(t.value.as_str()),
            _ => None,
        }
    }

    /// Returns the comment content if this is a comment node.
    pub fn as_comment(&self) -> Option<&str> {
        match &self.kind {
            NodeKind::Comment(c) => Some(c.value.as_str()),
            _ => None,
        }
    }

    /// Returns the doctype data if this is a doctype node.
    pub fn as_doctype(&self) -> Option<&DoctypeData> {
        match &self.kind {
            NodeKind::Doctype(d) => Some(d),
            _ => None,
        }
    }

    /// Returns the processing instruction data if this is a PI node.
    pub fn as_processing_instruction(&self) -> Option<&ProcessingInstructionData> {
        match &self.kind {
            NodeKind::ProcessingInstruction(pi) => Some(pi),
            _ => None,
        }
    }

    /// Returns `true` if this node is an element with the given tag name.
    pub fn is_tag(&self, name: &str) -> bool {
        self.tag_name() == Some(name)
    }

    /// Returns the first child matching the given tag name.
    pub fn find_child(&self, tag: &str) -> Option<&Node> {
        self.children.iter().find(|child| child.is_tag(tag))
    }

    /// Collects all text nodes in this subtree into a single string.
    pub fn text_content(&self) -> String {
        let mut result = String::new();
        self.collect_text(&mut result);
        result
    }

    /// Collects text content into `result` recursively.
    fn collect_text(&self, result: &mut String) {
        if let NodeKind::Text(t) = &self.kind {
            result.push_str(&t.value);
        }
        for child in &self.children {
            child.collect_text(result);
        }
    }

    /// Returns the depth of this node in the tree (if computed).
    /// This is computed on the fly by counting ancestors.
    pub fn depth_in(&self, ancestors: &[NodeId]) -> usize {
        ancestors.len()
    }
}

/// The root document, containing a tree of nodes.
///
/// Unlike the arena-based `motarjim_ast_html::Document`, this uses a
/// recursive tree structure where each [`Node`] directly owns its children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// The root children of the document (typically `<html>`, doctype, etc.).
    pub children: Vec<Node>,
    /// The source location of the document, if available.
    pub span: Option<SourceSpan>,
}

impl Document {
    /// Creates a new empty document.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            span: None,
        }
    }

    /// Creates a new document with the given children.
    pub fn with_children(children: Vec<Node>) -> Self {
        Self {
            children,
            span: None,
        }
    }

    /// Returns `true` if the document has no children.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Returns the number of children.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns the first child with the given tag name.
    pub fn find_tag(&self, tag: &str) -> Option<&Node> {
        find_tag_recursive(&self.children, tag)
    }

    /// Returns all nodes with the given tag name.
    pub fn find_tags(&self, tag: &str) -> Vec<&Node> {
        let mut result = Vec::new();
        collect_tags_recursive(&self.children, tag, &mut result);
        result
    }

    /// Returns the concatenated text content of the document.
    pub fn text_content(&self) -> String {
        let mut result = String::new();
        for child in &self.children {
            child.collect_text(&mut result);
        }
        result
    }

    /// Returns the root `<html>` element if present.
    pub fn root_element(&self) -> Option<&Node> {
        self.find_tag("html")
    }

    /// Returns the `<head>` element if present.
    pub fn head(&self) -> Option<&Node> {
        self.root_element().and_then(|html| html.find_child("head"))
    }

    /// Returns the `<body>` element if present.
    pub fn body(&self) -> Option<&Node> {
        self.root_element().and_then(|html| html.find_child("body"))
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// A parsed HTML fragment (not a full document).
///
/// Unlike [`Document`], a fragment does not get the implied `<html>`,
/// `<head>`, and `<body>` elements added by the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    /// The fragment's child nodes.
    pub children: Vec<Node>,
    /// The source location of the fragment, if available.
    pub span: Option<SourceSpan>,
}

impl Fragment {
    /// Creates a new empty fragment.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            span: None,
        }
    }

    /// Creates a new fragment with the given children.
    pub fn with_children(children: Vec<Node>) -> Self {
        Self {
            children,
            span: None,
        }
    }

    /// Returns `true` if the fragment has no children.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

impl Default for Fragment {
    fn default() -> Self {
        Self::new()
    }
}

/// A node ID generator for building trees.
#[derive(Debug, Clone, Default)]
pub struct NodeIdGenerator {
    /// The next ID to allocate.
    next: u32,
}

impl NodeIdGenerator {
    /// Creates a new ID generator starting at 0.
    pub fn new() -> Self {
        Self { next: 0 }
    }

    /// Allocates the next unique node ID.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next += 1;
        NodeId(id.0)
    }

    /// Resets the generator back to 0.
    pub fn reset(&mut self) {
        self.next = 0;
    }
}

/// Searches for a node with the given tag name in the subtree.
fn find_tag_recursive<'a>(nodes: &'a [Node], tag: &str) -> Option<&'a Node> {
    for node in nodes {
        if node.is_tag(tag) {
            return Some(node);
        }
        if let found @ Some(_) = find_tag_recursive(&node.children, tag) {
            return found;
        }
    }
    None
}

/// Collects all nodes with the given tag name from the subtree into `result`.
fn collect_tags_recursive<'a>(nodes: &'a [Node], tag: &str, result: &mut Vec<&'a Node>) {
    for node in nodes {
        if node.is_tag(tag) {
            result.push(node);
        }
        collect_tags_recursive(&node.children, tag, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::BytePos;

    #[test]
    fn test_attribute() {
        let attr = Attribute::new("class", "container");
        assert_eq!(attr.name.as_str(), "class");
        assert_eq!(attr.value.as_str(), "container");
        assert!(attr.span.is_none());
    }

    #[test]
    fn test_attribute_with_span() {
        let span = SourceSpan::new(BytePos(0), BytePos(5));
        let attr = Attribute::with_span("id", "main", span);
        assert_eq!(attr.span, Some(span));
    }

    #[test]
    fn test_element_node() {
        let attrs = vec![Attribute::new("class", "foo"), Attribute::new("id", "bar")];
        let node = Node::element(
            NodeId(1),
            "div",
            "http://www.w3.org/1999/xhtml",
            attrs.clone(),
        );
        assert_eq!(node.id, NodeId(1));
        assert_eq!(node.tag_name(), Some("div"));
        assert!(node.as_text().is_none());
        assert!(node.as_comment().is_none());

        let el = node.as_element().expect("should be element");
        assert_eq!(el.attributes.len(), 2);
        assert_eq!(el.attributes[0].name.as_str(), "class");
    }

    #[test]
    fn test_text_node() {
        let node = Node::text(NodeId(2), "Hello, world!");
        assert_eq!(node.id, NodeId(2));
        assert_eq!(node.as_text(), Some("Hello, world!"));
        assert!(node.as_element().is_none());
    }

    #[test]
    fn test_comment_node() {
        let node = Node::comment(NodeId(3), "a comment");
        assert_eq!(node.as_comment(), Some("a comment"));
    }

    #[test]
    fn test_doctype_node() {
        let node = Node::doctype(NodeId(4), "html", "", "");
        let dt = node.as_doctype().expect("should be doctype");
        assert_eq!(dt.name.as_str(), "html");
    }

    #[test]
    fn test_processing_instruction_node() {
        let node = Node::processing_instruction(NodeId(5), "xml", "version='1.0'");
        let pi = node.as_processing_instruction().expect("should be PI");
        assert_eq!(pi.target, "xml");
        assert_eq!(pi.data, "version='1.0'");
    }

    #[test]
    fn test_node_children() {
        let mut parent = Node::element(NodeId(0), "ul", "http://www.w3.org/1999/xhtml", vec![]);
        let child1 = Node::element(NodeId(1), "li", "http://www.w3.org/1999/xhtml", vec![]);
        let child2 = Node::text(NodeId(2), "item");

        parent.children.push(child1);
        parent.children.push(child2);

        assert_eq!(parent.children.len(), 2);
        assert_eq!(parent.children[0].tag_name(), Some("li"));
        assert_eq!(parent.children[1].as_text(), Some("item"));
    }

    #[test]
    fn test_find_child() {
        let mut div = Node::element(NodeId(0), "div", "http://www.w3.org/1999/xhtml", vec![]);
        div.children.push(Node::element(
            NodeId(1),
            "span",
            "http://www.w3.org/1999/xhtml",
            vec![],
        ));
        assert!(div.find_child("span").is_some());
        assert!(div.find_child("p").is_none());
    }

    #[test]
    fn test_text_content() {
        let mut p = Node::element(NodeId(0), "p", "http://www.w3.org/1999/xhtml", vec![]);
        p.children.push(Node::text(NodeId(1), "Hello, "));
        p.children.push(Node::text(NodeId(2), "world!"));
        assert_eq!(p.text_content(), "Hello, world!");
    }

    #[test]
    fn test_is_tag() {
        let node = Node::element(NodeId(0), "div", "http://www.w3.org/1999/xhtml", vec![]);
        assert!(node.is_tag("div"));
        assert!(!node.is_tag("span"));
    }

    #[test]
    fn test_document() {
        let doc = Document::new();
        assert!(doc.is_empty());
        assert_eq!(doc.len(), 0);
    }

    #[test]
    fn test_document_with_children() {
        let mut doc = Document::new();
        doc.children.push(Node::element(
            NodeId(1),
            "html",
            "http://www.w3.org/1999/xhtml",
            vec![],
        ));
        assert!(!doc.is_empty());
        assert_eq!(doc.len(), 1);
        assert!(doc.root_element().is_some());
    }

    #[test]
    fn test_document_find_tag() {
        let mut doc = Document::new();
        let mut html = Node::element(NodeId(0), "html", "http://www.w3.org/1999/xhtml", vec![]);
        html.children.push(Node::element(
            NodeId(1),
            "body",
            "http://www.w3.org/1999/xhtml",
            vec![],
        ));
        doc.children.push(html);
        assert!(doc.find_tag("body").is_some());
        assert!(doc.find_tag("footer").is_none());
    }

    #[test]
    fn test_node_id_ordering() {
        assert!(NodeId(0) < NodeId(1));
        assert!(NodeId(5) == NodeId(5));
    }

    #[test]
    fn test_node_id_generator() {
        let mut gen = NodeIdGenerator::new();
        assert_eq!(gen.next(), NodeId(0));
        assert_eq!(gen.next(), NodeId(1));
        assert_eq!(gen.next(), NodeId(2));
    }

    #[test]
    fn test_node_id_roof() {
        assert_eq!(NodeId::ROOT, NodeId(0));
    }

    #[test]
    fn test_get_attribute() {
        let data = ElementData {
            tag_name: "div".into(),
            namespace: "http://www.w3.org/1999/xhtml".into(),
            attributes: vec![
                Attribute::new("class", "main"),
                Attribute::new("id", "content"),
            ],
        };
        assert_eq!(data.get_attribute("class"), Some("main"));
        assert_eq!(data.get_attribute("id"), Some("content"));
        assert_eq!(data.get_attribute("style"), None);
    }

    #[test]
    fn test_text_data_whitespace() {
        let text = TextData {
            value: "   ".to_string(),
        };
        assert!(text.is_whitespace());
        let text = TextData {
            value: "hello".to_string(),
        };
        assert!(!text.is_whitespace());
    }

    #[test]
    fn test_fragment() {
        let frag = Fragment::new();
        assert!(frag.is_empty());

        let frag = Fragment::with_children(vec![Node::text(NodeId(0), "hello")]);
        assert!(!frag.is_empty());
    }

    #[test]
    fn test_document_root_element_head_body() {
        let mut doc = Document::new();
        let mut html = Node::element(NodeId(0), "html", "http://www.w3.org/1999/xhtml", vec![]);
        html.children.push(Node::element(
            NodeId(1),
            "head",
            "http://www.w3.org/1999/xhtml",
            vec![],
        ));
        html.children.push(Node::element(
            NodeId(2),
            "body",
            "http://www.w3.org/1999/xhtml",
            vec![],
        ));
        doc.children.push(html);

        assert!(doc.root_element().is_some());
        assert!(doc.head().is_some());
        assert!(doc.body().is_some());
    }

    #[test]
    fn test_document_text_content() {
        let mut doc = Document::new();
        let mut p = Node::element(NodeId(0), "p", "http://www.w3.org/1999/xhtml", vec![]);
        p.children.push(Node::text(NodeId(1), "Hello"));
        doc.children.push(p);
        assert_eq!(doc.text_content(), "Hello");
    }

    #[test]
    fn test_document_find_tags() {
        let mut doc = Document::new();
        let mut div = Node::element(NodeId(0), "div", "http://www.w3.org/1999/xhtml", vec![]);
        div.children.push(Node::element(
            NodeId(1),
            "span",
            "http://www.w3.org/1999/xhtml",
            vec![],
        ));
        div.children.push(Node::element(
            NodeId(2),
            "span",
            "http://www.w3.org/1999/xhtml",
            vec![],
        ));
        doc.children.push(div);
        assert_eq!(doc.find_tags("span").len(), 2);
    }

    #[test]
    fn test_element_data_get_attribute_nonexistent() {
        let data = ElementData {
            tag_name: "div".into(),
            namespace: "http://www.w3.org/1999/xhtml".into(),
            attributes: vec![],
        };
        assert_eq!(data.get_attribute("class"), None);
    }

    #[test]
    fn test_node_kind_variants() {
        assert!(matches!(
            Node::element(NodeId(0), "div", "", vec![]).kind,
            NodeKind::Element(_)
        ));
        assert!(matches!(
            Node::text(NodeId(0), "hello").kind,
            NodeKind::Text(_)
        ));
        assert!(matches!(
            Node::comment(NodeId(0), "hello").kind,
            NodeKind::Comment(_)
        ));
        assert!(matches!(
            Node::doctype(NodeId(0), "html", "", "").kind,
            NodeKind::Doctype(_)
        ));
        assert!(matches!(
            Node::processing_instruction(NodeId(0), "xml", "v=1").kind,
            NodeKind::ProcessingInstruction(_)
        ));
    }

    #[test]
    fn test_node_depth_in() {
        let node = Node::text(NodeId(0), "hello");
        assert_eq!(node.depth_in(&[NodeId(0), NodeId(1)]), 2);
    }
}
