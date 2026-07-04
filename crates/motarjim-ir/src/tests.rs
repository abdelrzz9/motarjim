use super::*;

use super::*;
use motarjim_ast::Attribute;
use smallvec::SmallVec;

fn make_element_doc(id: u32, tag_name: &str, attrs: &[(&str, &str)]) -> Document {
    let mut doc = Document::new();
    let mut element = Element::new(tag_name);
    for (name, value) in attrs {
        element.attributes.push(Attribute::new(*name, *value));
    }
    doc.nodes.push(HtmlNode {
        id: NodeId(id),
        node_type: NodeType::Element,
        element: Some(element),
        value: None,
        children: SmallVec::new(),
        parent: None,
        depth: 0,
        document_type: None,
    });
    doc.root_id = NodeId(id);
    doc
}

fn make_nested_doc() -> Document {
    let mut doc = Document::new();
    doc.nodes.push(HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Element,
        element: Some(Element::new("div")),
        value: None,
        children: smallvec::smallvec![NodeId(1)],
        parent: None,
        depth: 0,
        document_type: None,
    });
    doc.nodes.push(HtmlNode {
        id: NodeId(1),
        node_type: NodeType::Element,
        element: Some(Element::new("button")),
        value: None,
        children: SmallVec::new(),
        parent: Some(NodeId(0)),
        depth: 1,
        document_type: None,
    });
    doc.root_id = NodeId(0);
    doc
}

fn single_style(id: u32, style: ComputedStyle) -> HashMap<NodeId, ComputedStyle> {
    let mut map = HashMap::new();
    map.insert(NodeId(id), style);
    map
}

fn style_map(doc: &Document) -> HashMap<NodeId, ComputedStyle> {
    doc.nodes
        .iter()
        .map(|n| (n.id, ComputedStyle::default()))
        .collect()
}

fn make_html_node(id: u32, node_type: NodeType, element: Option<Element>) -> HtmlNode {
    HtmlNode {
        id: NodeId(id),
        node_type,
        element,
        value: None,
        children: SmallVec::new(),
        parent: None,
        depth: 0,
        document_type: None,
    }
}

// SemanticAnalyzer tests

#[test]
fn test_semantic_simple_elements() {
    let analyzer = SemanticAnalyzer::new();
    let cases = [
        ("button", SemanticIr::Button),
        ("div", SemanticIr::Container),
        ("span", SemanticIr::Text),
        ("p", SemanticIr::Paragraph),
        ("img", SemanticIr::Image),
        ("nav", SemanticIr::Navigation),
        ("header", SemanticIr::Header),
        ("footer", SemanticIr::Footer),
        ("main", SemanticIr::Main),
        ("aside", SemanticIr::Aside),
        ("article", SemanticIr::Article),
        ("section", SemanticIr::Section),
        ("hr", SemanticIr::Divider),
        ("ul", SemanticIr::List),
        ("ol", SemanticIr::List),
        ("li", SemanticIr::ListItem),
        ("table", SemanticIr::Table),
        ("tr", SemanticIr::TableRow),
        ("td", SemanticIr::TableCell),
        ("th", SemanticIr::TableCell),
        ("form", SemanticIr::Form),
        ("select", SemanticIr::Select),
        ("textarea", SemanticIr::TextArea),
        ("svg", SemanticIr::Icon),
        ("dialog", SemanticIr::Dialog),
    ];
    for (tag, expected) in &cases {
        let node = make_html_node(0, NodeType::Element, Some(Element::new(*tag)));
        assert_eq!(analyzer.infer(&node), *expected, "mismatch for tag {tag}");
    }
}

#[test]
fn test_semantic_headings() {
    let analyzer = SemanticAnalyzer::new();
    for level in 1u32..=6 {
        let tag = format!("h{level}");
        let node = make_html_node(level, NodeType::Element, Some(Element::new(tag)));
        assert_eq!(analyzer.infer(&node), SemanticIr::Heading { level });
    }
}

#[test]
fn test_semantic_input_types() {
    let analyzer = SemanticAnalyzer::new();
    let cases: [(&str, SemanticIr); 7] = [
        ("text", SemanticIr::Input),
        ("checkbox", SemanticIr::Checkbox),
        ("radio", SemanticIr::Radio),
        ("submit", SemanticIr::Button),
        ("button", SemanticIr::Button),
        ("hidden", SemanticIr::Spacer),
        (
            "file",
            SemanticIr::Custom(SmolStr::new_inline("file_picker")),
        ),
    ];
    for (input_type, expected) in &cases {
        let mut element = Element::new("input");
        element.attributes.push(Attribute::new("type", *input_type));
        let node = make_html_node(0, NodeType::Element, Some(element));
        assert_eq!(
            analyzer.infer(&node),
            *expected,
            "mismatch for input type {input_type}"
        );
    }
}

