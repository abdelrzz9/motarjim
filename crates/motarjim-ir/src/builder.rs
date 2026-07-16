use crate::*;
use motarjim_ast_ir::{BreakpointRange, TextDirection};
use std::collections::HashMap;

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
    ///
    /// The `breakpoints` parameter contains the breakpoint ranges extracted from
    /// `@media` rules by the CSS resolver's `collect_breakpoints()` method.
    pub fn build(
        &self,
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        diagnostics: &mut DiagnosticBag,
        breakpoints: &[BreakpointRange],
    ) -> IrTree {
        let mut ir_nodes: Vec<IrNode> = Vec::with_capacity(doc.nodes.len());
        let mut target_hints: Vec<TargetHint> = Vec::new();

        // Build ID → NodeId map for aria-labelledby resolution
        let id_map: HashMap<String, NodeId> = {
            let mut map = HashMap::new();
            for node in &doc.nodes {
                if let Some(ref element) = node.element {
                    if let Some(id) = element.get_attribute("id") {
                        map.insert(id.to_string(), node.id);
                    }
                }
            }
            map
        };

        for html_node in &doc.nodes {
            let computed_style = styles.get(&html_node.id).cloned().unwrap_or_default();

            let semantic = self.semantic_analyzer.infer(html_node);
            let layout = self.layout_inferrer.infer(&computed_style);
            let responsive = self.responsive_inferrer.infer(html_node, &computed_style, breakpoints);
            let accessibility = self.accessibility_inferrer.infer(html_node, doc, &id_map);

            // Check for unresolvable aria-labelledby references
            if let Some(ref element) = html_node.element {
                if let Some(labelledby) = element.get_attribute("aria-labelledby") {
                    if !id_map.contains_key(labelledby) {
                        diagnostics.push_warning(
                            motarjim_diag::codes::IR_ARIA_LABELLEDBY_UNRESOLVED,
                            format!(
                                "aria-labelledby references '{labelledby}' but no element with this id exists"
                            ),
                        );
                    }
                }
            }

            for variant in &responsive {
                let hint_value = format!(
                    "breakpoint={}:{}",
                    variant.breakpoint.classify(),
                    variant
                        .style_override
                        .iter()
                        .map(|(k, v)| format!("{k}:{v}"))
                        .collect::<Vec<_>>()
                        .join(",")
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

            // Emit diagnostic for unknown/custom semantic roles
            if let SemanticIr::Custom(ref name) = semantic {
                diagnostics.push_warning(
                    motarjim_diag::codes::A11Y_MISSING_ALT,
                    format!("Unknown semantic role '{name}' for element"),
                );
            }

            // Emit diagnostic for invalid heading levels (HTML only defines h1-h6)
            if let SemanticIr::Heading { level } = semantic {
                if level > 6 {
                    diagnostics.push_warning(
                        motarjim_diag::codes::IR_HEADING_LEVEL_INVALID,
                        format!("Heading level {level} is invalid; HTML defines h1-h6 only"),
                    );
                }
            }

            // Detect text direction from dir attribute
            let text_direction = if let Some(ref element) = html_node.element {
                if let Some(dir) = element.get_attribute("dir") {
                    match dir {
                        "ltr" => Some(TextDirection::Ltr),
                        "rtl" => Some(TextDirection::Rtl),
                        "auto" => Some(TextDirection::Auto),
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            };

            ir_nodes.push(IrNode {
                id: html_node.id,
                target: Self::infer_target(&semantic, &layout),
                semantic,
                layout,
                computed_style,
                children: html_node.children.clone(),
                parent: html_node.parent,
                text: html_node.value.clone(),
                responsive,
                events: Vec::new(),
                text_direction,
            });
        }

        // Post-processing: promote tree-aware layout variants (ZStack, LazyList)
        self.layout_inferrer.promote_tree_aware(&mut ir_nodes, &doc.nodes);

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
