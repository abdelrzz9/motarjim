#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Optimization pass manager and passes for the Motarjim compiler IR.
//!
//! This crate provides a [`PassManager`] that registers and runs optimization
//! passes on an [`IrTree`]. Each pass implements the [`Pass`] trait and
//! reports results via [`PassResult`].

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use motarjim_ast::ir::{IrNode, IrTree, SemanticIr, TargetIr};
use motarjim_ast::style::{Border, ComputedStyle, DisplayType, EdgeValues};
use motarjim_ast::NodeId;
use motarjim_diag::Diagnostic;
use smallvec::SmallVec;

/// The estimated computational cost of an optimization pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassCost {
    /// O(1) — constant time.
    O1,
    /// O(log n) — logarithmic.
    OLogN,
    /// O(n) — linear.
    ON,
    /// O(n log n) — linearithmic.
    ONLogN,
    /// O(n²) — quadratic.
    ON2,
}

/// Runtime statistics collected by an optimization pass.
///
/// All counters use atomic operations so they can be updated from multiple
/// threads and safely snapshotted by the pass manager.
#[derive(Debug, Default)]
pub struct PassStatistics {
    /// Total number of nodes visited during the pass.
    pub nodes_visited: AtomicUsize,
    /// Number of nodes modified (but not removed).
    pub nodes_modified: AtomicUsize,
    /// Number of nodes removed.
    pub nodes_removed: AtomicUsize,
    /// Approximate bytes of memory freed.
    pub memory_freed: AtomicUsize,
    /// Wall-clock duration of the pass in nanoseconds (set by the pass manager).
    pub duration_ns: AtomicU64,
}

impl PassStatistics {
    /// Creates a new statistics counter with all values at zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Atomically snapshots the current counters into a [`PassStatsSnapshot`].
    #[must_use]
    pub fn snapshot(&self) -> PassStatsSnapshot {
        PassStatsSnapshot {
            nodes_visited: self.nodes_visited.load(Ordering::SeqCst),
            nodes_modified: self.nodes_modified.load(Ordering::SeqCst),
            nodes_removed: self.nodes_removed.load(Ordering::SeqCst),
            memory_freed: self.memory_freed.load(Ordering::SeqCst),
            duration_ns: self.duration_ns.load(Ordering::SeqCst),
        }
    }

    /// Resets all counters to zero.
    pub fn reset(&self) {
        self.nodes_visited.store(0, Ordering::SeqCst);
        self.nodes_modified.store(0, Ordering::SeqCst);
        self.nodes_removed.store(0, Ordering::SeqCst);
        self.memory_freed.store(0, Ordering::SeqCst);
        self.duration_ns.store(0, Ordering::SeqCst);
    }
}

/// A point-in-time snapshot of [`PassStatistics`].
#[derive(Debug, Clone, Default)]
pub struct PassStatsSnapshot {
    /// Total number of nodes visited during the pass.
    pub nodes_visited: usize,
    /// Number of nodes modified (but not removed).
    pub nodes_modified: usize,
    /// Number of nodes removed.
    pub nodes_removed: usize,
    /// Approximate bytes of memory freed.
    pub memory_freed: usize,
    /// Wall-clock duration of the pass in nanoseconds.
    pub duration_ns: u64,
}

/// A token that signals cancellation of a long-running pass.
///
/// Cloning shares the underlying cancellation state — cancelling any clone
/// signals cancellation to all of them.
#[derive(Clone)]
pub struct CancelToken {
    /// Shared atomic flag; `true` once [`cancel`](Self::cancel) is called.
    cancelled: Arc<AtomicBool>,
}