#[test]
fn test_semantic_anchor() {
    let analyzer = SemanticAnalyzer::new();
    let mut element = Element::new("a");
    element
        .attributes
        .push(Attribute::new("href", "https://example.com"));
    let node = make_html_node(0, NodeType::Element, Some(element));
    assert_eq!(
        analyzer.infer(&node),
        SemanticIr::Custom(SmolStr::new_inline("link"))
    );

    let mut element2 = Element::new("a");
    element2.attributes.push(Attribute::new("role", "button"));
    let node2 = make_html_node(0, NodeType::Element, Some(element2));
    assert_eq!(analyzer.infer(&node2), SemanticIr::Button);
}

#[test]
fn test_semantic_text_nodes() {
    let analyzer = SemanticAnalyzer::new();
    let text = HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Text,
        element: None,
        value: Some("Hello".to_string()),
        children: SmallVec::new(),
        parent: None,
        depth: 0,
        document_type: None,
    };
    assert_eq!(analyzer.infer(&text), SemanticIr::Text);

    let empty = HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Text,
        element: None,
        value: Some("   ".to_string()),
        children: SmallVec::new(),
        parent: None,
        depth: 0,
        document_type: None,
    };
    assert_eq!(analyzer.infer(&empty), SemanticIr::Spacer);

    let comment = HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Comment,
        element: None,
        value: Some("comment".to_string()),
        children: SmallVec::new(),
        parent: None,
        depth: 0,
        document_type: None,
    };
    assert_eq!(analyzer.infer(&comment), SemanticIr::Spacer);

    let doctype = HtmlNode {
        id: NodeId(0),
        node_type: NodeType::DocumentType,
        element: None,
        value: None,
        children: SmallVec::new(),
        parent: None,
        depth: 0,
        document_type: None,
    };
    assert_eq!(analyzer.infer(&doctype), SemanticIr::Root);
}

#[test]
fn test_semantic_div_with_role() {
    let analyzer = SemanticAnalyzer::new();
    let mut element = Element::new("div");
    element.attributes.push(Attribute::new("role", "button"));
    let node = make_html_node(0, NodeType::Element, Some(element));
    assert_eq!(analyzer.infer(&node), SemanticIr::Button);
}

#[test]
fn test_semantic_heading_with_aria_level() {
    let analyzer = SemanticAnalyzer::new();
    let mut element = Element::new("div");
    element.attributes.push(Attribute::new("role", "heading"));
    element.attributes.push(Attribute::new("aria-level", "3"));
    let node = make_html_node(0, NodeType::Element, Some(element));
    assert_eq!(analyzer.infer(&node), SemanticIr::Heading { level: 3 });

    let mut element2 = Element::new("div");
    element2.attributes.push(Attribute::new("role", "heading"));
    let node2 = make_html_node(0, NodeType::Element, Some(element2));
    assert_eq!(analyzer.infer(&node2), SemanticIr::Heading { level: 1 });
}

#[test]
fn test_semantic_element_without_element() {
    let analyzer = SemanticAnalyzer::new();
    let node = make_html_node(0, NodeType::Element, None);
    assert_eq!(analyzer.infer(&node), SemanticIr::Container);

    let doc_node = make_html_node(0, NodeType::Document, None);
    assert_eq!(analyzer.infer(&doc_node), SemanticIr::Root);
}

// LayoutInferrer tests

#[test]
fn test_layout_basic() {
    let inferrer = LayoutInferrer::new();
    assert_eq!(inferrer.infer(&ComputedStyle::default()), LayoutIr::Stack);
}

#[test]
fn test_layout_flex() {
    let inferrer = LayoutInferrer::new();
    let cases: [(Option<FlexDirection>, LayoutIr); 4] = [
        (None, LayoutIr::FlexRow),
        (Some(FlexDirection::Row), LayoutIr::FlexRow),
        (Some(FlexDirection::RowReverse), LayoutIr::FlexRow),
        (Some(FlexDirection::Column), LayoutIr::FlexColumn),
    ];
    for (dir, expected) in &cases {
        let mut style = ComputedStyle::default();
        style.display = DisplayType::Flex;
        style.flex_direction = *dir;
        assert_eq!(
            inferrer.infer(&style),
            *expected,
            "mismatch for flex direction {dir:?}"
        );
    }
}

