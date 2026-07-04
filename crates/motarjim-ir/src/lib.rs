//! IR builder crate for the Motarjim compiler.
//!
//! Builds the platform-independent Intermediate Representation (IR) tree
//! from parsed HTML documents and computed CSS styles. The IR tree is the
//! central data structure consumed by all code generators.
//!
//! The builder orchestrates four inference passes:
//! - **Semantic inference** ([`SemanticAnalyzer`]): Maps HTML tag names and
//!   attributes to [`SemanticIr`] roles.
//! - **Layout inference** ([`LayoutInferrer`]): Converts CSS computed styles into [`LayoutIr`] strategies.
//! - **Responsive inference** ([`ResponsiveInferrer`]): Extracts responsive breakpoint information.
//! - **Accessibility inference** ([`AccessibilityInferrer`]): Extracts ARIA attributes and implicit roles.

#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

use std::collections::HashMap;

use motarjim_ast::{Document, Element, HtmlNode, NodeId, NodeType};
use motarjim_ast::ir::{HintType, IrNode, IrTree, LayoutIr, SemanticIr, TargetHint, TargetIr};
use motarjim_ast::layout::ResponsiveVariant;
use motarjim_ast::semantic::AccessibilityInfo;
use motarjim_ast::style::{
    ComputedStyle, DisplayType, FlexDirection, Overflow, PositionType,
};
use motarjim_diag::DiagnosticBag;
use smol_str::SmolStr;

/// The main IR builder that converts a parsed HTML [`Document`] into an [`IrTree`].
#[derive(Debug, Clone)]
pub struct IrBuilder {
    /// Analyzer for semantic role inference.
    semantic_analyzer: SemanticAnalyzer,
    /// Inferrer for layout strategy detection.
    layout_inferrer: LayoutInferrer,
    /// Inferrer for responsive variant attachment.
    responsive_inferrer: ResponsiveInferrer,
    /// Inferrer for accessibility metadata extraction.
    accessibility_inferrer: AccessibilityInferrer,
}