impl CancelToken {
    /// Creates a new cancellation token that is **not** cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Requests cancellation. All clones of this token will now report
    /// [`is_cancelled`](Self::is_cancelled) as `true`.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns `true` if [`cancel`](Self::cancel) has been called on any clone.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Context provided to every pass during [`Pass::run`].
#[derive(Default)]
pub struct PassContext {
    /// Token used to cooperatively cancel a long-running pass.
    pub cancel_token: CancelToken,
}

impl PassContext {
    /// Creates a new pass context with the given cancellation token.
    #[must_use]
    pub const fn new(cancel_token: CancelToken) -> Self {
        Self { cancel_token }
    }
}


/// The result produced by the pass manager after executing a single pass.
#[derive(Debug, Clone)]
pub struct PassResult {
    /// The name of the pass that produced this result.
    pub pass_name: &'static str,
    /// A human-readable description of what the pass does.
    pub description: &'static str,
    /// How many nodes were removed by the pass.
    pub nodes_removed: usize,
    /// How many nodes were modified (but not removed) by the pass.
    pub nodes_modified: usize,
    /// How many nodes were visited during the pass.
    pub nodes_visited: usize,
    /// Wall-clock duration of the pass in nanoseconds.
    pub duration_ns: u64,
    /// The estimated computational cost of the pass.
    pub cost: PassCost,
    /// Diagnostics emitted during the pass.
    pub diagnostics: Vec<Diagnostic>,
}

/// Trait implemented by all optimization passes.
///
/// Each pass operates on the full [`IrTree`] and reports its results through
/// the statistics counters available via [`statistics`](Self::statistics).
pub trait Pass: Send + Sync {
    /// Returns the name of this pass (used in [`PassResult`] and logging).
    fn name(&self) -> &'static str;

    /// Returns a human-readable description of what this pass does.
    fn description(&self) -> &'static str;

    /// Returns the names of passes that must be run before this one.
    fn prerequisites(&self) -> Vec<&'static str>;

    /// Returns the names of passes whose results are invalidated by this one.
    fn invalidated_by(&self) -> Vec<&'static str>;

    /// Returns the estimated computational cost of this pass.
    fn estimated_cost(&self) -> PassCost;

    /// Returns a reference to this pass's [`PassStatistics`].
    fn statistics(&self) -> &PassStatistics;

    /// Runs this pass on the given IR tree.
    ///
    /// The pass may restructure, remove, or modify nodes in the tree. It
    /// updates its own [`PassStatistics`] counters to report what was done.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a list of [`Diagnostic`]s if the pass encounters
    /// a problem (e.g. malformed IR).
    fn run(&self, tree: &mut IrTree, context: &PassContext) -> Result<(), Vec<Diagnostic>>;
}

/// Manages registration and sequential execution of optimization passes.
///
/// Passes are registered with [`register`](PassManager::register) and executed
/// in registration order with [`run_all`](PassManager::run_all).
pub struct PassManager {
    /// Registered optimization passes.
    passes: Vec<Box<dyn Pass>>,
}

impl PassManager {
    /// Creates a new empty pass manager.
    #[must_use]
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Registers a pass to be run during [`run_all`](PassManager::run_all).
    pub fn register(&mut self, pass: Box<dyn Pass>) {
        self.passes.push(pass);
    }

    /// Runs all registered passes in registration order, returning all results.
    ///
    /// Uses a default [`PassContext`] with a fresh [`CancelToken`].
    pub fn run_all(&self, tree: &mut IrTree) -> Vec<PassResult> {
        let context = PassContext::default();
        self.run_with_context(tree, &context)
    }

    /// Runs all registered passes with the given context, returning all results.
    pub fn run_with_context(&self, tree: &mut IrTree, context: &PassContext) -> Vec<PassResult> {
        let mut results = Vec::with_capacity(self.passes.len());
        for pass in &self.passes {
            let start = Instant::now();
            let diagnostics = match pass.run(tree, context) {
                Ok(()) => Vec::new(),
                Err(diags) => diags,
            };
            let duration_ns = start.elapsed().as_nanos() as u64;
            let stats = pass.statistics().snapshot();
            results.push(PassResult {
                pass_name: pass.name(),
                description: pass.description(),
                nodes_removed: stats.nodes_removed,
                nodes_modified: stats.nodes_modified,
                nodes_visited: stats.nodes_visited,
                duration_ns,
                cost: pass.estimated_cost(),
                diagnostics,
            });
        }
        results
    }

