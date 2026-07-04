use crate::helpers::*;
use crate::*;

// ─── RemoveEmptyNodes ───────────────────────────────────────────────────────

/// Removes nodes that have no meaningful content.
///
/// This includes:
/// - Text nodes with empty or whitespace-only content
/// - Container nodes (`Container`, `Section`, `Flex`, etc.) with no children
/// - `Spacer` and `Divider` nodes with no children
/// - Nodes with `display: none`
pub struct RemoveEmptyNodes {
    /// Tracks statistics for this pass.
    statistics: PassStatistics,
}

impl RemoveEmptyNodes {
    /// Creates a new `RemoveEmptyNodes` pass.
    #[must_use]
    pub fn new() -> Self {
        Self {
            statistics: PassStatistics::new(),
        }
    }
}

impl Default for RemoveEmptyNodes {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for RemoveEmptyNodes {
    fn name(&self) -> &'static str {
        "RemoveEmptyNodes"
    }

    fn description(&self) -> &'static str {
        "Removes nodes that have no meaningful content"
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn invalidated_by(&self) -> Vec<&'static str> {
        vec![
            "RemoveEmptyNodes",
            "MergeAdjacentText",
            "FlattenNestedContainers",
            "InlineConstantValues",
        ]
    }

    fn estimated_cost(&self) -> PassCost {
        PassCost::ON
    }

    fn statistics(&self) -> &PassStatistics {
        &self.statistics
    }

    fn run(&self, tree: &mut IrTree, _context: &PassContext) -> Result<(), Vec<Diagnostic>> {
        let mut removed = 0;

        loop {
            let empty_ids: Vec<NodeId> = tree
                .nodes
                .iter()
                .filter(|n| n.id != tree.root_id)
                .filter(|n| n.parent.is_some())
                .map(|n| n.id)
                .filter(|id| {
                    self.statistics
                        .nodes_visited
                        .fetch_add(1, Ordering::Relaxed);
                    let Some(node) = tree.nodes.get(id.0 as usize) else {
                        return false;
                    };

                    if node.computed_style.display == DisplayType::None {
                        return true;
                    }

                    match &node.semantic {
                        SemanticIr::Text => {
                            get_text_content(node).is_none_or(|t| t.trim().is_empty())
                        }
                        SemanticIr::Spacer | SemanticIr::Divider => node.children.is_empty(),
                        SemanticIr::Container
                        | SemanticIr::Section
                        | SemanticIr::Flex
                        | SemanticIr::Grid
                        | SemanticIr::Column
                        | SemanticIr::Row
                        | SemanticIr::Stack
                        | SemanticIr::Scroll
                        | SemanticIr::List => node.children.is_empty(),
                        SemanticIr::Card => {
                            node.children.is_empty()
                                && node.computed_style == ComputedStyle::default()
                        }
                        _ => false,
                    }
                })
                .collect();

            if empty_ids.is_empty() {
                break;
            }

            for id in empty_ids {
                remove_node(tree, id);
                removed += 1;
            }
        }

        self.statistics
            .nodes_removed
            .fetch_add(removed, Ordering::Relaxed);

        Ok(())
    }
}

// ─── MergeAdjacentText ──────────────────────────────────────────────────────

/// Merges adjacent sibling text nodes into a single text node.
///
/// When two or more text nodes appear consecutively in a parent's child list,
/// this pass coalesces them into one node whose text content is the
/// concatenation of all the original texts.
pub struct MergeAdjacentText {
    /// Tracks statistics for this pass.
    statistics: PassStatistics,
}

impl MergeAdjacentText {
    /// Creates a new `MergeAdjacentText` pass.
    #[must_use]
    pub fn new() -> Self {
        Self {
            statistics: PassStatistics::new(),
        }
    }
}

impl Default for MergeAdjacentText {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for MergeAdjacentText {
    fn name(&self) -> &'static str {
        "MergeAdjacentText"
    }

