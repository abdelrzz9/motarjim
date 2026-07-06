use std::fmt::Write;

use motarjim_html::{Document, HtmlParser, Node, NodeId, NodeKind};

/// Helper: parse HTML and unwrap the document.
fn parse(src: &str) -> Document {
    HtmlParser::parse(src).expect("Expected successful parse")
}

/// Recursively collect all text from the tree.
fn collect_text<'a>(nodes: &'a [Node], results: &mut Vec<&'a str>) {
    for node in nodes {
        if let NodeKind::Text(t) = &node.kind {
            let trimmed = t.value.trim();
            if !trimmed.is_empty() {
                results.push(trimmed);
            }
        }
        collect_text(&node.children, results);
    }
}

/// Find all nodes that satisfy a predicate, recursively.
fn find_nodes<'a, F>(nodes: &'a [Node], predicate: &F) -> Vec<&'a Node>
where
    F: Fn(&Node) -> bool,
{
    let mut result = Vec::new();
    for node in nodes {
        if predicate(node) {
            result.push(node);
        }
        result.extend(find_nodes(&node.children, predicate));
    }
    result
}

/// Find the first node with the given tag name, recursively.
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

fn debug_html(src: &str) -> String {
    let doc = HtmlParser::parse(src).expect("parse should succeed");
    debug_document(&doc)
}

fn debug_document(doc: &Document) -> String {
    let mut output = String::new();
    for child in &doc.children {
        debug_node(child, 0, &mut output);
    }
    output
}

fn debug_node(node: &Node, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    match &node.kind {
        NodeKind::Element(el) => {
            let attrs: String = el
                .attributes
                .iter()
                .map(|a| format!(" {}=\"{}\"", a.name, a.value))
                .collect();
            writeln!(output, "{indent}<{} {}>", el.tag_name, attrs.trim()).unwrap();
            for child in &node.children {
                debug_node(child, depth + 1, output);
            }
            writeln!(output, "{indent}</{}>", el.tag_name).unwrap();
        }
        NodeKind::Text(t) => {
            let val = t.value.trim();
            if !val.is_empty() {
                writeln!(output, "{indent}{}", val).unwrap();
            }
        }
        NodeKind::Comment(c) => {
            writeln!(output, "{indent}<!-- {} -->", c.value).unwrap();
        }
        NodeKind::Doctype(d) => {
            writeln!(output, "{indent}<!DOCTYPE {}>", d.name).unwrap();
        }
        NodeKind::ProcessingInstruction(pi) => {
            writeln!(output, "{indent}<?{} {}?>", pi.target, pi.data).unwrap();
        }
    }
}

// ============================================================================
// Basic parsing tests
// ============================================================================

#[test]
fn test_empty_document() {
    let doc = parse("");
    assert!(!doc.children.is_empty());
    assert_eq!(doc.children[0].tag_name(), Some("html"));
}

#[test]
fn test_simple_element() {
    let doc = parse("<div></div>");
    let body = find_tag(&doc.children, "body").expect("should have body");
    let div = &body.children[0];
    assert_eq!(div.tag_name(), Some("div"));
    assert!(div.children.is_empty());
}

#[test]
fn test_nested_elements() {
    let doc = parse("<ul><li>text</li></ul>");
    let body = find_tag(&doc.children, "body").expect("should have body");
    let ul = &body.children[0];
    assert_eq!(ul.tag_name(), Some("ul"));
    assert_eq!(ul.children.len(), 1);

    let li = &ul.children[0];
    assert_eq!(li.tag_name(), Some("li"));

    let mut texts = Vec::new();
    collect_text(&doc.children, &mut texts);
    assert!(
        texts.iter().any(|t| *t == "text"),
        "should find 'text' node, found: {texts:?}"
    );
}

#[test]
fn test_self_closing_tags() {
    let doc = parse("<br><hr>");
    let tag_names: Vec<Option<&str>> = find_nodes(&doc.children, &|_| true)
        .iter()
        .map(|n| n.tag_name())
        .collect();
    assert!(tag_names.contains(&Some("br")));
    assert!(tag_names.contains(&Some("hr")));
}

