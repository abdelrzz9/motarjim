use smallvec::SmallVec;
use smol_str::SmolStr;

use motarjim_ast::{Document, DocumentTypeNode, Element, HtmlNode, NodeId, NodeType};
use motarjim_diag::codes;
use motarjim_diag::{Diagnostic, DiagnosticBag};
use motarjim_lexer::html::{HtmlTokenKind, HtmlTokenizer};
use motarjim_lexer::Token as LexerToken;

use crate::util;

/// Parser for HTML source text.
///
/// Produces a [`Document`] containing a flat arena of [`HtmlNode`]s.
///
/// # Errors
///
/// Returns a vector of [`Diagnostic`]s if parsing fails.
///
/// # Example
///
/// ```rust
/// use motarjim_parser::HtmlParser;
///
/// let mut parser = HtmlParser::new("<p>Hello</p>");
/// let doc = parser.parse();
/// assert!(doc.is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct HtmlParser<'a> {
    /// The raw source text being parsed.
    source: &'a str,
    /// Tokenized HTML tokens.
    tokens: Vec<LexerToken<HtmlTokenKind>>,
    /// Current position in the token stream.
    pos: usize,
    /// Collected diagnostics during parsing.
    diagnostics: DiagnosticBag,
    /// Auto-incrementing node ID counter.
    next_id: u32,
}