impl IrBuilder {
    /// Creates a new `IrBuilder` with default inference engines.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            semantic_analyzer: SemanticAnalyzer::new(),
            layout_inferrer: LayoutInferrer::new(),
            responsive_inferrer: ResponsiveInferrer::new(),
            accessibility_inferrer: AccessibilityInferrer::new(),
        }
    }

    /// Builds an [`IrTree`] from the given parsed document and computed styles.
    #[must_use]
    pub fn build(
        &self,
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        _diagnostics: &DiagnosticBag,
    ) -> IrTree {
        let mut ir_nodes: Vec<IrNode> = Vec::with_capacity(doc.nodes.len());
        let mut target_hints: Vec<TargetHint> = Vec::new();

        for html_node in &doc.nodes {
            let computed_style = styles
                .get(&html_node.id)
                .cloned()
                .unwrap_or_default();

            let semantic = self.semantic_analyzer.infer(html_node);
            let layout = self.layout_inferrer.infer(&computed_style);
            let responsive = self.responsive_inferrer.infer(html_node, &computed_style);
            let accessibility = self.accessibility_inferrer.infer(html_node);

            for variant in &responsive {
                let hint_value = format!(
                    "breakpoint={variant:?}:{}",
                    variant.style_override.iter()
                        .map(|(k, v)| format!("{k}:{v}"))
                        .collect::<Vec<_>>().join(",")
                );
                target_hints.push(TargetHint {
                    target: SmolStr::new_inline("responsive"),
                    hint_type: HintType::Property,
                    value: hint_value,
                });
            }

            if accessibility.label.is_some() || accessibility.role.is_some() {
                let a11y_val = format!(
                    "label={},role={}",
                    accessibility.label.as_deref().unwrap_or(""),
                    accessibility.role.as_deref().unwrap_or(""),
                );
                target_hints.push(TargetHint {
                    target: SmolStr::new_inline("accessibility"),
                    hint_type: HintType::Property,
                    value: a11y_val,
                });
            }

            ir_nodes.push(IrNode {
                id: html_node.id,
                target: Self::infer_target(&semantic, &layout),
                semantic,
                layout,
                computed_style,
                children: html_node.children.clone(),
                parent: html_node.parent,
            });
        }

        IrTree {
            nodes: ir_nodes,
            root_id: doc.root_id,
            target_hints,
        }
    }

    /// Infers the target-specific IR for a node based on its semantic role and layout.
    #[must_use]
    fn infer_target(semantic: &SemanticIr, layout: &LayoutIr) -> TargetIr {
        let widget_name: &str = match semantic {
            SemanticIr::Root => "Root",
            SemanticIr::Navigation => "NavigationView",
            SemanticIr::NavigationBar => "AppBar",
            SemanticIr::HeroSection => "HeroSection",
            SemanticIr::Card => "Card",
            SemanticIr::Button => "Button",
            SemanticIr::Text => "Text",
            SemanticIr::Heading { .. } => "Heading",
            SemanticIr::Paragraph => "Paragraph",
            SemanticIr::Image => "Image",
            SemanticIr::Icon => "Icon",
            SemanticIr::Input => "TextField",
            SemanticIr::TextArea => "TextArea",
            SemanticIr::Select => "Dropdown",
            SemanticIr::Checkbox => "Checkbox",
            SemanticIr::Radio => "RadioButton",
            SemanticIr::Form => "Form",
            SemanticIr::List => "List",
            SemanticIr::ListItem => "ListItem",
            SemanticIr::Table => "Table",
            SemanticIr::TableRow => "TableRow",
            SemanticIr::TableCell => "TableCell",
            SemanticIr::Section => "Section",
            SemanticIr::Article => "Article",
            SemanticIr::Aside => "Sidebar",
            SemanticIr::Footer => "Footer",
            SemanticIr::Header => "Header",
            SemanticIr::Main => "MainContent",
            SemanticIr::Dialog => "Dialog",
            SemanticIr::Tooltip => "Tooltip",
            SemanticIr::Badge => "Badge",
            SemanticIr::Divider => "Divider",
            SemanticIr::Spacer => "Spacer",
            SemanticIr::Container => match layout {
                LayoutIr::FlexRow => "Row",
                LayoutIr::FlexColumn => "Column",
                LayoutIr::Grid => "Grid",
                LayoutIr::Scroll => "ScrollView",
                _ => "Container",
            },
            SemanticIr::Grid => "Grid",
            SemanticIr::Flex => "Flex",
            SemanticIr::Column => "Column",
            SemanticIr::Row => "Row",
            SemanticIr::Stack => "Stack",
            SemanticIr::Scroll => "ScrollView",
            SemanticIr::LazyList => "LazyList",
            SemanticIr::IconButton => "IconButton",
            SemanticIr::Chip => "Chip",
            SemanticIr::Avatar => "Avatar",
            SemanticIr::Progress => "ProgressIndicator",
            SemanticIr::Skeleton => "Skeleton",
            SemanticIr::Custom(name) => name.as_str(),
        };

        TargetIr::Generic {
            platform: SmolStr::new_inline("default"),
            node: SmolStr::from(widget_name),
        }
    }
}