#[test]
fn test_layout_grid_none() {
    let inferrer = LayoutInferrer::new();
    let mut style = ComputedStyle::default();
    style.display = DisplayType::Grid;
    assert_eq!(inferrer.infer(&style), LayoutIr::Grid);

    let mut style2 = ComputedStyle::default();
    style2.display = DisplayType::None;
    assert_eq!(inferrer.infer(&style2), LayoutIr::None);
}

#[test]
fn test_layout_position() {
    let inferrer = LayoutInferrer::new();
    let cases = [
        (PositionType::Absolute, LayoutIr::Absolute),
        (PositionType::Fixed, LayoutIr::Fixed),
        (PositionType::Sticky, LayoutIr::Sticky),
    ];
    for (pos, expected) in &cases {
        let mut style = ComputedStyle::default();
        style.position = *pos;
        assert_eq!(
            inferrer.infer(&style),
            *expected,
            "mismatch for position {pos:?}"
        );
    }
}

#[test]
fn test_layout_inline_types() {
    let inferrer = LayoutInferrer::new();
    let mut style = ComputedStyle::default();
    style.display = DisplayType::Inline;
    assert_eq!(inferrer.infer(&style), LayoutIr::Inline);

    let mut style2 = ComputedStyle::default();
    style2.display = DisplayType::InlineBlock;
    assert_eq!(inferrer.infer(&style2), LayoutIr::InlineBlock);
}

#[test]
fn test_layout_overflow() {
    let inferrer = LayoutInferrer::new();
    for overflow in [Overflow::Scroll, Overflow::Auto] {
        let mut style = ComputedStyle::default();
        style.overflow = Some(overflow);
        assert_eq!(inferrer.infer(&style), LayoutIr::Scroll);
    }
    for overflow in [Overflow::Visible, Overflow::Hidden] {
        let mut style = ComputedStyle::default();
        style.overflow = Some(overflow);
        assert_eq!(inferrer.infer(&style), LayoutIr::Stack);
    }
}

#[test]
fn test_layout_flow_types() {
    let inferrer = LayoutInferrer::new();
    let mut style = ComputedStyle::default();
    style.display = DisplayType::Flow;
    assert_eq!(inferrer.infer(&style), LayoutIr::Flow);

    let mut style2 = ComputedStyle::default();
    style2.display = DisplayType::FlowRoot;
    assert_eq!(inferrer.infer(&style2), LayoutIr::Stack);
}

#[test]
fn test_layout_table_contents() {
    let inferrer = LayoutInferrer::new();
    let mut style = ComputedStyle::default();
    style.display = DisplayType::Table;
    assert_eq!(inferrer.infer(&style), LayoutIr::Table);

    let mut style2 = ComputedStyle::default();
    style2.display = DisplayType::Contents;
    assert_eq!(inferrer.infer(&style2), LayoutIr::None);
}

// AccessibilityInferrer tests

#[test]
fn test_accessibility_role() {
    let inferrer = AccessibilityInferrer::new();
    let mut element = Element::new("div");
    element.attributes.push(Attribute::new("role", "button"));
    let node = make_html_node(0, NodeType::Element, Some(element));
    assert_eq!(inferrer.infer(&node).role.as_deref(), Some("button"));
}

#[test]
fn test_accessibility_labels() {
    let inferrer = AccessibilityInferrer::new();

    let mut btn = Element::new("button");
    btn.attributes.push(Attribute::new("aria-label", "Submit"));
    assert_eq!(
        inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(btn)))
            .label
            .as_deref(),
        Some("Submit")
    );

    let mut img = Element::new("img");
    img.attributes.push(Attribute::new("alt", "A photo"));
    assert_eq!(
        inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(img)))
            .label
            .as_deref(),
        Some("A photo")
    );

    let mut input = Element::new("input");
    input
        .attributes
        .push(Attribute::new("placeholder", "Enter name"));
    assert_eq!(
        inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(input)))
            .label
            .as_deref(),
        Some("Enter name")
    );
}