impl<'a> HtmlParser<'a> {
    /// Creates a new `HtmlParser` for the given source text.
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        let mut tokenizer = HtmlTokenizer::new(source);
        let tokens = tokenizer.tokenize();
        Self {
            source,
            tokens,
            pos: 0,
            diagnostics: DiagnosticBag::new(),
            next_id: 0,
        }
    }

    /// Parses the HTML source and returns a [`Document`].
    ///
    /// # Errors
    ///
    /// Returns a vector of [`Diagnostic`]s if parsing encounters errors.
    pub fn parse(&mut self) -> Result<Document, Vec<Diagnostic>> {
        let mut doc = Document::new();

        let root_id = self.alloc_node_id();
        let root = HtmlNode {
            id: root_id,
            node_type: NodeType::Document,
            element: None,
            value: None,
            children: SmallVec::new(),
            parent: None,
            depth: 0,
            document_type: None,
        };
        doc.nodes.push(root);
        doc.root_id = root_id;

        self.parse_children(&mut doc, root_id, 1);

        if self.diagnostics.has_errors() {
            Err(self.diagnostics.clone().into_diagnostics())
        } else {
            Ok(doc)
        }
    }

    /// Allocates a new unique node ID.
    const fn alloc_node_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        NodeId(id)
    }

    /// Peeks at the next HTML token without consuming it.
    fn peek_token(&self) -> Option<&LexerToken<HtmlTokenKind>> {
        self.tokens.get(self.pos)
    }

    /// Consumes and returns the next HTML token.
    fn consume_token(&mut self) -> Option<LexerToken<HtmlTokenKind>> {
        let token = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(token.clone())
    }

    /// Parses child nodes of a parent element.
    fn parse_children(&mut self, doc: &mut Document, parent_id: NodeId, depth: u32) {
        while let Some(t) = self.peek_token() {
            let token_kind = t.kind;

            match token_kind {
                HtmlTokenKind::Eof => break,
                HtmlTokenKind::CloseTagStart => break,
                HtmlTokenKind::OpenTagStart => {
                    self.parse_element(doc, parent_id, depth);
                }
                HtmlTokenKind::Text => {
                    let text_token = match self.consume_token() {
                        Some(t) => t,
                        None => break,
                    };
                    let text = text_token.raw.trim().to_string();
                    if text.is_empty() {
                        continue;
                    }
                    let node_id = self.alloc_node_id();
                    let text_node = HtmlNode {
                        id: node_id,
                        node_type: NodeType::Text,
                        element: None,
                        value: Some(text),
                        children: SmallVec::new(),
                        parent: Some(parent_id),
                        depth,
                        document_type: None,
                    };
                    doc.nodes.push(text_node);
                    if let Some(parent) = doc.node_mut(parent_id) {
                        parent.children.push(node_id);
                    }
                }
                HtmlTokenKind::Comment => {
                    let comment_token = match self.consume_token() {
                        Some(t) => t,
                        None => break,
                    };
                    let node_id = self.alloc_node_id();
                    let comment_node = HtmlNode {
                        id: node_id,
                        node_type: NodeType::Comment,
                        element: None,
                        value: Some(comment_token.raw),
                        children: SmallVec::new(),
                        parent: Some(parent_id),
                        depth,
                        document_type: None,
                    };
                    doc.nodes.push(comment_node);
                    if let Some(parent) = doc.node_mut(parent_id) {
                        parent.children.push(node_id);
                    }
                }
                HtmlTokenKind::MarkupDeclaration => {
                    let decl_token = match self.consume_token() {
                        Some(t) => t,
                        None => break,
                    };
                    // Raw is empty for markup declarations; check source text via span
                    let start = decl_token.span.start.offset as usize;
                    let end = decl_token.span.end.offset as usize;
                    let source_snippet = if end > start && end <= self.source.len() {
                        self.source[start..end].to_uppercase()
                    } else {
                        String::new()
                    };
                    if source_snippet.contains("DOCTYPE") {
                        let node_id = self.alloc_node_id();
                        let doctype_node = HtmlNode {
                            id: node_id,
                            node_type: NodeType::DocumentType,
                            element: None,
                            value: None,
                            children: SmallVec::new(),
                            parent: Some(parent_id),
                            depth,
                            document_type: Some(DocumentTypeNode {
                                name: SmolStr::from("html"),
                                public_id: SmolStr::from(""),
                                system_id: SmolStr::from(""),
                            }),
                        };
                        doc.nodes.push(doctype_node);
                        if let Some(parent) = doc.node_mut(parent_id) {
                            parent.children.push(node_id);
                        }
                    }
                }
                _ => {
                    self.consume_token();
                }
            }
        }
    }

    /// Parses a single HTML element and its children.
    fn parse_element(&mut self, doc: &mut Document, parent_id: NodeId, depth: u32) {
        let open_tag = match self.consume_token() {
            Some(t) => t,
            None => return,
        };

        let tag_name = self.extract_tag_name(&open_tag.raw);
        if tag_name.is_empty() {
            self.diagnostics
                .push_error(codes::PARSER_UNEXPECTED_TOKEN, "Empty tag name");
            return;
        }

        let tag_name_end = open_tag.span.end.offset as usize;

        // Fast path: if next token is TagEnd (no attributes, TagEnd is separate)
        let has_immediate_tag_end = self
            .peek_token()
            .is_some_and(|t| t.kind == HtmlTokenKind::TagEnd);

        let (attrs, is_self_closing, pending_text, has_attrs_in_token) =
            if has_immediate_tag_end {
                let tag_end = self.consume_token();
                let is_sc = tag_end.is_some_and(|t| t.raw.contains('/'));
                (SmallVec::new(), is_sc, None, false)
            } else {
                let (attr_str, is_sc, pt, _) = self.scan_tag_close(tag_name_end);
                let gt_offset = tag_name_end + util::find_tag_close_offset(&self.source[tag_name_end..]);
                self.skip_tokens_past(gt_offset);
                // The Text token that contained attributes and potentially content
                // starts before gt_offset; consume it to avoid duplicate text emission.
                if let Some(t) = self.peek_token() {
                    if t.kind == HtmlTokenKind::Text
                        && (t.span.start.offset as usize) < gt_offset
                    {
                        self.consume_token();
                    }
                }
                (
                    util::parse_attributes_from_str(&attr_str),
                    is_sc,
                    pt,
                    true,
                )
            };

        let node_id = self.alloc_node_id();
        let mut element = Element::new(&tag_name);
        element.attributes = attrs;

        for attr in &element.attributes {
            if attr.name.as_str() == "id" && !attr.value.is_empty() {
                element.id = Some(attr.value.clone());
            }
        }

        if let Some(class_attr) = element
            .attributes
            .iter()
            .find(|a| a.name.as_str() == "class")
        {
            for class_name in class_attr.value.split_whitespace() {
                if !class_name.is_empty() {
                    element.classes.push(SmolStr::from(class_name));
                }
            }
        }

        let node = HtmlNode {
            id: node_id,
            node_type: NodeType::Element,
            element: Some(element),
            value: None,
            children: SmallVec::new(),
            parent: Some(parent_id),
            depth,
            document_type: None,
        };
        doc.nodes.push(node);

        if let Some(parent) = doc.node_mut(parent_id) {
            parent.children.push(node_id);
        }

        let is_void = util::is_void_element(&tag_name);
        if is_self_closing || is_void {
            return;
        }

        // If attributes were embedded in a Text token, we already consumed that token
        // via skip_tokens_past, so use pending_text. Otherwise, let parse_children
        // pick up the Text token naturally.
        if has_attrs_in_token {
            if let Some(text) = pending_text {
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    let text_id = self.alloc_node_id();
                    let text_node = HtmlNode {
                        id: text_id,
                        node_type: NodeType::Text,
                        element: None,
                        value: Some(trimmed),
                        children: SmallVec::new(),
                        parent: Some(node_id),
                        depth: depth + 1,
                        document_type: None,
                    };
                    doc.nodes.push(text_node);
                    if let Some(n) = doc.node_mut(node_id) {
                        n.children.push(text_id);
                    }
                }
            }
        }

        self.parse_children(doc, node_id, depth + 1);

        // Expect close tag
        self.expect_close_tag(&tag_name);
    }

    /// Scans the source from `start_offset` to find the closing `>` of the tag,
    /// parsing the attribute section and determining if the tag is self-closing.
    fn scan_tag_close(&self, start_offset: usize) -> (String, bool, Option<String>, usize) {
        let remaining = &self.source[start_offset..];
        let mut actual_gt_pos: Option<usize> = None;
        let mut s_in_single = false;
        let mut s_in_double = false;
        for (i, c) in remaining.char_indices() {
            match c {
                '"' if !s_in_single => s_in_double = !s_in_double,
                '\'' if !s_in_double => s_in_single = !s_in_single,
                '>' if !s_in_single && !s_in_double => {
                    actual_gt_pos = Some(i);
                    break;
                }
                _ => {}
            }
        }

        let pos = match actual_gt_pos {
            Some(p) => p,
            None => {
                return (remaining.trim().to_string(), false, None, start_offset);
            }
        };

        let before_gt = &remaining[..pos];
        let trimmed = before_gt.trim_end();
        let is_self_closing = trimmed.ends_with('/');

        let attr_str = if is_self_closing {
            let without_slash = &trimmed[..trimmed.len() - 1];
            without_slash.trim_end().to_string()
        } else {
            trimmed.to_string()
        };

        // Content after '>' up to the next '<' or end of source
        let after_gt = &remaining[pos + 1..];
        let content_end = after_gt.find('<').unwrap_or(after_gt.len());
        let content = after_gt[..content_end].to_string();
        let pending_text = if content.is_empty() { None } else { Some(content) };

        let tag_close_offset = start_offset + pos + 1; // byte position after '>'

        (attr_str, is_self_closing, pending_text, tag_close_offset)
    }

    /// Advances the token position past all tokens that end before or at `offset`.
    fn skip_tokens_past(&mut self, offset: usize) {
        while let Some(token) = self.tokens.get(self.pos) {
            if (token.span.end.offset as usize) > offset {
                break;
            }
            self.pos += 1;
        }
    }

    /// Extracts the tag name from a raw HTML tag string.
    fn extract_tag_name(&self, raw: &str) -> String {
        util::extract_tag_name(raw).to_string()
    }

    /// Expects and consumes a closing tag matching `expected_name`.
    fn expect_close_tag(&mut self, expected_name: &str) {
        loop {
            let token = if let Some(t) = self.peek_token() { t.clone() } else {
                self.diagnostics
                    .push_error(codes::PARSER_UNCLOSED_TAG, expected_name);
                return;
            };

            match token.kind {
                HtmlTokenKind::CloseTagStart => {
                    let close_name = self.extract_tag_name(&token.raw);
                    self.consume_token();

                    if close_name != expected_name {
                        self.diagnostics.push_error(
                            codes::PARSER_UNEXPECTED_TOKEN,
                            format!(
                                "Mismatched close tag: expected </{expected_name}>, found </{close_name}>"
                            ),
                        );
                    }

                    // Consume the trailing TagEnd if present
                    if let Some(next) = self.peek_token() {
                        if next.kind == HtmlTokenKind::TagEnd {
                            self.consume_token();
                        }
                    }
                    return;
                }
                HtmlTokenKind::Text => {
                    self.consume_token();
                }
                HtmlTokenKind::Comment => {
                    self.consume_token();
                }
                _ => break,
            }
        }

        self.diagnostics
            .push_error(codes::PARSER_UNCLOSED_TAG, expected_name);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_parse_empty() {
        let mut parser = HtmlParser::new("");
        let result = parser.parse();
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(doc.root().node_type, NodeType::Document);
    }

    #[test]
    fn test_parse_simple_element() {
        let mut parser = HtmlParser::new("<div></div>");
        let doc = parser.parse().expect("Failed to parse");
        assert_eq!(doc.nodes.len(), 2);

        let root = doc.root();
        assert_eq!(root.children.len(), 1);
        let child_id = root.children[0];
        let child = doc.node(child_id).expect("Child node not found");
        assert_eq!(child.node_type, NodeType::Element);
        assert_eq!(
            child.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("div")
        );
    }

    #[test]
    fn test_parse_nested_elements() {
        let mut parser = HtmlParser::new("<ul><li>text</li></ul>");
        let doc = parser.parse().expect("Failed to parse");
        assert!(doc.nodes.len() >= 3);

        let root = doc.root();
        assert_eq!(root.children.len(), 1);
        let ul_id = root.children[0];
        let ul = doc.node(ul_id).expect("UL not found");
        assert_eq!(
            ul.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("ul")
        );
        assert_eq!(ul.children.len(), 1);

        let li_id = ul.children[0];
        let li = doc.node(li_id).expect("LI not found");
        assert_eq!(
            li.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("li")
        );

        assert!(!li.children.is_empty());
        let text_id = li.children[0];
        let text = doc.node(text_id).expect("Text not found");
        assert_eq!(text.node_type, NodeType::Text);
        assert_eq!(text.value.as_deref(), Some("text"));
    }

    #[test]
    fn test_parse_attributes() {
        let mut parser = HtmlParser::new("<div class=\"container\" id=\"main\"></div>");
        let doc = parser.parse().expect("Failed to parse");
        let root = doc.root();
        let div_id = root.children[0];
        let div = doc.node(div_id).expect("Div not found");
        let el = div.element.as_ref().expect("Element not found");

        assert_eq!(el.tag_name.as_str(), "div");
        assert_eq!(el.attributes.len(), 2);
        assert_eq!(el.get_attribute("class"), Some("container"));
        assert_eq!(el.get_attribute("id"), Some("main"));
        assert_eq!(el.id.as_deref(), Some("main"));
        assert!(el.has_class("container"));
    }

    #[test]
    fn test_parse_self_closing_tag() {
        let mut parser = HtmlParser::new("<br/><hr/>");
        let doc = parser.parse().expect("Failed to parse");
        let root = doc.root();
        assert_eq!(root.children.len(), 2);

        let br = doc.node(root.children[0]).expect("BR not found");
        assert_eq!(
            br.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("br")
        );
        assert!(br.children.is_empty());

        let hr = doc.node(root.children[1]).expect("HR not found");
        assert_eq!(
            hr.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("hr")
        );
        assert!(hr.children.is_empty());
    }

    #[test]
    fn test_parse_text_content() {
        let mut parser = HtmlParser::new("<p>Hello, world!</p>");
        let doc = parser.parse().expect("Failed to parse");
        let root = doc.root();
        let p_id = root.children[0];
        let p = doc.node(p_id).expect("P not found");
        assert_eq!(p.children.len(), 1);
        let text_id = p.children[0];
        let text = doc.node(text_id).expect("Text not found");
        assert_eq!(text.node_type, NodeType::Text);
        assert_eq!(text.value.as_deref(), Some("Hello, world!"));
    }

    #[test]
    fn test_parse_comment() {
        let mut parser = HtmlParser::new("<!-- comment -->");
        let doc = parser.parse().expect("Failed to parse");
        let root = doc.root();
        assert_eq!(root.children.len(), 1);
        let comment = doc.node(root.children[0]).expect("Comment not found");
        assert_eq!(comment.node_type, NodeType::Comment);
    }

    #[test]
    fn test_parse_doctype() {
        let mut parser = HtmlParser::new("<!DOCTYPE html><html></html>");
        let doc = parser.parse().expect("Failed to parse");
        let root = doc.root();
        assert_eq!(root.children.len(), 2);
        let doctype = doc.node(root.children[0]).expect("Doctype not found");
        assert_eq!(doctype.node_type, NodeType::DocumentType);
        let html = doc.node(root.children[1]).expect("HTML not found");
        assert_eq!(
            html.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("html")
        );
    }

    #[test]
    fn test_parse_multiple_classes() {
        let mut parser = HtmlParser::new("<span class=\"foo bar baz\"></span>");
        let doc = parser.parse().expect("Failed to parse");
        let span = doc.node(doc.root().children[0]).expect("Span not found");
        let el = span.element.as_ref().expect("Element not found");
        assert_eq!(el.classes.len(), 3);
        assert!(el.has_class("foo"));
        assert!(el.has_class("bar"));
        assert!(el.has_class("baz"));
    }

    #[test]
    fn test_parse_void_elements_no_close() {
        let mut parser = HtmlParser::new("<img src=\"test.png\"><br><input type=\"text\">");
        let doc = parser.parse().expect("Failed to parse");
        let root = doc.root();
        assert_eq!(root.children.len(), 3);

        let img = doc.node(root.children[0]).expect("IMG not found");
        assert_eq!(
            img.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("img")
        );
        assert_eq!(
            img.element.as_ref().and_then(|e| e.get_attribute("src")),
            Some("test.png")
        );

        assert_eq!(
            doc.node(root.children[1])
                .and_then(|n| n.element.as_ref().map(|e| e.tag_name.as_str())),
            Some("br")
        );

        assert_eq!(
            doc.node(root.children[2])
                .and_then(|n| n.element.as_ref().map(|e| e.tag_name.as_str())),
            Some("input")
        );
    }

    #[test]
    fn test_parse_boolean_attribute() {
        let mut parser = HtmlParser::new("<input disabled>");
        let doc = parser.parse().expect("Failed to parse");
        let input = doc.node(doc.root().children[0]).expect("Input not found");
        let el = input.element.as_ref().expect("Element not found");
        assert_eq!(el.get_attribute("disabled"), Some(""));
    }

    #[test]
    fn test_html_parse_error_recovery() {
        let mut parser = HtmlParser::new("<div><span></div>");
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_mixed_html_content() {
        let mut parser = HtmlParser::new(
            "<section><h1>Title</h1><p>Paragraph with <strong>bold</strong> text</p></section>",
        );
        let doc = parser.parse().expect("Failed to parse");
        let root = doc.root();
        let section = doc.node(root.children[0]).expect("Section not found");
        assert_eq!(
            section.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("section")
        );
        assert_eq!(section.children.len(), 2);

        let h1 = doc.node(section.children[0]).expect("H1 not found");
        assert_eq!(
            h1.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("h1")
        );

        if let Some(t) = h1.children.first() {
            let text = doc.node(*t).expect("Text in h1");
            assert_eq!(text.value.as_deref(), Some("Title"));
        }

        let p = doc.node(section.children[1]).expect("P not found");
        assert_eq!(
            p.element.as_ref().map(|e| e.tag_name.as_str()),
            Some("p")
        );
    }
}