impl Default for IrBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Performs semantic role inference on HTML nodes.
#[derive(Debug, Clone)]
pub struct SemanticAnalyzer {}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Infers the semantic role of the given HTML node.
    #[must_use]
    pub fn infer(&self, node: &HtmlNode) -> SemanticIr {
        match node.node_type {
            NodeType::Document | NodeType::Fragment => SemanticIr::Root,
            NodeType::Text => {
                let text = node.value.as_deref().unwrap_or("");
                if text.trim().is_empty() {
                    SemanticIr::Spacer
                } else {
                    SemanticIr::Text
                }
            }
            NodeType::Comment => SemanticIr::Spacer,
            NodeType::DocumentType => SemanticIr::Root,
            NodeType::Element => {
                let Some(ref element) = node.element else {
                    return SemanticIr::Container;
                };
                Self::infer_from_element(element)
            }
        }
    }

    /// Infer semantic role from an HTML element.
    fn infer_from_element(element: &Element) -> SemanticIr {
        match element.tag_name.as_str() {
            "div" | "figure" => Self::infer_div_like(element),
            "span" => SemanticIr::Text,
            "p" => SemanticIr::Paragraph,
            "h1" => SemanticIr::Heading { level: 1 },
            "h2" => SemanticIr::Heading { level: 2 },
            "h3" => SemanticIr::Heading { level: 3 },
            "h4" => SemanticIr::Heading { level: 4 },
            "h5" => SemanticIr::Heading { level: 5 },
            "h6" => SemanticIr::Heading { level: 6 },
            "a" => {
                if element.get_attribute("role") == Some("button") {
                    SemanticIr::Button
                } else {
                    SemanticIr::Custom(SmolStr::new_inline("link"))
                }
            }
            "button" => SemanticIr::Button,
            "input" => Self::infer_input(element),
            "textarea" => SemanticIr::TextArea,
            "select" => SemanticIr::Select,
            "label" => SemanticIr::Text,
            "form" => SemanticIr::Form,
            "ul" | "ol" | "dl" => SemanticIr::List,
            "li" | "dt" | "dd" => SemanticIr::ListItem,
            "table" => SemanticIr::Table,
            "tr" => SemanticIr::TableRow,
            "td" | "th" => SemanticIr::TableCell,
            "caption" => SemanticIr::Text,
            "thead" | "tbody" | "tfoot" => SemanticIr::Section,
            "img" => SemanticIr::Image,
            "video" => SemanticIr::Custom(SmolStr::new_inline("video")),
            "audio" => SemanticIr::Custom(SmolStr::new_inline("audio")),
            "canvas" => SemanticIr::Custom(SmolStr::new_inline("canvas")),
            "svg" => SemanticIr::Icon,
            "picture" => SemanticIr::Image,
            "source" => SemanticIr::Spacer,
            "nav" => SemanticIr::Navigation,
            "header" => SemanticIr::Header,
            "footer" => SemanticIr::Footer,
            "main" => SemanticIr::Main,
            "aside" => SemanticIr::Aside,
            "article" => SemanticIr::Article,
            "section" => SemanticIr::Section,
            "strong" | "b" | "em" | "i" | "u" | "s" | "mark" | "small" | "sub"
            | "sup" | "ins" | "del" | "code" | "kbd" | "samp" | "var" | "cite"
            | "abbr" | "dfn" | "time" | "data" | "q" | "bdo" | "bdi" | "ruby"
            | "rt" | "rp" | "wbr" => SemanticIr::Text,
            "hr" => SemanticIr::Divider,
            "br" => SemanticIr::Spacer,
            "dialog" => SemanticIr::Dialog,
            "details" => SemanticIr::Container,
            "summary" => SemanticIr::Text,
            "template" | "noscript" | "iframe" | "embed" | "object" => SemanticIr::Container,
            "param" => SemanticIr::Spacer,
            "fieldset" => SemanticIr::Form,
            "legend" | "output" => SemanticIr::Text,
            "optgroup" | "datalist" => SemanticIr::Select,
            "option" => SemanticIr::Custom(SmolStr::new_inline("option")),
            "progress" | "meter" => SemanticIr::Progress,
            "figcaption" => SemanticIr::Text,
            _ => SemanticIr::Container,
        }
    }

    /// Infer semantic role from a div-like (or figure) element using attributes.
    fn infer_div_like(element: &Element) -> SemanticIr {
        if let Some(role) = element.get_attribute("role") {
            return match role {
                "button" => SemanticIr::Button,
                "navigation" => SemanticIr::Navigation,
                "banner" => SemanticIr::Header,
                "contentinfo" => SemanticIr::Footer,
                "main" => SemanticIr::Main,
                "complementary" => SemanticIr::Aside,
                "form" | "search" => SemanticIr::Form,
                "dialog" | "alertdialog" => SemanticIr::Dialog,
                "tooltip" => SemanticIr::Tooltip,
                "img" => SemanticIr::Image,
                "list" => SemanticIr::List,
                "listitem" => SemanticIr::ListItem,
                "tab" => SemanticIr::Button,
                "tabpanel" => SemanticIr::Container,
                "grid" => SemanticIr::Grid,
                "progressbar" => SemanticIr::Progress,
                "slider" => SemanticIr::Custom(SmolStr::new_inline("slider")),
                "switch" => SemanticIr::Checkbox,
                "heading" => {
                    let level: u32 = element.get_attribute("aria-level")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(1);
                    SemanticIr::Heading { level }
                }
                "article" => SemanticIr::Article,
                "region" => SemanticIr::Section,
                "alert" | "status" | "timer" => SemanticIr::Badge,
                "marquee" => SemanticIr::Custom(SmolStr::new_inline("marquee")),
                _ => SemanticIr::Container,
            };
        }
        SemanticIr::Container
    }

    /// Infer semantic role from an `<input>` element using its type attribute.
    fn infer_input(element: &Element) -> SemanticIr {
        let input_type = element.get_attribute("type").unwrap_or("text");
        match input_type {
            "text" | "search" | "email" | "url" | "tel" | "password" | "number"
            | "date" | "datetime-local" | "month" | "week" | "time" => SemanticIr::Input,
            "checkbox" => SemanticIr::Checkbox,
            "radio" => SemanticIr::Radio,
            "button" | "submit" | "reset" | "image" => SemanticIr::Button,
            "file" => SemanticIr::Custom(SmolStr::new_inline("file_picker")),
            "color" => SemanticIr::Custom(SmolStr::new_inline("color_picker")),
            "range" => SemanticIr::Custom(SmolStr::new_inline("slider")),
            "hidden" => SemanticIr::Spacer,
            _ => SemanticIr::Input,
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Infers layout strategies from CSS computed styles.
#[derive(Debug, Clone)]
pub struct LayoutInferrer {}

impl LayoutInferrer {
    /// Creates a new layout inferrer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Infers the layout strategy from a node's computed style.
    #[must_use]
    pub fn infer(&self, style: &ComputedStyle) -> LayoutIr {
        if style.display == DisplayType::None {
            return LayoutIr::None;
        }

        match style.position {
            PositionType::Absolute => return LayoutIr::Absolute,
            PositionType::Fixed => return LayoutIr::Fixed,
            PositionType::Sticky => return LayoutIr::Sticky,
            PositionType::Relative | PositionType::Static => {}
        }

        if style.display == DisplayType::Flex {
            return match style.flex_direction {
                Some(FlexDirection::Row) | Some(FlexDirection::RowReverse) => LayoutIr::FlexRow,
                Some(FlexDirection::Column) | Some(FlexDirection::ColumnReverse) => LayoutIr::FlexColumn,
                None => LayoutIr::FlexRow,
            };
        }

        if style.display == DisplayType::Grid {
            return LayoutIr::Grid;
        }

        if let Some(ref overflow) = style.overflow {
            if *overflow == Overflow::Scroll || *overflow == Overflow::Auto {
                return LayoutIr::Scroll;
            }
        }

        match style.display {
            DisplayType::Block | DisplayType::FlowRoot => LayoutIr::Stack,
            DisplayType::Flow => LayoutIr::Flow,
            DisplayType::Inline => LayoutIr::Inline,
            DisplayType::InlineBlock => LayoutIr::InlineBlock,
            DisplayType::Table | DisplayType::TableRow | DisplayType::TableCell => LayoutIr::Table,
            DisplayType::ListItem => LayoutIr::Stack,
            DisplayType::Contents => LayoutIr::None,
            DisplayType::Flex | DisplayType::Grid | DisplayType::None => LayoutIr::None,
        }
    }
}

impl Default for LayoutInferrer {
    fn default() -> Self {
        Self::new()
    }
}

/// Infers responsive breakpoint information from styles.
#[derive(Debug, Clone)]
pub struct ResponsiveInferrer {}

impl ResponsiveInferrer {
    /// Creates a new responsive inferrer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Infers responsive variants for a node.
    #[must_use]
    pub const fn infer(
        &self,
        _node: &HtmlNode,
        _style: &ComputedStyle,
    ) -> Vec<ResponsiveVariant> {
        Vec::new()
    }
}

impl Default for ResponsiveInferrer {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracts accessibility metadata from HTML element attributes.
#[derive(Debug, Clone)]
pub struct AccessibilityInferrer {}

impl AccessibilityInferrer {
    /// Creates a new accessibility inferrer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Extracts accessibility information from an HTML node.
    #[must_use]
    pub fn infer(&self, node: &HtmlNode) -> AccessibilityInfo {
        let Some(ref element) = node.element else {
            return AccessibilityInfo {
                role: None, label: None, description: None, focused: false,
                hidden: false, tab_index: None, aria_level: None,
                aria_expanded: None, aria_controls: None, aria_live: None,
                aria_busy: false,
            };
        };

        let role = element.get_attribute("role").map(String::from);

        let label = element.get_attribute("aria-label")
            .or_else(|| element.get_attribute("aria-labelledby"))
            .map(String::from);

        let description = element.get_attribute("aria-description")
            .or_else(|| element.get_attribute("aria-describedby"))
            .map(String::from);

        let hidden = element.get_attribute("aria-hidden") == Some("true");
        let tab_index = element.get_attribute("tabindex").and_then(|v| v.parse::<i32>().ok());
        let aria_level = element.get_attribute("aria-level").and_then(|v| v.parse::<u32>().ok());
        let aria_expanded = element.get_attribute("aria-expanded").map(|v| v == "true");
        let aria_controls = element.get_attribute("aria-controls").map(String::from);
        let aria_live = element.get_attribute("aria-live").map(String::from);
        let aria_busy = element.get_attribute("aria-busy") == Some("true");
        let focused = element.get_attribute("autofocus").is_some();

        let role = role.or_else(|| {
            let tag_role: Option<&str> = match element.tag_name.as_str() {
                "nav" => Some("navigation"),
                "header" => Some("banner"),
                "footer" => Some("contentinfo"),
                "main" => Some("main"),
                "aside" => Some("complementary"),
                "article" => Some("article"),
                "form" | "search" => Some("form"),
                "dialog" => Some("dialog"),
                "button" => Some("button"),
                "a" if element.get_attribute("href").is_some() => Some("link"),
                "img" => Some("img"),
                "table" => Some("table"),
                "progress" => Some("progressbar"),
                "input" => Some("textbox"),
                "textarea" => Some("textbox"),
                "select" => Some("combobox"),
                "ul" | "ol" => Some("list"),
                "li" => Some("listitem"),
                _ => None,
            };
            tag_role.map(String::from)
        });

        let label = label.or_else(|| element.get_attribute("placeholder").map(String::from));

        let label = label.or_else(|| {
            if element.tag_name.as_str() == "img" {
                element.get_attribute("alt").map(String::from)
            } else {
                None
            }
        });

        AccessibilityInfo {
            role, label, description, focused, hidden, tab_index,
            aria_level, aria_expanded, aria_controls, aria_live, aria_busy,
        }
    }
}

impl Default for AccessibilityInferrer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
            id: NodeId(id), node_type: NodeType::Element,
            element: Some(element), value: None,
            children: SmallVec::new(), parent: None, depth: 0, document_type: None,
        });
        doc.root_id = NodeId(id);
        doc
    }

    fn make_nested_doc() -> Document {
        let mut doc = Document::new();
        doc.nodes.push(HtmlNode {
            id: NodeId(0), node_type: NodeType::Element,
            element: Some(Element::new("div")), value: None,
            children: smallvec::smallvec![NodeId(1)],
            parent: None, depth: 0, document_type: None,
        });
        doc.nodes.push(HtmlNode {
            id: NodeId(1), node_type: NodeType::Element,
            element: Some(Element::new("button")), value: None,
            children: SmallVec::new(), parent: Some(NodeId(0)),
            depth: 1, document_type: None,
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
        doc.nodes.iter().map(|n| (n.id, ComputedStyle::default())).collect()
    }

    fn make_html_node(id: u32, node_type: NodeType, element: Option<Element>) -> HtmlNode {
        HtmlNode {
            id: NodeId(id), node_type, element, value: None,
            children: SmallVec::new(), parent: None, depth: 0, document_type: None,
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
            ("file", SemanticIr::Custom(SmolStr::new_inline("file_picker"))),
        ];
        for (input_type, expected) in &cases {
            let mut element = Element::new("input");
            element.attributes.push(Attribute::new("type", *input_type));
            let node = make_html_node(0, NodeType::Element, Some(element));
            assert_eq!(analyzer.infer(&node), *expected, "mismatch for input type {input_type}");
        }
    }

    #[test]
    fn test_semantic_anchor() {
        let analyzer = SemanticAnalyzer::new();
        let mut element = Element::new("a");
        element.attributes.push(Attribute::new("href", "https://example.com"));
        let node = make_html_node(0, NodeType::Element, Some(element));
        assert_eq!(analyzer.infer(&node), SemanticIr::Custom(SmolStr::new_inline("link")));

        let mut element2 = Element::new("a");
        element2.attributes.push(Attribute::new("role", "button"));
        let node2 = make_html_node(0, NodeType::Element, Some(element2));
        assert_eq!(analyzer.infer(&node2), SemanticIr::Button);
    }

    #[test]
    fn test_semantic_text_nodes() {
        let analyzer = SemanticAnalyzer::new();
        let text = HtmlNode {
            id: NodeId(0), node_type: NodeType::Text, element: None,
            value: Some("Hello".to_string()),
            children: SmallVec::new(), parent: None, depth: 0, document_type: None,
        };
        assert_eq!(analyzer.infer(&text), SemanticIr::Text);

        let empty = HtmlNode {
            id: NodeId(0), node_type: NodeType::Text, element: None,
            value: Some("   ".to_string()),
            children: SmallVec::new(), parent: None, depth: 0, document_type: None,
        };
        assert_eq!(analyzer.infer(&empty), SemanticIr::Spacer);

        let comment = HtmlNode {
            id: NodeId(0), node_type: NodeType::Comment, element: None,
            value: Some("comment".to_string()),
            children: SmallVec::new(), parent: None, depth: 0, document_type: None,
        };
        assert_eq!(analyzer.infer(&comment), SemanticIr::Spacer);

        let doctype = HtmlNode {
            id: NodeId(0), node_type: NodeType::DocumentType, element: None, value: None,
            children: SmallVec::new(), parent: None, depth: 0, document_type: None,
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
            assert_eq!(inferrer.infer(&style), *expected, "mismatch for flex direction {dir:?}");
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
            assert_eq!(inferrer.infer(&style), *expected, "mismatch for position {pos:?}");
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
        assert_eq!(inferrer.infer(&make_html_node(0, NodeType::Element, Some(btn))).label.as_deref(), Some("Submit"));

        let mut img = Element::new("img");
        img.attributes.push(Attribute::new("alt", "A photo"));
        assert_eq!(inferrer.infer(&make_html_node(0, NodeType::Element, Some(img))).label.as_deref(), Some("A photo"));

        let mut input = Element::new("input");
        input.attributes.push(Attribute::new("placeholder", "Enter name"));
        assert_eq!(inferrer.infer(&make_html_node(0, NodeType::Element, Some(input))).label.as_deref(), Some("Enter name"));
    }

    #[test]
    fn test_accessibility_hidden_focused() {
        let inferrer = AccessibilityInferrer::new();

        let mut el = Element::new("div");
        el.attributes.push(Attribute::new("aria-hidden", "true"));
        assert!(inferrer.infer(&make_html_node(0, NodeType::Element, Some(el))).hidden);

        let mut el2 = Element::new("div");
        el2.attributes.push(Attribute::new("aria-hidden", "false"));
        assert!(!inferrer.infer(&make_html_node(0, NodeType::Element, Some(el2))).hidden);

        let mut el3 = Element::new("input");
        el3.attributes.push(Attribute::new("autofocus", ""));
        assert!(inferrer.infer(&make_html_node(0, NodeType::Element, Some(el3))).focused);
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
            assert_eq!(inferrer.infer(&node).role.as_deref(), *expected, "mismatch for tag {tag}");
        }
    }

    #[test]
    fn test_accessibility_aria_attributes() {
        let inferrer = AccessibilityInferrer::new();

        let mut el = Element::new("div");
        el.attributes.push(Attribute::new("tabindex", "0"));
        assert_eq!(inferrer.infer(&make_html_node(0, NodeType::Element, Some(el))).tab_index, Some(0));

        let mut el2 = Element::new("button");
        el2.attributes.push(Attribute::new("aria-expanded", "true"));
        assert_eq!(inferrer.infer(&make_html_node(0, NodeType::Element, Some(el2))).aria_expanded, Some(true));

        let mut el3 = Element::new("div");
        el3.attributes.push(Attribute::new("aria-controls", "panel-1"));
        assert_eq!(inferrer.infer(&make_html_node(0, NodeType::Element, Some(el3))).aria_controls.as_deref(), Some("panel-1"));

        let mut el4 = Element::new("div");
        el4.attributes.push(Attribute::new("aria-live", "polite"));
        assert_eq!(inferrer.infer(&make_html_node(0, NodeType::Element, Some(el4))).aria_live.as_deref(), Some("polite"));
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
            id: NodeId(0), node_type: NodeType::Text, element: None,
            value: Some("hello".to_string()),
            children: SmallVec::new(), parent: None, depth: 0, document_type: None,
        };
        let info = inferrer.infer(&node);
        assert!(info.role.is_none() && info.label.is_none());
    }

    #[test]
    fn test_accessibility_description() {
        let inferrer = AccessibilityInferrer::new();
        let mut element = Element::new("div");
        element.attributes.push(Attribute::new("aria-description", "A description"));
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
            id: NodeId(0), node_type: NodeType::Text, element: None,
            value: Some("Click me".to_string()),
            children: SmallVec::new(), parent: None, depth: 0, document_type: None,
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
        assert!(tree.target_hints.iter().any(|h| h.target.as_str() == "accessibility"));

        let doc2 = make_element_doc(0, "div", &[]);
        let tree2 = IrBuilder::new().build(&doc2, &style_map(&doc2), &DiagnosticBag::new());
        assert!(tree2.target_hints.iter().all(|h| h.target.as_str() != "accessibility"));
    }

    #[test]
    fn test_builder_target_ir() {
        let doc = make_element_doc(0, "button", &[]);
        let tree = IrBuilder::new().build(&doc, &style_map(&doc), &DiagnosticBag::new());
        assert_eq!(tree.nodes[0].target, TargetIr::Generic {
            platform: SmolStr::new_inline("default"),
            node: SmolStr::new_inline("Button"),
        });
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
        assert_eq!(tree.nodes[0].target, TargetIr::Generic {
            platform: SmolStr::new_inline("default"),
            node: SmolStr::new_inline("Column"),
        });
    }

    #[test]
    fn test_builder_alt_accessibility() {
        let doc = make_element_doc(0, "img", &[("alt", "Logo")]);
        let tree = IrBuilder::new().build(&doc, &style_map(&doc), &DiagnosticBag::new());
        assert!(tree.target_hints.iter().any(|h| h.target.as_str() == "accessibility"));
    }
}