    /// Returns the number of registered passes.
    #[must_use]
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Returns a snapshot of all registered passes' statistics, keyed by pass name.
    #[must_use]
    pub fn statistics(&self) -> HashMap<&'static str, PassStatsSnapshot> {
        let mut map = HashMap::with_capacity(self.passes.len());
        for pass in &self.passes {
            map.insert(pass.name(), pass.statistics().snapshot());
        }
        map
    }

    /// Returns the number of registered passes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.passes.len()
    }

    /// Returns `true` if no passes are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.passes.is_empty()
    }
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helper functions ───────────────────────────────────────────────────────

/// Returns the text content of an IR node that has [`SemanticIr::Text`] role.
///
/// Looks for `"text"`, `"data"`, or `"content"` properties in Flutter, Compose,
/// and `SwiftUI` target variants. For `Generic` targets the node name is returned
/// when it is not `"Text"`.
#[must_use]
fn get_text_content(node: &IrNode) -> Option<String> {
    if !matches!(node.semantic, SemanticIr::Text) {
        return None;
    }
    match &node.target {
        TargetIr::Flutter { properties, .. }
        | TargetIr::Compose { properties, .. }
        | TargetIr::SwiftUI { properties, .. } => properties
            .iter()
            .find(|(k, _)| k.as_str() == "text" || k.as_str() == "data" || k.as_str() == "content")
            .map(|(_, v)| v.clone()),
        TargetIr::Generic { node: n, .. } => {
            if n.as_str() == "Text" {
                None
            } else {
                Some(n.to_string())
            }
        }
    }
}

/// Sets the text content of an IR node by updating its target properties.
fn set_text_content(node: &mut IrNode, text: String) {
    match &mut node.target {
        TargetIr::Flutter { properties, .. }
        | TargetIr::Compose { properties, .. }
        | TargetIr::SwiftUI { properties, .. } => {
            for (k, v) in properties.iter_mut() {
                if k.as_str() == "text" || k.as_str() == "data" || k.as_str() == "content" {
                    *v = text;
                    return;
                }
            }
            properties.push((smol_str::SmolStr::new_inline("text"), text));
        }
        TargetIr::Generic { node: n, .. } => {
            *n = smol_str::SmolStr::new(&text);
        }
    }
}

/// Returns `true` if the semantic role represents a container-like node.
#[must_use]
const fn is_container_role(semantic: &SemanticIr) -> bool {
    matches!(
        semantic,
        SemanticIr::Root
            | SemanticIr::Navigation
            | SemanticIr::NavigationBar
            | SemanticIr::HeroSection
            | SemanticIr::Card
            | SemanticIr::Section
            | SemanticIr::Article
            | SemanticIr::Aside
            | SemanticIr::Footer
            | SemanticIr::Header
            | SemanticIr::Main
            | SemanticIr::Container
            | SemanticIr::Grid
            | SemanticIr::Flex
            | SemanticIr::Column
            | SemanticIr::Row
            | SemanticIr::Stack
            | SemanticIr::Scroll
            | SemanticIr::List
            | SemanticIr::Form
            | SemanticIr::Dialog
            | SemanticIr::Table
            | SemanticIr::TableRow
    )
}

/// Returns `true` if the semantic role represents a flex container.
#[must_use]
const fn is_flex_container(semantic: &SemanticIr) -> bool {
    matches!(semantic, SemanticIr::Flex | SemanticIr::Row | SemanticIr::Column)
}

/// Returns `true` if the node identified by `id` is a text node.
#[must_use]
fn is_text_node(tree: &IrTree, id: NodeId) -> bool {
    tree.nodes
        .get(id.0 as usize)
        .is_some_and(|n| matches!(n.semantic, SemanticIr::Text))
}

