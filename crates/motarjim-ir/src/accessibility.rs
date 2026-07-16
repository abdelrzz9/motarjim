use crate::*;
use std::collections::HashMap;

/// Extracts accessibility metadata from HTML element attributes.
#[derive(Debug, Clone)]
pub struct AccessibilityInferrer {}

impl AccessibilityInferrer {
    /// Creates a new accessibility inferrer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Extracts accessibility information from an HTML node, resolving
    /// `aria-labelledby` references against the document's ID map.
    #[must_use]
    pub fn infer(
        &self,
        node: &HtmlNode,
        doc: &Document,
        id_map: &HashMap<String, NodeId>,
    ) -> AccessibilityInfo {
        let Some(ref element) = node.element else {
            return AccessibilityInfo {
                role: None,
                label: None,
                description: None,
                focused: false,
                hidden: false,
                tab_index: None,
                aria_level: None,
                aria_expanded: None,
                aria_controls: None,
                aria_live: None,
                aria_busy: false,
            };
        };

        let role = element.get_attribute("role").map(String::from);

        // Resolve aria-label first, then aria-labelledby as fallback
        let label = element
            .get_attribute("aria-label")
            .map(String::from)
            .or_else(|| {
                // Try to resolve aria-labelledby reference
                if let Some(labelledby) = element.get_attribute("aria-labelledby") {
                    if let Some(&target_id) = id_map.get(labelledby) {
                        if let Some(target_node) = doc.nodes.iter().find(|n| n.id == target_id) {
                            extract_text_content(target_node)
                        } else {
                            Some(labelledby.to_string())
                        }
                    } else {
                        Some(labelledby.to_string())
                    }
                } else {
                    None
                }
            });

        let description = element
            .get_attribute("aria-description")
            .or_else(|| element.get_attribute("aria-describedby"))
            .map(String::from);

        let hidden = element.get_attribute("aria-hidden") == Some("true");
        let tab_index = element
            .get_attribute("tabindex")
            .and_then(|v| v.parse::<i32>().ok());
        let aria_level = element
            .get_attribute("aria-level")
            .and_then(|v| v.parse::<u32>().ok());
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
            role,
            label,
            description,
            focused,
            hidden,
            tab_index,
            aria_level,
            aria_expanded,
            aria_controls,
            aria_live,
            aria_busy,
        }
    }
}

/// Extract text content from a node and its text children.
fn extract_text_content(node: &HtmlNode) -> Option<String> {
    if let Some(ref text) = node.value {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
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

    fn make_html_node(id: u32, tag_name: &str, attrs: &[(&str, &str)]) -> HtmlNode {
        let mut element = Element::new(tag_name);
        for (name, value) in attrs {
            element.attributes.push(Attribute::new(*name, *value));
        }
        HtmlNode {
            id: NodeId(id),
            node_type: NodeType::Element,
            element: Some(element),
            value: None,
            children: SmallVec::new(),
            parent: None,
            depth: 0,
            document_type: None,
        }
    }

    fn make_doc(nodes: Vec<HtmlNode>) -> Document {
        let mut doc = Document::new();
        let root_id = nodes.first().map(|n| n.id).unwrap_or(NodeId(0));
        doc.nodes = nodes;
        doc.root_id = root_id;
        doc
    }

    fn build_id_map(doc: &Document) -> HashMap<String, NodeId> {
        let mut map = HashMap::new();
        for node in &doc.nodes {
            if let Some(ref element) = node.element {
                if let Some(id) = element.get_attribute("id") {
                    map.insert(id.to_string(), node.id);
                }
            }
        }
        map
    }

    #[test]
    fn test_aria_labelledby_resolves() {
        let mut label_el = make_html_node(0, "span", &[("id", "name-label")]);
        label_el.value = Some("Full Name".to_string());
        let target = make_html_node(
            1,
            "input",
            &[("aria-labelledby", "name-label")],
        );
        let doc = make_doc(vec![label_el, target]);
        let id_map = build_id_map(&doc);

        let inferrer = AccessibilityInferrer::new();
        let info = inferrer.infer(&doc.nodes[1], &doc, &id_map);
        assert_eq!(info.label.as_deref(), Some("Full Name"));
    }

    #[test]
    fn test_aria_labelledby_unresolved() {
        let target = make_html_node(
            0,
            "input",
            &[("aria-labelledby", "nonexistent")],
        );
        let doc = make_doc(vec![target]);
        let id_map = build_id_map(&doc);

        let inferrer = AccessibilityInferrer::new();
        let info = inferrer.infer(&doc.nodes[0], &doc, &id_map);
        // Falls back to raw string when ID not found
        assert_eq!(info.label.as_deref(), Some("nonexistent"));
    }

    #[test]
    fn test_aria_label_takes_precedence() {
        let target = make_html_node(
            0,
            "button",
            &[("aria-label", "Submit"), ("aria-labelledby", "some-id")],
        );
        let doc = make_doc(vec![target]);
        let id_map = build_id_map(&doc);

        let inferrer = AccessibilityInferrer::new();
        let info = inferrer.infer(&doc.nodes[0], &doc, &id_map);
        // aria-label takes precedence over aria-labelledby
        assert_eq!(info.label.as_deref(), Some("Submit"));
    }

    #[test]
    fn test_text_node_no_element() {
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
        let doc = make_doc(vec![]);
        let id_map = HashMap::new();

        let inferrer = AccessibilityInferrer::new();
        let info = inferrer.infer(&node, &doc, &id_map);
        assert!(info.role.is_none() && info.label.is_none());
    }
}
