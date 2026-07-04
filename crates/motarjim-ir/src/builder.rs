use crate::*;

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
            let computed_style = styles.get(&html_node.id).cloned().unwrap_or_default();

            let semantic = self.semantic_analyzer.infer(html_node);
            let layout = self.layout_inferrer.infer(&computed_style);
            let responsive = self.responsive_inferrer.infer(html_node, &computed_style);
            let accessibility = self.accessibility_inferrer.infer(html_node);

            for variant in &responsive {
                let hint_value = format!(
                    "breakpoint={variant:?}:{}",
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
