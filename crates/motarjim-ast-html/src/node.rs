//! HTML AST node types for the Motarjim compiler.

use smallvec::SmallVec;
use smol_str::SmolStr;

/// A unique identifier for a node in the AST arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId(pub u32);

/// The type of an HTML node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeType {
    /// The root document node.
    Document,
    /// An HTML element node (has a tag name and attributes).
    Element,
    /// A text content node.
    Text,
    /// An HTML comment node.
    Comment,
    /// A DOCTYPE declaration node.
    DocumentType,
    /// A document fragment node.
    Fragment,
}

/// An HTML attribute (name-value pair).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Attribute {
    /// The attribute name.
    pub name: SmolStr,
    /// The attribute value.
    pub value: SmolStr,
}

impl Attribute {
    /// Creates a new attribute.
    #[must_use]
    pub fn new(name: impl Into<SmolStr>, value: impl Into<SmolStr>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// An HTML element node with tag name, attributes, and metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Element {
    /// The tag name.
    pub tag_name: SmolStr,
    /// The element's attributes.
    pub attributes: SmallVec<[Attribute; 8]>,
    /// The element's `id` attribute value, if present.
    pub id: Option<SmolStr>,
    /// The element's class names.
    pub classes: SmallVec<[SmolStr; 4]>,
    /// The namespace URI.
    pub namespace: SmolStr,
}

impl Element {
    /// Creates a new element with the given tag name.
    #[must_use]
    pub fn new(tag_name: impl Into<SmolStr>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes: SmallVec::new(),
            id: None,
            classes: SmallVec::new(),
            namespace: SmolStr::from("http://www.w3.org/1999/xhtml"),
        }
    }

    /// Returns `true` if the element has the given class name.
    #[must_use]
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c.as_str() == class)
    }

    /// Returns the attribute value, or `None`.
    #[must_use]
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attr| attr.name.as_str() == name)
            .map(|attr| attr.value.as_str())
    }

    /// Returns `true` if this is a void (self-closing) HTML element.
    #[must_use]
    pub fn is_void_element(&self) -> bool {
        matches!(
            self.tag_name.as_str(),
            "area"
                | "base"
                | "br"
                | "col"
                | "embed"
                | "hr"
                | "img"
                | "input"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
    }

    /// Returns `true` if this is a block-level HTML element.
    #[must_use]
    pub fn is_block_element(&self) -> bool {
        matches!(
            self.tag_name.as_str(),
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "details"
                | "dialog"
                | "dd"
                | "div"
                | "dl"
                | "dt"
                | "fieldset"
                | "figcaption"
                | "figure"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "hgroup"
                | "hr"
                | "li"
                | "main"
                | "nav"
                | "ol"
                | "p"
                | "pre"
                | "section"
                | "table"
                | "ul"
        )
    }

    /// Returns `true` if this is an inline-level HTML element.
    #[must_use]
    pub fn is_inline_element(&self) -> bool {
        matches!(
            self.tag_name.as_str(),
            "a" | "abbr"
                | "area"
                | "b"
                | "bdi"
                | "bdo"
                | "br"
                | "button"
                | "cite"
                | "code"
                | "data"
                | "del"
                | "dfn"
                | "em"
                | "i"
                | "img"
                | "input"
                | "ins"
                | "kbd"
                | "label"
                | "link"
                | "map"
                | "mark"
                | "meter"
                | "output"
                | "progress"
                | "q"
                | "ruby"
                | "s"
                | "samp"
                | "select"
                | "small"
                | "source"
                | "span"
                | "strong"
                | "sub"
                | "sup"
                | "template"
                | "textarea"
                | "time"
                | "u"
                | "var"
                | "video"
                | "wbr"
        )
    }
}

/// An HTML document type declaration node.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentTypeNode {
    /// The document type name.
    pub name: SmolStr,
    /// The public identifier.
    pub public_id: SmolStr,
    /// The system identifier.
    pub system_id: SmolStr,
}

/// A single HTML node in the AST arena.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct HtmlNode {
    /// The unique node identifier.
    pub id: NodeId,
    /// The type of this node.
    pub node_type: NodeType,
    /// The element data, if this is an Element node.
    pub element: Option<Element>,
    /// The text value, if this is a Text or Comment node.
    pub value: Option<String>,
    /// The IDs of this node's children.
    pub children: SmallVec<[NodeId; 4]>,
    /// The ID of this node's parent, if any.
    pub parent: Option<NodeId>,
    /// The depth of this node in the tree (0 for root).
    pub depth: u32,
    /// The document type data, if this is a `DocumentType` node.
    pub document_type: Option<DocumentTypeNode>,
}

/// The root document, containing all nodes in a flat arena.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Document {
    /// All nodes in the document arena.
    pub nodes: Vec<HtmlNode>,
    /// The ID of the root node.
    pub root_id: NodeId,
}