/// Detaches a node from its parent, making it unreachable in the tree.
///
/// The node remains in the `nodes` vector but its `parent` is set to `None`
/// and its ID is removed from the parent's `children` list.
fn remove_node(tree: &mut IrTree, id: NodeId) {
    let parent_id = tree
        .nodes
        .get(id.0 as usize)
        .and_then(|n| n.parent);
    if let Some(pid) = parent_id {
        if let Some(parent) = tree.nodes.get_mut(pid.0 as usize) {
            parent.children.retain(|c| *c != id);
        }
    }
    if let Some(node) = tree.nodes.get_mut(id.0 as usize) {
        node.parent = None;
    }
}

/// Collapses runs of whitespace characters into single spaces, trimming ends.
#[must_use]
fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_whitespace = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !in_whitespace {
                result.push(' ');
                in_whitespace = true;
            }
        } else {
            result.push(ch);
            in_whitespace = false;
        }
    }
    let trimmed = result.trim().to_string();
    if trimmed.is_empty() && !s.is_empty() {
        String::new()
    } else {
        trimmed
    }
}

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
        vec!["RemoveEmptyNodes", "MergeAdjacentText", "FlattenNestedContainers", "InlineConstantValues"]
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
                    self.statistics.nodes_visited.fetch_add(1, Ordering::Relaxed);
                    let Some(node) = tree.nodes.get(id.0 as usize) else {
                        return false;
                    };

                    if node.computed_style.display == DisplayType::None {
                        return true;
                    }

                    match &node.semantic {
                        SemanticIr::Text => get_text_content(node)
                            .is_none_or(|t| t.trim().is_empty()),
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

        self.statistics.nodes_removed.fetch_add(removed, Ordering::Relaxed);

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
                self.statistics.nodes_visited.fetch_add(1, Ordering::Relaxed);
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
                self.statistics.nodes_modified.fetch_add(1, Ordering::Relaxed);
            }
            remove_node(tree, *remove_id);
        }

        self.statistics.nodes_removed.fetch_add(merge_ops.len(), Ordering::Relaxed);

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
            self.statistics.nodes_visited.fetch_add(1, Ordering::Relaxed);
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

        self.statistics.nodes_modified.fetch_add(modified, Ordering::Relaxed);

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
const fn is_background_empty(bg: &motarjim_ast::style::Background) -> bool {
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
            self.statistics.nodes_visited.fetch_add(1, Ordering::Relaxed);
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

        self.statistics.nodes_modified.fetch_add(modified, Ordering::Relaxed);

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
                self.statistics.nodes_visited.fetch_add(1, Ordering::Relaxed);
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

        self.statistics.nodes_removed.fetch_add(flattened, Ordering::Relaxed);

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
                self.statistics.nodes_visited.fetch_add(1, Ordering::Relaxed);
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

                let grandparent_id: Option<NodeId> = tree
                    .nodes
                    .get(parent_id.0 as usize)
                    .and_then(|n| n.parent);

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

        self.statistics.nodes_removed.fetch_add(inlined, Ordering::Relaxed);

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

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_ast::ir::LayoutIr;
    use motarjim_ast::style::{FlexDirection, JustifyContent};

    fn default_context() -> PassContext {
        PassContext::default()
    }

    /// Helper to build an IrNode for testing.
    fn make_node(
        id: u32,
        semantic: SemanticIr,
        layout: LayoutIr,
        children: SmallVec<[NodeId; 4]>,
        parent: Option<NodeId>,
    ) -> IrNode {
        IrNode {
            id: NodeId(id),
            semantic,
            layout,
            target: TargetIr::Generic {
                platform: smol_str::SmolStr::new_inline("flutter"),
                node: smol_str::SmolStr::new_inline("Container"),
            },
            computed_style: ComputedStyle::default(),
            children,
            parent,
        }
    }

    /// Helper to build a text IrNode with the given text content.
    fn make_text_node(
        id: u32,
        text: &str,
        children: SmallVec<[NodeId; 4]>,
        parent: Option<NodeId>,
    ) -> IrNode {
        IrNode {
            id: NodeId(id),
            semantic: SemanticIr::Text,
            layout: LayoutIr::Flow,
            target: TargetIr::Flutter {
                widget: smol_str::SmolStr::new_inline("Text"),
                properties: vec![(
                    smol_str::SmolStr::new_inline("data"),
                    text.to_string(),
                )],
            },
            computed_style: ComputedStyle::default(),
            children,
            parent,
        }
    }

    /// Helper to create a simple tree with root + children.
    fn simple_tree(nodes: Vec<IrNode>) -> IrTree {
        IrTree {
            nodes,
            root_id: NodeId(0),
            target_hints: Vec::new(),
        }
    }

    // ─── RemoveEmptyNodes tests ──────────────────────────────────────────

    #[test]
    fn test_remove_empty_text_node() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_text_node(1, "", SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = RemoveEmptyNodes::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_removed, 1);

        if let Some(root) = tree.nodes.get(0) {
            assert!(root.children.is_empty());
        }
    }

    #[test]
    fn test_remove_whitespace_only_text_node() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_text_node(1, "   \n  \t  ", SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = RemoveEmptyNodes::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_removed, 1);
        if let Some(root) = tree.nodes.get(0) {
            assert!(root.children.is_empty());
        }
    }

    #[test]
    fn test_remove_empty_container() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_node(1, SemanticIr::Container, LayoutIr::FlexColumn, SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = RemoveEmptyNodes::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_removed, 1);
        if let Some(root) = tree.nodes.get(0) {
            assert!(root.children.is_empty());
        }
    }

    #[test]
    fn test_remove_display_none_node() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_node(1, SemanticIr::Button, LayoutIr::Static, SmallVec::new(), Some(NodeId(0))),
        ]);
        if let Some(btn) = tree.nodes.get_mut(1) {
            btn.computed_style.display = DisplayType::None;
        }

        let pass = RemoveEmptyNodes::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_removed, 1);
        if let Some(root) = tree.nodes.get(0) {
            assert!(root.children.is_empty());
        }
    }

    #[test]
    fn test_keep_non_empty_text_node() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_text_node(1, "Hello", SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = RemoveEmptyNodes::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_removed, 0, "non-empty text node should not be removed");
        if let Some(text_node) = tree.nodes.get(1) {
            assert_eq!(text_node.semantic, SemanticIr::Text);
        }
    }

    // ─── MergeAdjacentText tests ─────────────────────────────────────────

    #[test]
    fn test_merge_two_adjacent_text_nodes() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1), NodeId(2)],
                None,
            ),
            make_text_node(1, "Hello ", SmallVec::new(), Some(NodeId(0))),
            make_text_node(2, "World", SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = MergeAdjacentText::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert!(stats.nodes_removed >= 1);

        if let Some(root) = tree.nodes.get(0) {
            assert_eq!(root.children.len(), 1);
            if let Some(remaining) = root.children.first().and_then(|&id| tree.nodes.get(id.0 as usize)) {
                assert_eq!(get_text_content(remaining).as_deref(), Some("Hello World"));
            }
        }
    }

    #[test]
    fn test_merge_three_adjacent_text_nodes() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1), NodeId(2), NodeId(3)],
                None,
            ),
            make_text_node(1, "A", SmallVec::new(), Some(NodeId(0))),
            make_text_node(2, "B", SmallVec::new(), Some(NodeId(0))),
            make_text_node(3, "C", SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = MergeAdjacentText::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert!(stats.nodes_removed >= 1);

        if let Some(root) = tree.nodes.get(0) {
            assert_eq!(root.children.len(), 1);
            if let Some(remaining) = root.children.first().and_then(|&id| tree.nodes.get(id.0 as usize)) {
                assert_eq!(get_text_content(remaining).as_deref(), Some("ABC"));
            }
        }
    }

    #[test]
    fn test_merge_does_not_merge_non_text_siblings() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1), NodeId(2)],
                None,
            ),
            make_text_node(1, "Hello", SmallVec::new(), Some(NodeId(0))),
            make_node(2, SemanticIr::Button, LayoutIr::Static, SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = MergeAdjacentText::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_removed, 0);

        if let Some(root) = tree.nodes.get(0) {
            assert_eq!(root.children.len(), 2);
        }
    }

    // ─── CollapseWhitespace tests ────────────────────────────────────────

    #[test]
    fn test_collapse_whitespace_in_text_node() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_text_node(1, "Hello    World\n  Foo", SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = CollapseWhitespace::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_modified, 1);

        if let Some(text_node) = tree.nodes.get(1) {
            assert_eq!(get_text_content(text_node).as_deref(), Some("Hello World Foo"));
        }
    }

    #[test]
    fn test_collapse_whitespace_no_change() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_text_node(1, "Hello World", SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = CollapseWhitespace::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_modified, 0);

        if let Some(text_node) = tree.nodes.get(1) {
            assert_eq!(get_text_content(text_node).as_deref(), Some("Hello World"));
        }
    }

    #[test]
    fn test_collapse_whitespace_trims_edges() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_text_node(1, "   Hello World   ", SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = CollapseWhitespace::new();
        let ctx = default_context();
        let _ = pass.run(&mut tree, &ctx);

        if let Some(text_node) = tree.nodes.get(1) {
            assert_eq!(get_text_content(text_node).as_deref(), Some("Hello World"));
        }
    }

    // ─── RemoveUnusedStyles tests ────────────────────────────────────────

    #[test]
    fn test_remove_flex_styles_from_non_flex() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_node(1, SemanticIr::Button, LayoutIr::Static, SmallVec::new(), Some(NodeId(0))),
        ]);
        if let Some(btn) = tree.nodes.get_mut(1) {
            btn.computed_style.flex_direction = Some(FlexDirection::Row);
            btn.computed_style.justify_content = Some(JustifyContent::Center);
        }

        let pass = RemoveUnusedStyles::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_modified, 1);

        if let Some(button) = tree.nodes.get(1) {
            assert_eq!(button.computed_style.flex_direction, None);
            assert_eq!(button.computed_style.justify_content, None);
        }
    }

    #[test]
    fn test_remove_grid_styles_from_non_grid() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_node(1, SemanticIr::Section, LayoutIr::Stack, SmallVec::new(), Some(NodeId(0))),
        ]);
        if let Some(node) = tree.nodes.get_mut(1) {
            node.computed_style.grid_template_columns = Some("1fr 1fr".to_string());
        }

        let pass = RemoveUnusedStyles::new();
        let ctx = default_context();
        let _ = pass.run(&mut tree, &ctx);

        if let Some(node) = tree.nodes.get(1) {
            assert_eq!(node.computed_style.grid_template_columns, None);
        }
    }

    #[test]
    fn test_keep_flex_styles_on_flex_container() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_node(1, SemanticIr::Flex, LayoutIr::FlexColumn, SmallVec::new(), Some(NodeId(0))),
        ]);
        if let Some(node) = tree.nodes.get_mut(1) {
            node.computed_style.flex_direction = Some(FlexDirection::Row);
            node.computed_style.justify_content = Some(JustifyContent::Center);
        }

        let pass = RemoveUnusedStyles::new();
        let ctx = default_context();
        let _ = pass.run(&mut tree, &ctx);

        if let Some(node) = tree.nodes.get(1) {
            assert_eq!(node.computed_style.flex_direction, Some(FlexDirection::Row));
            assert_eq!(node.computed_style.justify_content, Some(JustifyContent::Center));
        }
    }

    // ─── FlattenNestedContainers tests ────────────────────────────────────

    #[test]
    fn test_flatten_nested_container() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_node(
                1,
                SemanticIr::Container,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(2)],
                Some(NodeId(0)),
            ),
            make_node(
                2,
                SemanticIr::Container,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(3)],
                Some(NodeId(1)),
            ),
            make_text_node(3, "deep", SmallVec::new(), Some(NodeId(2))),
        ]);

        let pass = FlattenNestedContainers::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert!(stats.nodes_removed >= 1);

        if let Some(container) = tree.nodes.get(1) {
            assert!(container.children.contains(&NodeId(3)));
        }
    }

    #[test]
    fn test_flatten_does_not_remove_non_container_child() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_node(
                1,
                SemanticIr::Container,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(2)],
                Some(NodeId(0)),
            ),
            make_text_node(2, "hello", SmallVec::new(), Some(NodeId(1))),
        ]);

        let pass = FlattenNestedContainers::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_removed, 0);
    }

    // ─── InlineConstantValues tests ──────────────────────────────────────

    #[test]
    fn test_inline_constant_container() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_node(
                1,
                SemanticIr::Section,
                LayoutIr::Stack,
                smallvec::smallvec![NodeId(2)],
                Some(NodeId(0)),
            ),
            make_text_node(2, "content", SmallVec::new(), Some(NodeId(1))),
        ]);

        let pass = InlineConstantValues::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert!(stats.nodes_removed >= 1);

        if let Some(root) = tree.nodes.get(0) {
            assert!(root.children.contains(&NodeId(2)));
        }
    }

    #[test]
    fn test_does_not_inline_root() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_text_node(1, "content", SmallVec::new(), Some(NodeId(0))),
        ]);

        let pass = InlineConstantValues::new();
        let ctx = default_context();
        pass.run(&mut tree, &ctx).unwrap();
        let stats = pass.statistics().snapshot();
        assert_eq!(stats.nodes_removed, 0);
    }

    // ─── PassManager tests ───────────────────────────────────────────────

    #[test]
    fn test_pass_manager_empty() {
        let manager = PassManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_pass_manager_register_and_run() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_text_node(1, "", SmallVec::new(), Some(NodeId(0))),
        ]);

        let mut manager = PassManager::new();
        manager.register(Box::new(RemoveEmptyNodes::new()));

        assert_eq!(manager.len(), 1);
        let results = manager.run_all(&mut tree);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].nodes_removed, 1);
    }

    #[test]
    fn test_register_default_passes() {
        let mut manager = PassManager::new();
        register_default_passes(&mut manager);
        assert_eq!(manager.len(), 6);
    }

    #[test]
    fn test_pass_manager_pass_count() {
        let mut manager = PassManager::new();
        register_default_passes(&mut manager);
        assert_eq!(manager.pass_count(), 6);
    }

    #[test]
    fn test_pass_manager_statistics() {
        let mut manager = PassManager::new();
        register_default_passes(&mut manager);
        let stats = manager.statistics();
        assert!(stats.contains_key("RemoveEmptyNodes"));
        assert!(stats.contains_key("CollapseWhitespace"));
        assert!(stats.contains_key("MergeAdjacentText"));
        assert!(stats.contains_key("RemoveUnusedStyles"));
        assert!(stats.contains_key("FlattenNestedContainers"));
        assert!(stats.contains_key("InlineConstantValues"));
    }

    #[test]
    fn test_pass_result_contains_cost() {
        let mut tree = simple_tree(vec![
            make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                smallvec::smallvec![NodeId(1)],
                None,
            ),
            make_text_node(1, "", SmallVec::new(), Some(NodeId(0))),
        ]);

        let mut manager = PassManager::new();
        manager.register(Box::new(RemoveEmptyNodes::new()));

        let results = manager.run_all(&mut tree);
        assert_eq!(results[0].cost, PassCost::ON);
        assert!(results[0].duration_ns > 0);
    }

    #[test]
    fn test_pass_statistics_snapshot() {
        let stats = PassStatistics::new();
        let snap = stats.snapshot();
        assert_eq!(snap.nodes_visited, 0);
        assert_eq!(snap.nodes_modified, 0);
        assert_eq!(snap.nodes_removed, 0);
        assert_eq!(snap.memory_freed, 0);
        assert_eq!(snap.duration_ns, 0);
    }

    #[test]
    fn test_pass_statistics_reset() {
        let stats = PassStatistics::new();
        stats.nodes_visited.fetch_add(42, Ordering::Relaxed);
        stats.reset();
        assert_eq!(stats.nodes_visited.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_pass_cost_equality() {
        assert_eq!(PassCost::O1, PassCost::O1);
        assert_ne!(PassCost::O1, PassCost::ON);
    }

    #[test]
    fn test_cancel_token() {
        let token = CancelToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    // ─── Integration: multiple passes ────────────────────────────────────

    #[test]
    fn test_full_pipeline_on_complex_tree() {
        let mut tree = IrTree {
            nodes: vec![
                make_node(
                    0,
                    SemanticIr::Root,
                    LayoutIr::FlexColumn,
                    smallvec::smallvec![NodeId(1), NodeId(2), NodeId(3), NodeId(4)],
                    None,
                ),
                make_node(1, SemanticIr::Container, LayoutIr::FlexColumn, SmallVec::new(), Some(NodeId(0))),
                make_text_node(2, "Hello    World", SmallVec::new(), Some(NodeId(0))),
                make_text_node(3, "Foo", SmallVec::new(), Some(NodeId(0))),
                make_node(
                    4,
                    SemanticIr::Section,
                    LayoutIr::Stack,
                    smallvec::smallvec![NodeId(5)],
                    Some(NodeId(0)),
                ),
                make_node(
                    5,
                    SemanticIr::Container,
                    LayoutIr::FlexColumn,
                    smallvec::smallvec![NodeId(6)],
                    Some(NodeId(4)),
                ),
                make_text_node(6, "Bar", SmallVec::new(), Some(NodeId(5))),
            ],
            root_id: NodeId(0),
            target_hints: Vec::new(),
        };

        let mut manager = PassManager::new();
        register_default_passes(&mut manager);
        let results = manager.run_all(&mut tree);

        let total_removed: usize = results.iter().map(|r| r.nodes_removed).sum();
        assert!(total_removed >= 3, "Expected at least 3 removals, got {total_removed}");

        if let Some(root) = tree.nodes.get(0) {
            let has_text_child = root.children.iter().any(|&cid| {
                tree.nodes
                    .get(cid.0 as usize)
                    .map_or(false, |n| matches!(n.semantic, SemanticIr::Text))
            });
            assert!(has_text_child, "root should have text children after passes");
        }
    }

    // ─── collapse_whitespace unit tests ──────────────────────────────────

    #[test]
    fn test_collapse_whitespace_simple() {
        assert_eq!(collapse_whitespace("a   b"), "a b");
    }

    #[test]
    fn test_collapse_whitespace_newlines_and_tabs() {
        assert_eq!(collapse_whitespace("a\n\tb\n\nc"), "a b c");
    }

    #[test]
    fn test_collapse_whitespace_trim() {
        assert_eq!(collapse_whitespace("  hello  "), "hello");
    }

    #[test]
    fn test_collapse_whitespace_empty() {
        assert_eq!(collapse_whitespace(""), "");
    }

    #[test]
    fn test_collapse_whitespace_all_whitespace() {
        assert_eq!(collapse_whitespace("   \n  \t  "), "");
    }

    #[test]
    fn test_collapse_whitespace_no_change_needed() {
        assert_eq!(collapse_whitespace("hello world"), "hello world");
    }
}
