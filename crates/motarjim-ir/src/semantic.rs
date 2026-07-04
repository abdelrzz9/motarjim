use crate::*;

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
            "strong" | "b" | "em" | "i" | "u" | "s" | "mark" | "small" | "sub" | "sup" | "ins"
            | "del" | "code" | "kbd" | "samp" | "var" | "cite" | "abbr" | "dfn" | "time"
            | "data" | "q" | "bdo" | "bdi" | "ruby" | "rt" | "rp" | "wbr" => SemanticIr::Text,
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
                    let level: u32 = element
                        .get_attribute("aria-level")
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
            "text" | "search" | "email" | "url" | "tel" | "password" | "number" | "date"
            | "datetime-local" | "month" | "week" | "time" => SemanticIr::Input,
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