#[test]
fn test_attributes() {
    let doc = parse("<div class='container' id='main' data-value='test'></div>");
    let div = find_tag(&doc.children, "div").expect("should find div");
    let el = div.as_element().expect("element expected");
    assert_eq!(el.tag_name.as_str(), "div");
    assert_eq!(el.attributes.len(), 3);
    assert_eq!(el.attributes[0].value.as_str(), "container");
    assert_eq!(el.attributes[1].value.as_str(), "main");
    assert_eq!(el.attributes[2].value.as_str(), "test");
}

#[test]
fn test_boolean_attributes() {
    let doc = parse("<input disabled checked>");
    let input = find_tag(&doc.children, "input").expect("should find input");
    let el = input.as_element().expect("element expected");
    assert!(el.attributes.iter().any(|a| a.name == "disabled"));
    assert!(el.attributes.iter().any(|a| a.name == "checked"));
}

#[test]
fn test_text_nodes() {
    let doc = parse("<p>Hello, world!</p>");
    let p = find_tag(&doc.children, "p").expect("should find p");
    assert_eq!(p.children.len(), 1);
    if let NodeKind::Text(t) = &p.children[0].kind {
        assert!(t.value.contains("Hello, world!"));
    } else {
        panic!("expected text child");
    }
}

#[test]
fn test_comments() {
    let doc = parse("<!-- a comment --><div>visible</div>");
    let comments: Vec<&Node> = find_nodes(&doc.children, &|n| matches!(n.kind, NodeKind::Comment(_)));
    assert!(!comments.is_empty(), "should have a comment node");
    if let NodeKind::Comment(c) = &comments[0].kind {
        assert!(c.value.contains("a comment"));
    }
}

#[test]
fn test_doctype() {
    let doc = parse("<!DOCTYPE html><html></html>");
    let doctypes: Vec<&Node> =
        find_nodes(&doc.children, &|n| matches!(n.kind, NodeKind::Doctype(_)));
    assert!(!doctypes.is_empty(), "should have a doctype node");
    if let NodeKind::Doctype(d) = &doctypes[0].kind {
        assert_eq!(d.name.as_str(), "html");
    }
}

#[test]
fn test_malformed_html() {
    let doc = parse("<div><span>unclosed");
    assert!(!doc.children.is_empty(), "should still produce a tree");
}

#[test]
fn test_unicode() {
    let doc = parse("<p>Hello, 世界! 🎉</p>");
    let p = find_tag(&doc.children, "p").expect("should find p");
    if let NodeKind::Text(t) = &p.children[0].kind {
        assert!(t.value.contains("世界"));
        assert!(t.value.contains("🎉"));
    }
}

#[test]
fn test_void_elements() {
    let doc = parse("<img src='test.png'><br><input type='text'>");
    let all_tags: Vec<&str> = find_nodes(&doc.children, &|_| true)
        .iter()
        .filter_map(|n| n.tag_name())
        .collect();
    assert!(all_tags.contains(&"img"));
    assert!(all_tags.contains(&"br"));
    assert!(all_tags.contains(&"input"));
}

#[test]
fn test_nested_deeply() {
    let doc = parse("<div><div><div><p>deep</p></div></div></div>");
    let mut texts = Vec::new();
    collect_text(&doc.children, &mut texts);
    assert!(
        texts.iter().any(|t| *t == "deep"),
        "should find deep text"
    );
}

#[test]
fn test_mixed_content() {
    let doc = parse("<p>Hello <b>world</b>!</p>");
    let p = find_tag(&doc.children, "p").expect("should find p");
    assert!(!p.children.is_empty());
    let has_bold = p.children.iter().any(|c| c.tag_name() == Some("b"));
    assert!(has_bold, "should contain <b> element");
}

// ============================================================================
// Document and node constructor tests
// ============================================================================

#[test]
fn test_parse_with_diagnostics() {
    let (result, _diagnostics) = HtmlParser::parse_with_diagnostics("<div>hello</div>");
    assert!(result.is_ok());
}

