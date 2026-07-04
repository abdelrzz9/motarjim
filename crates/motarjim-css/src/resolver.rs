use crate::matching::{rule_matches_element, rule_max_specificity};
use crate::*;

/// Accepts parsed stylesheets, matches selectors against DOM elements,
/// resolves the cascade, and computes final styles.
///
/// # Example
///
/// ```rust
/// use motarjim_ast::css::{CssStylesheet, StyleRule, Declaration};
/// use motarjim_ast::Element;
/// use motarjim_css::StyleResolver;
///
/// let mut resolver = StyleResolver::new();
/// resolver.add_stylesheet(CssStylesheet {
///     rules: vec![],
///     source_path: None,
/// });
/// let el = Element::new("div");
/// let values = resolver.resolve(&el);
/// ```
pub struct StyleResolver {
    /// Loaded stylesheets to resolve against.
    stylesheets: Vec<CssStylesheet>,
}

impl StyleResolver {
    /// Create a new empty style resolver.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stylesheets: Vec::new(),
        }
    }

    /// Add a parsed stylesheet to the resolver.
    pub fn add_stylesheet(&mut self, sheet: CssStylesheet) {
        self.stylesheets.push(sheet);
    }

    /// Return a reference to the registered stylesheets.
    #[must_use]
    pub fn stylesheets(&self) -> &[CssStylesheet] {
        &self.stylesheets
    }

    /// Resolve the computed style for a single element.
    ///
    /// Uses the registered stylesheets, matching selectors and resolving the
    /// cascade. If a `parent` value is provided, inheritable properties from
    /// the parent are used as the starting point.
    #[must_use]
    pub fn resolve_with_parent(
        &self,
        element: &Element,
        parent: Option<&ComputedValues>,
    ) -> ComputedValues {
        let mut cascade = Cascade::new();

        for sheet in &self.stylesheets {
            self.collect_matching_declarations(&mut cascade, sheet, element);
        }

        let resolved_map = cascade.resolve();
        ComputedValues::from_map(&resolved_map, parent)
    }

    /// Resolve the computed style for a single element without a parent context.
    #[must_use]
    pub fn resolve(&self, element: &Element) -> ComputedValues {
        self.resolve_with_parent(element, None)
    }

    /// Resolve computed styles for multiple elements in parallel (uses rayon).
    ///
    /// Each element is resolved independently (no parent-child relationships
    /// are assumed — use [`resolve_with_parent`](Self::resolve_with_parent)
    /// for individual elements when parent style is needed for inheritance).
    #[must_use]
    pub fn resolve_parallel(&self, elements: &[Element]) -> Vec<ComputedValues> {
        use rayon::prelude::*;
        elements
            .par_iter()
            .map(|element| self.resolve(element))
            .collect()
    }

    /// Collect declarations from all rules in a stylesheet that match an element.
    fn collect_matching_declarations(
        &self,
        cascade: &mut Cascade,
        sheet: &CssStylesheet,
        element: &Element,
    ) {
        for rule in &sheet.rules {
            self.collect_from_rule(cascade, rule, element);
        }
    }

    /// Collect declarations from a single rule (or nested rules inside at-rules).
    fn collect_from_rule(&self, cascade: &mut Cascade, rule: &CssRule, element: &Element) {
        match rule {
            CssRule::Style(style_rule) => {
                if rule_matches_element(style_rule, element) {
                    let spec = rule_max_specificity(style_rule);
                    cascade.add_declarations(&style_rule.declarations, spec);
                }
            }
            CssRule::Media(media_rule) => {
                // Always match media rules in the CSS engine (we don't have viewport info here).
                for nested in &media_rule.rules {
                    self.collect_from_rule(cascade, nested, element);
                }
            }
            CssRule::Supports(supports_rule) => {
                for nested in &supports_rule.rules {
                    self.collect_from_rule(cascade, nested, element);
                }
            }
            // Other at-rules (font-face, keyframes, import) do not contribute
            // declarations to the cascade for element styles.
            CssRule::FontFace(_)
            | CssRule::Keyframes(_)
            | CssRule::Import(_)
            | CssRule::Charset(_)
            | CssRule::Namespace(_)
            | CssRule::Page(_)
            | CssRule::Other(_) => {}
        }
    }

    /// Clear all registered stylesheets.
    pub fn clear(&mut self) {
        self.stylesheets.clear();
    }
}

impl Default for StyleResolver {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
