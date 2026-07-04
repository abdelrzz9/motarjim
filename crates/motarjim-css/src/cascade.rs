use crate::*;

/// A single declaration after cascade resolution, carrying its specificity and
/// source order for tie-breaking.
#[derive(Debug, Clone)]
pub struct ResolvedDeclaration {
    /// The CSS property name.
    pub property: SmolStr,
    /// The raw CSS value string.
    pub value: String,
    /// Whether this declaration has `!important`.
    pub important: bool,
    /// Specificity of the selector that matched this declaration `(id, class, type)`.
    pub specificity: (u32, u32, u32),
    /// Source order index (lower = earlier in source).
    pub source_order: usize,
}

// ---------------------------------------------------------------------------
// Cascade
// ---------------------------------------------------------------------------

/// Collects declarations that match an element and sorts them by CSS cascade
/// rules (specificity, importance, source order).
#[derive(Debug, Clone)]
pub struct Cascade {
    /// Resolved declarations sorted by cascade order.
    declarations: Vec<ResolvedDeclaration>,
    /// Next insertion order counter.
    next_order: usize,
}

impl Cascade {
    /// Create an empty cascade.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            declarations: Vec::new(),
            next_order: 0,
        }
    }

    /// Push a batch of declarations from a matched rule.
    pub fn add_declarations(&mut self, declarations: &[Declaration], specificity: (u32, u32, u32)) {
        for decl in declarations {
            self.declarations.push(ResolvedDeclaration {
                property: decl.property.clone(),
                value: decl.value.clone(),
                important: decl.important,
                specificity,
                source_order: self.next_order,
            });
            self.next_order += 1;
        }
    }

    /// Resolve the cascade: sort declarations by importance, specificity, and
    /// source order, keeping only the winning value per property.
    ///
    /// Returns a map of property name → resolved value.
    #[must_use]
    pub fn resolve(&self) -> HashMap<SmolStr, String> {
        let mut sorted = self.declarations.clone();

        // Sort: !important first (higher priority), then specificity (higher wins),
        // then source order (later wins).
        //
        // We sort in ascending order and then keep the *last* occurrence per property
        // since later entries override earlier ones at the same specificity level.
        sorted.sort_by(|a, b| {
            // !important wins over non-important
            let imp_cmp = a.important.cmp(&b.important);
            if imp_cmp != std::cmp::Ordering::Equal {
                return imp_cmp;
            }
            // Higher specificity wins
            let spec_cmp = (a.specificity.0, a.specificity.1, a.specificity.2).cmp(&(
                b.specificity.0,
                b.specificity.1,
                b.specificity.2,
            ));
            if spec_cmp != std::cmp::Ordering::Equal {
                return spec_cmp;
            }
            // Later source order wins
            a.source_order.cmp(&b.source_order)
        });

        // Keep the last (winning) declaration for each property.
        let mut result: HashMap<SmolStr, String> = HashMap::new();
        for decl in &sorted {
            result.insert(decl.property.clone(), decl.value.clone());
        }

        result
    }
}

impl Default for Cascade {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ComputedValues
// ---------------------------------------------------------------------------

/// Wraps a [`ComputedStyle`] with convenient construction helpers.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedValues {
    /// The underlying computed style.
    pub style: ComputedStyle,
}

impl ComputedValues {
    /// Create a new computed values wrapper with default style.
    #[must_use]
    pub fn new() -> Self {
        Self {
            style: ComputedStyle::default(),
        }
    }

    /// Build computed values from a map of resolved declarations and an optional
    /// parent style (for inheritance).
    #[must_use]
    pub fn from_map(map: &HashMap<SmolStr, String>, parent: Option<&Self>) -> Self {
        let mut cv = if let Some(p) = parent {
            // Inherit from parent first
            Self {
                style: p.style.clone(),
            }
        } else {
            Self::new()
        };

        // Apply resolved declarations on top (or as initial values).
        apply_declarations(&mut cv.style, map);
        cv
    }
}

impl Default for ComputedValues {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ComputedStyle> for ComputedValues {
    fn from(style: ComputedStyle) -> Self {
        Self { style }
    }
}

// ---------------------------------------------------------------------------
// Property application
// ---------------------------------------------------------------------------
