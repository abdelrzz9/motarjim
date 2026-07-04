use crate::*;

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

        let label = element
            .get_attribute("aria-label")
            .or_else(|| element.get_attribute("aria-labelledby"))
            .map(String::from);

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

impl Default for AccessibilityInferrer {
    fn default() -> Self {
        Self::new()
    }
}