#[test]
fn test_accessibility_hidden_focused() {
    let inferrer = AccessibilityInferrer::new();

    let mut el = Element::new("div");
    el.attributes.push(Attribute::new("aria-hidden", "true"));
    assert!(
        inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(el)))
            .hidden
    );

    let mut el2 = Element::new("div");
    el2.attributes.push(Attribute::new("aria-hidden", "false"));
    assert!(
        !inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(el2)))
            .hidden
    );

    let mut el3 = Element::new("input");
    el3.attributes.push(Attribute::new("autofocus", ""));
    assert!(
        inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(el3)))
            .focused
    );
}

#[test]
fn test_accessibility_inferred_role() {
    let inferrer = AccessibilityInferrer::new();
    let cases: [(&str, Option<&str>); 10] = [
        ("nav", Some("navigation")),
        ("header", Some("banner")),
        ("footer", Some("contentinfo")),
        ("main", Some("main")),
        ("aside", Some("complementary")),
        ("article", Some("article")),
        ("form", Some("form")),
        ("button", Some("button")),
        ("div", None),
        ("span", None),
    ];
    for (tag, expected) in &cases {
        let node = make_html_node(0, NodeType::Element, Some(Element::new(*tag)));
        assert_eq!(
            inferrer.infer(&node).role.as_deref(),
            *expected,
            "mismatch for tag {tag}"
        );
    }
}

#[test]
fn test_accessibility_aria_attributes() {
    let inferrer = AccessibilityInferrer::new();

    let mut el = Element::new("div");
    el.attributes.push(Attribute::new("tabindex", "0"));
    assert_eq!(
        inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(el)))
            .tab_index,
        Some(0)
    );

    let mut el2 = Element::new("button");
    el2.attributes.push(Attribute::new("aria-expanded", "true"));
    assert_eq!(
        inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(el2)))
            .aria_expanded,
        Some(true)
    );

    let mut el3 = Element::new("div");
    el3.attributes
        .push(Attribute::new("aria-controls", "panel-1"));
    assert_eq!(
        inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(el3)))
            .aria_controls
            .as_deref(),
        Some("panel-1")
    );

    let mut el4 = Element::new("div");
    el4.attributes.push(Attribute::new("aria-live", "polite"));
    assert_eq!(
        inferrer
            .infer(&make_html_node(0, NodeType::Element, Some(el4)))
            .aria_live
            .as_deref(),
        Some("polite")
    );
}

#[test]
fn test_accessibility_no_attributes() {
    let inferrer = AccessibilityInferrer::new();
    let node = make_html_node(0, NodeType::Element, Some(Element::new("div")));
    let info = inferrer.infer(&node);
    assert!(info.role.is_none() && info.label.is_none() && !info.hidden && !info.focused);
}

#[test]
fn test_accessibility_text_node() {
    let inferrer = AccessibilityInferrer::new();
    let node = HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Text,
        element: None,
        value: Some("hello".to_string()),
        children: SmallVec::new(),
        parent: None,
        depth: 0,
        document_type: None,
    };
    let info = inferrer.infer(&node);
    assert!(info.role.is_none() && info.label.is_none());
}

#[test]
fn test_accessibility_description() {
    let inferrer = AccessibilityInferrer::new();
    let mut element = Element::new("div");
    element
        .attributes
        .push(Attribute::new("aria-description", "A description"));
    let info = inferrer.infer(&make_html_node(0, NodeType::Element, Some(element)));
    assert_eq!(info.description.as_deref(), Some("A description"));
}

// ResponsiveInferrer tests

#[test]
fn test_responsive_inferrer() {
    let inferrer = ResponsiveInferrer::new();
    let node = make_html_node(0, NodeType::Element, Some(Element::new("div")));
    assert!(inferrer.infer(&node, &ComputedStyle::default()).is_empty());
}

// IrBuilder integration tests

#[test]
fn test_builder_basic() {
    let doc = make_element_doc(0, "div", &[]);
    let styles = style_map(&doc);
    let builder = IrBuilder::new();
    let diagnostics = DiagnosticBag::new();
    let tree = builder.build(&doc, &styles, &diagnostics);
    assert_eq!(tree.root_id, NodeId(0));
    assert_eq!(tree.nodes.len(), 1);
}

#[test]
fn test_builder_inference_integration() {
    let doc = make_element_doc(0, "button", &[]);
    let styles = style_map(&doc);
    let builder = IrBuilder::new();
    let diagnostics = DiagnosticBag::new();
    let tree = builder.build(&doc, &styles, &diagnostics);
    assert_eq!(tree.nodes[0].semantic, SemanticIr::Button);
}