#[test]
fn test_document_default() {
    let doc = Document::default();
    assert!(doc.is_empty());
}

#[test]
fn test_node_constructors() {
    let el = Node::element(NodeId(0), "div", "http://www.w3.org/1999/xhtml", vec![]);
    assert_eq!(el.tag_name(), Some("div"));

    let text = Node::text(NodeId(1), "hello");
    assert_eq!(text.as_text(), Some("hello"));

    let comment = Node::comment(NodeId(2), "note");
    assert_eq!(comment.as_comment(), Some("note"));

    let doctype = Node::doctype(NodeId(3), "html", "", "");
    assert!(doctype.as_doctype().is_some());

    let pi = Node::processing_instruction(NodeId(4), "xml", "version='1.0'");
    assert!(pi.as_processing_instruction().is_some());
}

#[test]
fn test_document_find_api() {
    let doc = parse("<html><head></head><body><h1>Title</h1></body></html>");
    assert!(doc.root_element().is_some());
    assert!(doc.head().is_some());
    assert!(doc.body().is_some());
    assert_eq!(doc.find_tag("h1").map(|n| n.tag_name()), Some(Some("h1")));
    assert!(!doc.find_tags("h1").is_empty());
    assert!(doc.text_content().contains("Title"));
}

#[test]
fn test_fragment_parsing() {
    let frag = HtmlParser::parse_fragment("<p>Hello</p>").expect("fragment parse");
    assert!(!frag.is_empty());
}

// ============================================================================
// Snapshot tests
// ============================================================================

#[test]
fn test_snapshot_simple_div() {
    insta::assert_snapshot!(debug_html("<div>Hello</div>"));
}

#[test]
fn test_snapshot_nested_elements() {
    insta::assert_snapshot!(debug_html("<ul><li>Item 1</li><li>Item 2</li></ul>"));
}

#[test]
fn test_snapshot_self_closing() {
    insta::assert_snapshot!(debug_html("<br><hr><img src='test.png'>"));
}

#[test]
fn test_snapshot_with_attributes() {
    insta::assert_snapshot!(debug_html(
        "<div class='container' id='main' data-test='value'>Content</div>"
    ));
}

#[test]
fn test_snapshot_comment() {
    insta::assert_snapshot!(debug_html("<!-- comment --><p>text</p>"));
}

#[test]
fn test_snapshot_doctype() {
    insta::assert_snapshot!(debug_html("<!DOCTYPE html><html><body></body></html>"));
}

#[test]
fn test_snapshot_mixed_content() {
    insta::assert_snapshot!(debug_html("<p>Hello <b>world</b> and <i>everyone</i></p>"));
}

#[test]
fn test_snapshot_deeply_nested() {
    insta::assert_snapshot!(debug_html(
        "<div><div><div><ul><li><span>deep</span></li></ul></div></div></div>"
    ));
}

#[test]
fn test_snapshot_empty_document() {
    insta::assert_snapshot!(debug_html(""));
}

#[test]
fn test_snapshot_malformed_html() {
    insta::assert_snapshot!(debug_html("<div><span>missing end tags"));
}

#[test]
fn test_snapshot_unicode() {
    insta::assert_snapshot!(debug_html("<p>Hello 世界 🎉</p>"));
}

#[test]
fn test_snapshot_void_elements() {
    insta::assert_snapshot!(debug_html(
        "<div><img src='a.png'><br><input type='text' value='hello'></div>"
    ));
}

#[test]
fn test_snapshot_boolean_attributes() {
    insta::assert_snapshot!(debug_html("<input disabled readonly>"));
}

#[test]
fn test_snapshot_script_tag() {
    insta::assert_snapshot!(debug_html("<script>alert('hello');</script>"));
}

#[test]
fn test_snapshot_style_tag() {
    insta::assert_snapshot!(debug_html("<style>body { color: red; }</style>"));
}

#[test]
fn test_snapshot_many_attributes() {
    insta::assert_snapshot!(debug_html(
        "<div class='a' id='b' data-x='1' data-y='2' data-z='3' aria-label='test' role='button' tabindex='0'>content</div>"
    ));
}