impl Document {
    /// Creates a new empty document.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root_id: NodeId(0),
        }
    }

    /// Returns a reference to the node with the given ID.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&HtmlNode> {
        self.nodes.get(id.0 as usize)
    }

    /// Returns a mutable reference to the node with the given ID.
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut HtmlNode> {
        self.nodes.get_mut(id.0 as usize)
    }

    /// Returns a reference to the root node.
    ///
    /// # Panics
    ///
    /// Panics if the document is empty or the root ID is invalid.
    #[must_use]
    pub fn root(&self) -> &HtmlNode {
        &self.nodes[self.root_id.0 as usize]
    }

    /// Returns references to all child nodes of the node with the given ID.
    #[must_use]
    pub fn children(&self, id: NodeId) -> Vec<&HtmlNode> {
        let Some(node) = self.nodes.get(id.0 as usize) else {
            return Vec::new();
        };
        node.children
            .iter()
            .filter_map(|child_id| self.nodes.get(child_id.0 as usize))
            .collect()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// A styled node: an HTML node paired with its resolved computed style.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct StyledNode {
    /// The underlying HTML node.
    pub node: HtmlNode,
    /// The resolved computed style for this node.
    pub computed_style: crate::style::ComputedStyle,
}

/// A styled document: all nodes with their resolved styles.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct StyledDocument {
    /// All styled nodes in the document.
    pub nodes: Vec<StyledNode>,
    /// The ID of the root node.
    pub root_id: NodeId,
}

/// A document with semantic role annotations.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticDocument {
    /// All nodes in the document.
    pub nodes: Vec<HtmlNode>,
    /// The ID of the root node.
    pub root_id: NodeId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute() {
        let attr = Attribute::new("class", "container");
        assert_eq!(attr.name.as_str(), "class");
        assert_eq!(attr.value.as_str(), "container");
        let attr = Attribute::new("id", "main");
        assert_eq!(attr.name.as_str(), "id");
    }

    #[test]
    fn test_element() {
        let mut el = Element::new("div");
        assert_eq!(el.tag_name.as_str(), "div");
        assert!(el.attributes.is_empty() && el.id.is_none() && el.classes.is_empty());
        assert_eq!(el.namespace.as_str(), "http://www.w3.org/1999/xhtml");

        el.classes.push(SmolStr::new_inline("container"));
        assert!(el.has_class("container") && !el.has_class("hidden"));

        el.attributes
            .push(Attribute::new("href", "https://example.com"));
        assert_eq!(el.get_attribute("href"), Some("https://example.com"));
        assert_eq!(el.get_attribute("rel"), None);

        assert!(Element::new("br").is_void_element() && !Element::new("div").is_void_element());
        assert!(Element::new("div").is_block_element() && !Element::new("span").is_block_element());
        assert!(
            Element::new("span").is_inline_element() && !Element::new("div").is_inline_element()
        );
    }

    #[test]
    fn test_htmlnode() {
        let node = HtmlNode {
            id: NodeId(1),
            node_type: NodeType::Element,
            element: Some(Element::new("div")),
            value: None,
            children: SmallVec::new(),
            parent: None,
            depth: 0,
            document_type: None,
        };
        assert_eq!(node.id, NodeId(1));
        assert_eq!(node.node_type, NodeType::Element);

        let text = HtmlNode {
            id: NodeId(2),
            node_type: NodeType::Text,
            element: None,
            value: Some("Hello".to_string()),
            children: SmallVec::new(),
            parent: None,
            depth: 1,
            document_type: None,
        };
        assert_eq!(text.node_type, NodeType::Text);
        assert_eq!(text.value.as_deref(), Some("Hello"));
    }

    #[test]
    fn test_document() {
        let doc = Document::new();
        assert!(doc.nodes.is_empty() && doc.root_id == NodeId(0));
        assert!(NodeId(0) < NodeId(1) && NodeId(42) == NodeId(42));
        assert!(doc.children(NodeId(42)).is_empty());
    }

    #[test]
    fn test_document_root_and_children() {
        let mut doc = Document::new();
        doc.nodes.push(HtmlNode {
            id: NodeId(0),
            node_type: NodeType::Element,
            element: Some(Element::new("root")),
            value: None,
            children: smallvec::smallvec![NodeId(1), NodeId(2)],
            parent: None,
            depth: 0,
            document_type: None,
        });
        doc.nodes.push(HtmlNode {
            id: NodeId(1),
            node_type: NodeType::Element,
            element: Some(Element::new("child")),
            value: None,
            children: SmallVec::new(),
            parent: Some(NodeId(0)),
            depth: 1,
            document_type: None,
        });
        doc.nodes.push(HtmlNode {
            id: NodeId(2),
            node_type: NodeType::Text,
            element: None,
            value: Some("hello".to_string()),
            children: SmallVec::new(),
            parent: Some(NodeId(0)),
            depth: 1,
            document_type: None,
        });
        doc.root_id = NodeId(0);

        assert!(doc.node(NodeId(0)).is_some() && doc.node(NodeId(99)).is_none());
        assert_eq!(doc.root().id, NodeId(0));

        if let Some(r) = doc.node_mut(NodeId(0)) {
            if let Some(ref mut el) = r.element {
                el.tag_name = SmolStr::new_inline("updated");
            }
        }
        assert_eq!(
            doc.node(NodeId(0))
                .unwrap()
                .element
                .as_ref()
                .unwrap()
                .tag_name
                .as_str(),
            "updated"
        );

        let children = doc.children(NodeId(0));
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].id, NodeId(1));
    }
}