#[test]
fn test_builder_layout_integration() {
    let doc = make_element_doc(0, "div", &[]);
    let mut style = ComputedStyle::default();
    style.display = DisplayType::Flex;
    style.flex_direction = Some(FlexDirection::Row);
    let styles = single_style(0, style);
    let builder = IrBuilder::new();
    let diagnostics = DiagnosticBag::new();
    let tree = builder.build(&doc, &styles, &diagnostics);
    assert_eq!(tree.nodes[0].layout, LayoutIr::FlexRow);
}

#[test]
fn test_builder_nested() {
    let doc = make_nested_doc();
    let styles = style_map(&doc);
    let builder = IrBuilder::new();
    let diagnostics = DiagnosticBag::new();
    let tree = builder.build(&doc, &styles, &diagnostics);
    assert_eq!(tree.nodes.len(), 2);
    assert_eq!(tree.nodes[0].semantic, SemanticIr::Container);
    assert_eq!(tree.nodes[1].semantic, SemanticIr::Button);
    assert_eq!(tree.nodes[0].children[0], NodeId(1));
    assert_eq!(tree.nodes[1].parent, Some(NodeId(0)));
}

#[test]
fn test_builder_text_node() {
    let mut doc = Document::new();
    doc.nodes.push(HtmlNode {
        id: NodeId(0),
        node_type: NodeType::Text,
        element: None,
        value: Some("Click me".to_string()),
        children: SmallVec::new(),
        parent: None,
        depth: 0,
        document_type: None,
    });
    doc.root_id = NodeId(0);
    let tree = IrBuilder::new().build(&doc, &style_map(&doc), &DiagnosticBag::new());
    assert_eq!(tree.nodes[0].semantic, SemanticIr::Text);
}

#[test]
fn test_builder_missing_styles() {
    let doc = make_element_doc(0, "div", &[]);
    let tree = IrBuilder::new().build(&doc, &HashMap::new(), &DiagnosticBag::new());
    assert_eq!(tree.nodes[0].computed_style, ComputedStyle::default());
}

#[test]
fn test_builder_target_hints() {
    let doc = make_element_doc(0, "button", &[]);
    let tree = IrBuilder::new().build(&doc, &style_map(&doc), &DiagnosticBag::new());
    assert!(tree
        .target_hints
        .iter()
        .any(|h| h.target.as_str() == "accessibility"));

    let doc2 = make_element_doc(0, "div", &[]);
    let tree2 = IrBuilder::new().build(&doc2, &style_map(&doc2), &DiagnosticBag::new());
    assert!(tree2
        .target_hints
        .iter()
        .all(|h| h.target.as_str() != "accessibility"));
}

#[test]
fn test_builder_target_ir() {
    let doc = make_element_doc(0, "button", &[]);
    let tree = IrBuilder::new().build(&doc, &style_map(&doc), &DiagnosticBag::new());
    assert_eq!(
        tree.nodes[0].target,
        TargetIr::Generic {
            platform: SmolStr::new_inline("default"),
            node: SmolStr::new_inline("Button"),
        }
    );
}

#[test]
fn test_builder_default() {
    let builder = IrBuilder::default();
    let doc = make_element_doc(0, "div", &[]);
    let tree = builder.build(&doc, &style_map(&doc), &DiagnosticBag::new());
    assert_eq!(tree.nodes.len(), 1);
}

#[test]
fn test_builder_flex_column_target() {
    let doc = make_element_doc(0, "div", &[]);
    let mut style = ComputedStyle::default();
    style.display = DisplayType::Flex;
    style.flex_direction = Some(FlexDirection::Column);
    let tree = IrBuilder::new().build(&doc, &single_style(0, style), &DiagnosticBag::new());
    assert_eq!(tree.nodes[0].layout, LayoutIr::FlexColumn);
    assert_eq!(
        tree.nodes[0].target,
        TargetIr::Generic {
            platform: SmolStr::new_inline("default"),
            node: SmolStr::new_inline("Column"),
        }
    );
}

#[test]
fn test_builder_alt_accessibility() {
    let doc = make_element_doc(0, "img", &[("alt", "Logo")]);
    let tree = IrBuilder::new().build(&doc, &style_map(&doc), &DiagnosticBag::new());
    assert!(tree
        .target_hints
        .iter()
        .any(|h| h.target.as_str() == "accessibility"));
}