    fn description(&self) -> &'static str {
        "Merges adjacent sibling text nodes into a single text node"
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["CollapseWhitespace"]
    }

    fn invalidated_by(&self) -> Vec<&'static str> {
        vec!["RemoveEmptyNodes", "MergeAdjacentText"]
    }

    fn estimated_cost(&self) -> PassCost {
        PassCost::ON
    }

    fn statistics(&self) -> &PassStatistics {
        &self.statistics
    }

    fn run(&self, tree: &mut IrTree, _context: &PassContext) -> Result<(), Vec<Diagnostic>> {
        let parent_ids: Vec<NodeId> = tree
            .nodes
            .iter()
            .filter(|n| n.children.len() >= 2)
            .map(|n| n.id)
            .collect();

        let mut merge_ops: Vec<(NodeId, NodeId, String)> = Vec::new();

        for &parent_id in &parent_ids {
            let Some(parent) = tree.nodes.get(parent_id.0 as usize) else {
                continue;
            };
            let children: SmallVec<[NodeId; 4]> = parent.children.clone();

            let mut i = 0;
            while i < children.len() {
                self.statistics
                    .nodes_visited
                    .fetch_add(1, Ordering::Relaxed);
                if !is_text_node(tree, children[i]) {
                    i += 1;
                    continue;
                }

                let mut j = i + 1;
                while j < children.len() && is_text_node(tree, children[j]) {
                    j += 1;
                }

                if j - i > 1 {
                    let mut merged = String::new();
                    for k in i..j {
                        if let Some(n) = tree.nodes.get(children[k].0 as usize) {
                            if let Some(t) = get_text_content(n) {
                                merged.push_str(&t);
                            }
                        }
                    }

                    for k in (i + 1..j).rev() {
                        merge_ops.push((children[i], children[k], merged.clone()));
                    }
                }

                i = j;
            }
        }

        for (keep_id, remove_id, text) in &merge_ops {
            if let Some(node) = tree.nodes.get_mut(keep_id.0 as usize) {
                set_text_content(node, text.clone());
                self.statistics
                    .nodes_modified
                    .fetch_add(1, Ordering::Relaxed);
            }
            remove_node(tree, *remove_id);
        }

        self.statistics
            .nodes_removed
            .fetch_add(merge_ops.len(), Ordering::Relaxed);

        Ok(())
    }
}

// ─── CollapseWhitespace ─────────────────────────────────────────────────────

/// Collapses runs of whitespace in text nodes into single spaces.
///
/// Leading and trailing whitespace are trimmed. This pass only modifies text
/// nodes whose content actually contains collapsible whitespace.
pub struct CollapseWhitespace {
    /// Tracks statistics for this pass.
    statistics: PassStatistics,
}

impl CollapseWhitespace {
    /// Creates a new `CollapseWhitespace` pass.
    #[must_use]
    pub fn new() -> Self {
        Self {
            statistics: PassStatistics::new(),
        }
    }
}

impl Default for CollapseWhitespace {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for CollapseWhitespace {
    fn name(&self) -> &'static str {
        "CollapseWhitespace"
    }

    fn description(&self) -> &'static str {
        "Collapses runs of whitespace in text nodes into single spaces"
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn invalidated_by(&self) -> Vec<&'static str> {
        vec!["MergeAdjacentText", "CollapseWhitespace"]
    }

    fn estimated_cost(&self) -> PassCost {
        PassCost::ON
    }

    fn statistics(&self) -> &PassStatistics {
        &self.statistics
    }

    fn run(&self, tree: &mut IrTree, _context: &PassContext) -> Result<(), Vec<Diagnostic>> {
        let mut to_collapse: Vec<(NodeId, String)> = Vec::new();

        for node in &tree.nodes {
            self.statistics
                .nodes_visited
                .fetch_add(1, Ordering::Relaxed);
            if !matches!(node.semantic, SemanticIr::Text) {
                continue;
            }
            if let Some(text) = get_text_content(node) {
                let collapsed = collapse_whitespace(&text);
                if collapsed != text {
                    to_collapse.push((node.id, collapsed));
                }
            }
        }

        let modified = to_collapse.len();
        for (id, collapsed) in &to_collapse {
            if let Some(node) = tree.nodes.get_mut(id.0 as usize) {
                set_text_content(node, collapsed.clone());
            }
        }

        self.statistics
            .nodes_modified
            .fetch_add(modified, Ordering::Relaxed);

        Ok(())
    }
}

// ─── RemoveUnusedStyles ─────────────────────────────────────────────────────

/// Removes computed-style properties that do not affect the rendered output.
///
/// For each node this pass clears:
/// - Flex properties (`flex_direction`, `flex_wrap`, `justify_content`, etc.)
///   on non-flex nodes.
/// - Grid properties on non-grid nodes.
/// - Text/font properties on non-text nodes.
/// - Default-valued `background` and `border`.
pub struct RemoveUnusedStyles {
    /// Tracks statistics for this pass.
    statistics: PassStatistics,
}

impl RemoveUnusedStyles {
    /// Creates a new `RemoveUnusedStyles` pass.
    #[must_use]
    pub fn new() -> Self {
        Self {
            statistics: PassStatistics::new(),
        }
    }
}

impl Default for RemoveUnusedStyles {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper: returns true when a `Background` is entirely empty (all fields `None`).
#[must_use]
const fn is_background_empty(bg: &motarjim_ast_html::Background) -> bool {
    bg.color.is_none()
        && bg.image.is_none()
        && bg.position.is_none()
        && bg.repeat.is_none()
        && bg.size.is_none()
}

/// Helper: returns true when a `Border` is at its default state.
#[must_use]
fn is_border_default(border: &Border) -> bool {
    border.width == EdgeValues::default()
        && border.color.is_none()
        && border.style.is_none()
        && border.radius == EdgeValues::default()
}

impl Pass for RemoveUnusedStyles {
    fn name(&self) -> &'static str {
        "RemoveUnusedStyles"
    }

    fn description(&self) -> &'static str {
        "Removes computed-style properties that do not affect the rendered output"
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn invalidated_by(&self) -> Vec<&'static str> {
        vec!["RemoveUnusedStyles"]
    }

    fn estimated_cost(&self) -> PassCost {
        PassCost::ON
    }

    fn statistics(&self) -> &PassStatistics {
        &self.statistics
    }

    fn run(&self, tree: &mut IrTree, _context: &PassContext) -> Result<(), Vec<Diagnostic>> {
        let mut modified = 0;

        for node in &mut tree.nodes {
            self.statistics
                .nodes_visited
                .fetch_add(1, Ordering::Relaxed);
            let style = &mut node.computed_style;
            let semantic = &node.semantic;

            let before = style.clone();

            if !is_flex_container(semantic) {
                style.flex_direction = None;
                style.flex_wrap = None;
                style.flex_grow = 0.0;
                style.flex_shrink = 1.0;
                style.flex_basis = None;
                style.justify_content = None;
                style.align_items = None;
                style.align_content = None;
                style.align_self = None;
                style.gap = None;
                style.row_gap = None;
                style.column_gap = None;
            }

            if !matches!(semantic, SemanticIr::Grid) {
                style.grid_template_columns = None;
                style.grid_template_rows = None;
                style.grid_column = None;
                style.grid_row = None;
            }

            if !matches!(semantic, SemanticIr::Text | SemanticIr::Heading { .. }) {
                style.color = None;
                style.font_family = None;
                style.font_size = None;
                style.font_weight = None;
                style.font_style = None;
                style.line_height = None;
                style.text_align = None;
                style.text_decoration = None;
            }

            if let Some(ref bg) = style.background {
                if is_background_empty(bg) {
                    style.background = None;
                }
            }

            if let Some(ref border) = style.border {
                if is_border_default(border) {
                    style.border = None;
                }
            }

            if *style != before {
                modified += 1;
            }
        }

        self.statistics
            .nodes_modified
            .fetch_add(modified, Ordering::Relaxed);

        Ok(())
    }
}

// ─── FlattenNestedContainers ────────────────────────────────────────────────

/// Flattens unnecessary container nesting by removing single-child containers.
///
/// When a container node has exactly one child that is also a container node,
/// the child is removed and its children are promoted to the parent. This
/// reduces tree depth without changing semantics.
pub struct FlattenNestedContainers {
    /// Tracks statistics for this pass.
    statistics: PassStatistics,
}

impl FlattenNestedContainers {
    /// Creates a new `FlattenNestedContainers` pass.
    #[must_use]
    pub fn new() -> Self {
        Self {
            statistics: PassStatistics::new(),
        }
    }
}

impl Default for FlattenNestedContainers {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for FlattenNestedContainers {
    fn name(&self) -> &'static str {
        "FlattenNestedContainers"
    }

    fn description(&self) -> &'static str {
        "Flattens unnecessary container nesting by removing single-child containers"
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["RemoveEmptyNodes"]
    }

    fn invalidated_by(&self) -> Vec<&'static str> {
        vec!["FlattenNestedContainers", "InlineConstantValues"]
    }

    fn estimated_cost(&self) -> PassCost {
        PassCost::ON
    }

    fn statistics(&self) -> &PassStatistics {
        &self.statistics
    }

    fn run(&self, tree: &mut IrTree, _context: &PassContext) -> Result<(), Vec<Diagnostic>> {
        let mut flattened = 0;

        loop {
            let mut to_flatten: Vec<(NodeId, NodeId)> = Vec::new();

            for node in &tree.nodes {
                self.statistics
                    .nodes_visited
                    .fetch_add(1, Ordering::Relaxed);
                if node.parent.is_none() {
                    continue;
                }
                if node.children.len() != 1 {
                    continue;
                }
                let child_id = node.children[0];
                let Some(child) = tree.nodes.get(child_id.0 as usize) else {
                    continue;
                };
                if is_container_role(&child.semantic) {
                    to_flatten.push((node.id, child_id));
                }
            }

            if to_flatten.is_empty() {
                break;
            }

            for (parent_id, child_id) in &to_flatten {
                let grandchild_ids: SmallVec<[NodeId; 4]> = tree
                    .nodes
                    .get(child_id.0 as usize)
                    .map(|c| c.children.clone())
                    .unwrap_or_default();

                if let Some(parent) = tree.nodes.get_mut(parent_id.0 as usize) {
                    parent.children = grandchild_ids.clone();
                }
                for &gcid in &grandchild_ids {
                    if let Some(gc) = tree.nodes.get_mut(gcid.0 as usize) {
                        gc.parent = Some(*parent_id);
                    }
                }

                remove_node(tree, *child_id);
                flattened += 1;
            }
        }

        self.statistics
            .nodes_removed
            .fetch_add(flattened, Ordering::Relaxed);

        Ok(())
    }
}

// ─── InlineConstantValues ───────────────────────────────────────────────────

/// Inlines nodes whose properties are entirely at their default values.
///
/// When a container node (not the root) has exactly one child and its computed
/// style is equal to [`ComputedStyle::default()`], the container is removed
/// and the child is promoted to replace it. This eliminates wrapper nodes that
/// carry no additional styling information.
pub struct InlineConstantValues {
    /// Tracks statistics for this pass.
    statistics: PassStatistics,
}

impl InlineConstantValues {
    /// Creates a new `InlineConstantValues` pass.
    #[must_use]
    pub fn new() -> Self {
        Self {
            statistics: PassStatistics::new(),
        }
    }
}

impl Default for InlineConstantValues {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for InlineConstantValues {
    fn name(&self) -> &'static str {
        "InlineConstantValues"
    }

    fn description(&self) -> &'static str {
        "Inlines container nodes whose properties are entirely at default values"
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["RemoveEmptyNodes", "FlattenNestedContainers"]
    }

    fn invalidated_by(&self) -> Vec<&'static str> {
        vec!["InlineConstantValues"]
    }

    fn estimated_cost(&self) -> PassCost {
        PassCost::ON
    }

    fn statistics(&self) -> &PassStatistics {
        &self.statistics
    }

    fn run(&self, tree: &mut IrTree, _context: &PassContext) -> Result<(), Vec<Diagnostic>> {
        let mut inlined = 0;

        loop {
            let mut to_inline: Vec<NodeId> = Vec::new();

            for node in &tree.nodes {
                self.statistics
                    .nodes_visited
                    .fetch_add(1, Ordering::Relaxed);
                if node.id == tree.root_id {
                    continue;
                }
                if node.parent.is_none() {
                    continue;
                }
                if !is_container_role(&node.semantic) {
                    continue;
                }
                if node.children.len() != 1 {
                    continue;
                }
                if node.computed_style == ComputedStyle::default() {
                    to_inline.push(node.id);
                }
            }

            if to_inline.is_empty() {
                break;
            }

            for &parent_id in &to_inline {
                let child_id: Option<NodeId> = tree
                    .nodes
                    .get(parent_id.0 as usize)
                    .and_then(|n| n.children.first().copied());
                let Some(child_id) = child_id else {
                    continue;
                };

                let grandchild_ids: SmallVec<[NodeId; 4]> = tree
                    .nodes
                    .get(child_id.0 as usize)
                    .map(|c| c.children.clone())
                    .unwrap_or_default();

                let grandparent_id: Option<NodeId> =
                    tree.nodes.get(parent_id.0 as usize).and_then(|n| n.parent);

                if let Some(gpid) = grandparent_id {
                    if let Some(gp) = tree.nodes.get_mut(gpid.0 as usize) {
                        let mut new_children: SmallVec<[NodeId; 4]> = SmallVec::new();
                        for &cid in &gp.children {
                            if cid == parent_id {
                                new_children.push(child_id);
                                for &gcid in &grandchild_ids {
                                    new_children.push(gcid);
                                }
                            } else {
                                new_children.push(cid);
                            }
                        }
                        gp.children = new_children;
                    }

                    if let Some(child) = tree.nodes.get_mut(child_id.0 as usize) {
                        child.parent = grandparent_id;
                    }
                    for &gcid in &grandchild_ids {
                        if let Some(gc) = tree.nodes.get_mut(gcid.0 as usize) {
                            gc.parent = Some(child_id);
                        }
                    }
                }

                remove_node(tree, parent_id);
                inlined += 1;
            }
        }

        self.statistics
            .nodes_removed
            .fetch_add(inlined, Ordering::Relaxed);

        Ok(())
    }
}

// ─── Default pass set ───────────────────────────────────────────────────────

/// Registers all standard optimization passes into the given [`PassManager`].
///
/// The passes are registered in a sensible order:
/// 1. `RemoveEmptyNodes` — clean out dead weight first
/// 2. `CollapseWhitespace` — normalize whitespace before merging
/// 3. `MergeAdjacentText` — merge text now that whitespace is consistent
/// 4. `RemoveUnusedStyles` — strip irrelevant style properties
/// 5. `FlattenNestedContainers` — reduce tree depth
/// 6. `InlineConstantValues` — remove trivial wrappers
pub fn register_default_passes(manager: &mut PassManager) {
    manager.register(Box::new(RemoveEmptyNodes::new()));
    manager.register(Box::new(CollapseWhitespace::new()));
    manager.register(Box::new(MergeAdjacentText::new()));
    manager.register(Box::new(RemoveUnusedStyles::new()));
    manager.register(Box::new(FlattenNestedContainers::new()));
    manager.register(Box::new(InlineConstantValues::new()));
}
