#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

//! Compilation DAG — a directed acyclic graph-based compilation scheduler.
//!
//! Replaces the sequential pipeline with a dependency-aware DAG that can
//! execute independent phases in parallel via [`rayon`].
//!
//! # Architecture
//!
//! Each compilation phase is a [`CompilationNode`] with declared dependencies.
//! The [`CompilationDag`] computes a topological order and executes phases
//! level-by-level: phases at the same level (no transitive dependencies) run
//! concurrently.
//!
//! # Level-based scheduling (default DAG)
//!
//! | Level | Phases |
//! |-------|--------|
//! | 0 | ReadFiles, ParseHtml, ParseCss |
//! | 1 | CssSelectorMatching |
//! | 2 | CascadeStyles |
//! | 3 | SemanticInference, LayoutInference, AccessibilityAnalysis |
//! | 4 | BuildIr |
//! | 5 | OptimizeIr |
//! | 6 | GenerateFlutter, GenerateCompose, GenerateSwiftui |

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use motarjim_ast::css::CssStylesheet;
use motarjim_ast::ir::IrTree;
use motarjim_ast::{ComputedStyle, Document, NodeId, StyledDocument, StyledNode};
use motarjim_diag::{Diagnostic, DiagnosticBag, Severity};
use rayon::prelude::*;

/// Identifies a single phase in the compilation DAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Read source files from disk.
    ReadFiles,
    /// Parse HTML source into a [`Document`].
    ParseHtml,
    /// Parse CSS source into a [`CssStylesheet`].
    ParseCss,
    /// Match CSS selectors against HTML elements.
    CssSelectorMatching,
    /// Resolve the cascade for matched styles.
    CascadeStyles,
    /// Infer semantic roles for each node.
    SemanticInference,
    /// Infer layout strategies for each node.
    LayoutInference,
    /// Analyse accessibility metadata for each node.
    AccessibilityAnalysis,
    /// Build the intermediate representation tree.
    BuildIr,
    /// Run optimisation passes on the IR tree.
    OptimizeIr,
    /// Generate Flutter/Dart code.
    GenerateFlutter,
    /// Generate Jetpack Compose/Kotlin code.
    GenerateCompose,
    /// Generate SwiftUI/Swift code.
    GenerateSwiftui,
}

/// The output produced by a single compilation phase.
///
/// Each variant wraps a concrete type that downstream phases can read from
/// [`CompilationContext::inputs`].
#[derive(Debug, Clone)]
pub enum PhaseOutput {
    /// A parsed HTML document.
    Document(Document),
    /// A parsed CSS stylesheet.
    Stylesheet(CssStylesheet),
    /// A document with per-node resolved styles.
    StyledDocument(StyledDocument),
    /// A map of node IDs to computed styles.
    StyleMap(HashMap<NodeId, ComputedStyle>),
    /// The intermediate representation tree.
    IrTree(IrTree),
    /// Generated platform source code.
    Code(String),
}

impl PhaseOutput {
    /// If this output is a `Document`, returns a reference to it.
    #[must_use]
    pub fn as_document(&self) -> Option<&Document> {
        match self {
            Self::Document(d) => Some(d),
            _ => None,
        }
    }

    /// If this output is a `Stylesheet`, returns a reference to it.
    #[must_use]
    pub fn as_stylesheet(&self) -> Option<&CssStylesheet> {
        match self {
            Self::Stylesheet(s) => Some(s),
            _ => None,
        }
    }

    /// If this output is a `StyledDocument`, returns a reference to it.
    #[must_use]
    pub fn as_styled_document(&self) -> Option<&StyledDocument> {
        match self {
            Self::StyledDocument(d) => Some(d),
            _ => None,
        }
    }

    /// If this output is a `StyleMap`, returns a reference to it.
    #[must_use]
    pub fn as_style_map(&self) -> Option<&HashMap<NodeId, ComputedStyle>> {
        match self {
            Self::StyleMap(m) => Some(m),
            _ => None,
        }
    }

    /// If this output is an `IrTree`, returns a reference to it.
    #[must_use]
    pub fn as_ir_tree(&self) -> Option<&IrTree> {
        match self {
            Self::IrTree(t) => Some(t),
            _ => None,
        }
    }

    /// If this output is `Code`, returns the string content.
    #[must_use]
    pub fn as_code(&self) -> Option<&str> {
        match self {
            Self::Code(c) => Some(c.as_str()),
            _ => None,
        }
    }
}

/// A lightweight cancellation token for cooperative cancellation.
///
/// This is a local re-implementation that does not depend on the
/// `cancellation` feature flag of `motarjim-core`.
#[derive(Clone)]
pub struct CancelToken {
    /// Shared atomic flag; `true` once [`cancel`](Self::cancel) is called.
    cancelled: Arc<AtomicBool>,
}

impl CancelToken {
    /// Creates a new token that is **not** cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Signals cancellation. All clones now report `true` for
    /// [`is_cancelled`](Self::is_cancelled).
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns `true` if [`cancel`](Self::cancel) has been called.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Returns `Err(Cancelled)` if cancelled, or `Ok(())` otherwise.
    ///
    /// # Errors
    /// Returns [`Cancelled`] when cancellation has been requested.
    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned when a cancelled operation is detected.
#[derive(Debug, Clone, Copy)]
pub struct Cancelled;

/// Shared context available to every phase during DAG execution.
///
/// Phases read their inputs from [`inputs`](Self::inputs) and write their
/// outputs back to the same map. Non-fatal diagnostics are collected in
/// [`diagnostics`](Self::diagnostics).
pub struct CompilationContext {
    /// The raw HTML source string.
    pub html_source: String,
    /// An optional raw CSS source string.
    pub css_source: Option<String>,
    /// All phase outputs produced so far, keyed by phase.
    pub inputs: HashMap<Phase, PhaseOutput>,
    /// Bag for collecting non-fatal diagnostics.
    pub diagnostics: DiagnosticBag,
    /// Token checked by phases for cooperative cancellation.
    pub cancel_token: CancelToken,
}

impl CompilationContext {
    /// Creates a new compilation context with the given sources.
    #[must_use]
    pub fn new(html_source: String, css_source: Option<String>) -> Self {
        Self {
            html_source,
            css_source,
            inputs: HashMap::new(),
            diagnostics: DiagnosticBag::new(),
            cancel_token: CancelToken::new(),
        }
    }
}

/// A boxed phase-execution closure; see [`CompilationNode::run`].
pub type PhaseFn =
    Box<dyn Fn(&CompilationContext) -> Result<PhaseOutput, Vec<Diagnostic>> + Send + Sync>;

/// A single node (phase) in the compilation DAG.
///
/// Each node has a [`phase`](Self::phase) identifier, a list of
/// [`dependencies`](Self::dependencies) it depends on, and a [`run`](Self::run)
/// closure that performs the phase's work.
pub struct CompilationNode {
    /// The phase this node represents.
    pub phase: Phase,
    /// The phases that must complete before this node can execute.
    pub dependencies: Vec<Phase>,
    /// The closure that performs the phase logic.
    ///
    /// Receives a shared reference to the [`CompilationContext`] and returns
    /// either the phase output or a list of fatal diagnostics.
    pub run: PhaseFn,
}

impl std::fmt::Debug for CompilationNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompilationNode")
            .field("phase", &self.phase)
            .field("dependencies", &self.dependencies)
            .field("run", &format_args!("Fn(...)"))
            .finish()
    }
}

impl CompilationNode {
    /// Creates a new compilation node.
    pub fn new(
        phase: Phase,
        dependencies: Vec<Phase>,
        run: impl Fn(&CompilationContext) -> Result<PhaseOutput, Vec<Diagnostic>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            phase,
            dependencies,
            run: Box::new(run),
        }
    }
}

/// A directed acyclic graph of compilation phases.
///
/// Supports sequential execution via [`execute`](Self::execute) and parallel
/// execution via [`execute_parallel`](Self::execute_parallel) using level-based
/// scheduling.
pub struct CompilationDag {
    /// Registered nodes keyed by phase.
    nodes: HashMap<Phase, CompilationNode>,
}

impl CompilationDag {
    /// Creates an empty DAG.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Adds a node to the DAG.
    ///
    /// # Errors
    /// Returns an error if a node with the same phase already exists.
    pub fn add_node(&mut self, node: CompilationNode) -> Result<(), String> {
        let phase = node.phase;
        if self.nodes.contains_key(&phase) {
            return Err(format!("Phase {:?} already registered", phase));
        }
        self.nodes.insert(phase, node);
        Ok(())
    }

    /// Returns the topological order of phases.
    ///
    /// Uses Kahn's algorithm. Returns an error if the graph contains a cycle
    /// (i.e., a topological order cannot be produced).
    ///
    /// # Errors
    /// Returns an error if the dependency graph contains a cycle.
    pub fn topological_order(&self) -> Result<Vec<Phase>, String> {
        // Build in-degree map and adjacency list
        let mut in_degree: HashMap<Phase, usize> = HashMap::new();
        let mut adjacency: HashMap<Phase, Vec<Phase>> = HashMap::new();

        for (phase, node) in &self.nodes {
            in_degree.entry(*phase).or_insert(0);
            adjacency.entry(*phase).or_default();

            for dep in &node.dependencies {
                if !self.nodes.contains_key(dep) {
                    return Err(format!(
                        "Phase {:?} depends on {:?} which is not registered",
                        phase, dep
                    ));
                }
                adjacency.entry(*dep).or_default().push(*phase);
                *in_degree.entry(*phase).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<Phase> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&phase, _)| phase)
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());

        while let Some(phase) = queue.pop_front() {
            order.push(phase);

            if let Some(neighbours) = adjacency.get(&phase) {
                for &next in neighbours {
                    if let Some(deg) = in_degree.get_mut(&next) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(next);
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err("Cycle detected in compilation DAG".to_string());
        }

        Ok(order)
    }

    /// Computes the level (longest-path distance from any source) for each
    /// phase, and groups phases by level.
    ///
    /// # Errors
    /// Propagates errors from [`topological_order`](Self::topological_order).
    fn compute_levels(&self) -> Result<Vec<Vec<Phase>>, String> {
        let order = self.topological_order()?;
        let mut phase_to_level: HashMap<Phase, usize> = HashMap::new();
        let mut levels: Vec<Vec<Phase>> = Vec::new();

        for &phase in &order {
            let node = &self.nodes[&phase];
            let level = node
                .dependencies
                .iter()
                .filter_map(|dep| phase_to_level.get(dep))
                .max()
                .map_or(0, |&l| l + 1);

            phase_to_level.insert(phase, level);

            while levels.len() <= level {
                levels.push(Vec::new());
            }
            levels[level].push(phase);
        }

        Ok(levels)
    }

    /// Executes all phases in topological order **sequentially**.
    ///
    /// # Errors
    /// Returns the first batch of fatal diagnostics if any phase fails.
    /// Phases after the failing one are not executed.
    pub fn execute(&self, context: &mut CompilationContext) -> Result<(), Vec<Diagnostic>> {
        let order = self.topological_order().map_err(|e| {
            vec![Diagnostic::new(
                Severity::Error,
                motarjim_diag::DiagnosticCode::new(999, "Dag error"),
                e,
            )]
        })?;

        for phase in &order {
            context.cancel_token.check().map_err(|_| {
                vec![Diagnostic::new(
                    Severity::Error,
                    motarjim_diag::DiagnosticCode::new(700, "Cancelled"),
                    "Compilation cancelled",
                )]
            })?;

            let node = &self.nodes[phase];
            let output = (node.run)(context)?;
            context.inputs.insert(*phase, output);
        }

        Ok(())
    }

    /// Executes phases using **level-based parallel scheduling**.
    ///
    /// Phases at the same level (no transitive dependencies) execute
    /// concurrently via [`rayon`]. Execution proceeds level-by-level
    /// sequentially.
    ///
    /// # Errors
    /// Collects all fatal diagnostics and returns early if any phase
    /// in a level fails.
    pub fn execute_parallel(
        &self,
        context: &mut CompilationContext,
    ) -> Result<(), Vec<Diagnostic>> {
        let levels = self.compute_levels().map_err(|e| {
            vec![Diagnostic::new(
                Severity::Error,
                motarjim_diag::DiagnosticCode::new(999, "Dag error"),
                e,
            )]
        })?;

        let mut all_errors: Vec<Diagnostic> = Vec::new();

        for level in &levels {
            if level.is_empty() {
                continue;
            }

            context.cancel_token.check().map_err(|_| {
                vec![Diagnostic::new(
                    Severity::Error,
                    motarjim_diag::DiagnosticCode::new(700, "Cancelled"),
                    "Compilation cancelled",
                )]
            })?;

            // Execute this level's phases in parallel.
            // Each phase receives a shared reference to the context but
            // returns its output; we write outputs back after joining.
            let results: Vec<(Phase, Result<PhaseOutput, Vec<Diagnostic>>)> = level
                .par_iter()
                .map(|phase| {
                    let node = &self.nodes[phase];
                    let result = (node.run)(context);
                    (*phase, result)
                })
                .collect();

            // Collect results and write outputs
            for (phase, result) in results {
                match result {
                    Ok(output) => {
                        context.inputs.insert(phase, output);
                    }
                    Err(diags) => {
                        all_errors.extend(diags);
                    }
                }
            }

            // Stop on first level with errors
            if !all_errors.is_empty() {
                return Err(all_errors);
            }
        }

        Ok(())
    }

    /// Returns the number of registered nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if no nodes are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Builds the default compilation DAG for the standard pipeline.
    ///
    /// All 13 phases are registered with their full dependency chains.
    /// The `compiler` parameter is reserved for future use (e.g., plugin
    /// system integration, configuration overrides).
    ///
    /// # Panics
    /// This method uses `unwrap` internally because the hardcoded nodes
    /// are guaranteed not to conflict and their dependencies are guaranteed
    /// to be registered.
    #[allow(clippy::unwrap_used, clippy::too_many_lines)]
    pub fn build_default() -> Self {
        let mut dag = Self::new();

        // --- Level 0: no dependencies ---

        // ReadFiles: a no-op in the default setup (sources are already in context).
        dag.add_node(CompilationNode::new(Phase::ReadFiles, vec![], |ctx| {
            ctx.cancel_token.check().map_err(|_| {
                vec![Diagnostic::new(
                    Severity::Error,
                    motarjim_diag::DiagnosticCode::new(700, "Cancelled"),
                    "Compilation cancelled",
                )]
            })?;
            Ok(PhaseOutput::Code(ctx.html_source.clone()))
        }))
        .unwrap();

        // ParseHtml
        dag.add_node(CompilationNode::new(Phase::ParseHtml, vec![], |ctx| {
            let mut parser = motarjim_parser::HtmlParser::new(&ctx.html_source);
            let doc = parser.parse()?;
            Ok(PhaseOutput::Document(doc))
        }))
        .unwrap();

        // ParseCss
        dag.add_node(CompilationNode::new(Phase::ParseCss, vec![], |ctx| {
            if let Some(ref css_source) = ctx.css_source {
                let mut parser = motarjim_parser::CssParser::new(css_source);
                let sheet = parser.parse()?;
                Ok(PhaseOutput::Stylesheet(sheet))
            } else {
                Ok(PhaseOutput::Stylesheet(CssStylesheet {
                    rules: vec![],
                    source_path: None,
                }))
            }
        }))
        .unwrap();

        // --- Level 1: depends on ParseHtml + ParseCss ---

        // CssSelectorMatching: match CSS selectors to DOM elements.
        dag.add_node(CompilationNode::new(
            Phase::CssSelectorMatching,
            vec![Phase::ParseHtml, Phase::ParseCss],
            |ctx| {
                let doc = ctx
                    .inputs
                    .get(&Phase::ParseHtml)
                    .and_then(|o| o.as_document())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "ParseHtml output not found",
                        )]
                    })?;
                let sheet = ctx
                    .inputs
                    .get(&Phase::ParseCss)
                    .and_then(|o| o.as_stylesheet());

                let mut resolver = motarjim_css::StyleResolver::new();
                if let Some(s) = sheet {
                    resolver.add_stylesheet(s.clone());
                }

                let styled_nodes: Vec<StyledNode> = doc
                    .nodes
                    .iter()
                    .map(|node| {
                        let style = node
                            .element
                            .as_ref()
                            .map(|el| resolver.resolve(el).style)
                            .unwrap_or_default();
                        StyledNode {
                            node: node.clone(),
                            computed_style: style,
                        }
                    })
                    .collect();

                Ok(PhaseOutput::StyledDocument(StyledDocument {
                    nodes: styled_nodes,
                    root_id: doc.root_id,
                }))
            },
        ))
        .unwrap();

        // --- Level 2: depends on CssSelectorMatching ---

        // CascadeStyles: resolve the cascade for computed styles.
        // In the current setup this is the same as CssSelectorMatching
        // because the StyleResolver already cascades.  This phase exists
        // as a separate node for future refinement.
        dag.add_node(CompilationNode::new(
            Phase::CascadeStyles,
            vec![Phase::CssSelectorMatching],
            |ctx| {
                let styled = ctx
                    .inputs
                    .get(&Phase::CssSelectorMatching)
                    .and_then(|o| o.as_styled_document())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "CssSelectorMatching output not found",
                        )]
                    })?;

                // Build a style map from the styled document.
                let style_map: HashMap<NodeId, ComputedStyle> = styled
                    .nodes
                    .iter()
                    .map(|sn| (sn.node.id, sn.computed_style.clone()))
                    .collect();

                Ok(PhaseOutput::StyleMap(style_map))
            },
        ))
        .unwrap();

        // --- Level 3: all depend on CascadeStyles, run in parallel ---

        // SemanticInference: annotate nodes with semantic roles.
        dag.add_node(CompilationNode::new(
            Phase::SemanticInference,
            vec![Phase::CascadeStyles],
            |ctx| {
                let doc = ctx
                    .inputs
                    .get(&Phase::ParseHtml)
                    .and_then(|o| o.as_document())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "ParseHtml output not found",
                        )]
                    })?;

                let analyzer = motarjim_ir::SemanticAnalyzer::new();
                for node in &doc.nodes {
                    let _semantic = analyzer.infer(node);
                }

                // Return the style map unchanged — semantic analysis is
                // consumed by BuildIr via `IrBuilder`.
                let style_map = ctx
                    .inputs
                    .get(&Phase::CascadeStyles)
                    .and_then(|o| o.as_style_map())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "CascadeStyles output not found",
                        )]
                    })?;

                Ok(PhaseOutput::StyleMap(style_map.clone()))
            },
        ))
        .unwrap();

        // LayoutInference: infer layout strategies.
        dag.add_node(CompilationNode::new(
            Phase::LayoutInference,
            vec![Phase::CascadeStyles],
            |ctx| {
                let style_map = ctx
                    .inputs
                    .get(&Phase::CascadeStyles)
                    .and_then(|o| o.as_style_map())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "CascadeStyles output not found",
                        )]
                    })?;

                let inferrer = motarjim_ir::LayoutInferrer::new();
                for style in style_map.values() {
                    let _layout = inferrer.infer(style);
                }

                Ok(PhaseOutput::StyleMap(style_map.clone()))
            },
        ))
        .unwrap();

        // AccessibilityAnalysis: extract ARIA metadata.
        dag.add_node(CompilationNode::new(
            Phase::AccessibilityAnalysis,
            vec![Phase::CascadeStyles],
            |ctx| {
                let doc = ctx
                    .inputs
                    .get(&Phase::ParseHtml)
                    .and_then(|o| o.as_document())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "ParseHtml output not found",
                        )]
                    })?;

                let inferrer = motarjim_ir::AccessibilityInferrer::new();
                for node in &doc.nodes {
                    let _a11y = inferrer.infer(node);
                }

                let style_map = ctx
                    .inputs
                    .get(&Phase::CascadeStyles)
                    .and_then(|o| o.as_style_map())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "CascadeStyles output not found",
                        )]
                    })?;

                Ok(PhaseOutput::StyleMap(style_map.clone()))
            },
        ))
        .unwrap();

        // --- Level 4: depends on Semantic + Layout + Accessibility ---

        // BuildIr: combine everything into an IR tree.
        dag.add_node(CompilationNode::new(
            Phase::BuildIr,
            vec![
                Phase::SemanticInference,
                Phase::LayoutInference,
                Phase::AccessibilityAnalysis,
            ],
            |ctx| {
                let doc = ctx
                    .inputs
                    .get(&Phase::ParseHtml)
                    .and_then(|o| o.as_document())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "ParseHtml output not found",
                        )]
                    })?;
                let style_map = ctx
                    .inputs
                    .get(&Phase::CascadeStyles)
                    .and_then(|o| o.as_style_map())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "CascadeStyles output not found",
                        )]
                    })?;

                let builder = motarjim_ir::IrBuilder::new();
                let diag = DiagnosticBag::new();
                let ir = builder.build(doc, style_map, &diag);

                // Transfer any diagnostics from the IR builder
                // (currently `diag` is always empty, but forward it anyway).
                if diag.has_errors() {
                    return Err(diag.into_diagnostics());
                }

                Ok(PhaseOutput::IrTree(ir))
            },
        ))
        .unwrap();

        // --- Level 5: depends on BuildIr ---

        // OptimizeIr: run optimisation passes.
        dag.add_node(CompilationNode::new(
            Phase::OptimizeIr,
            vec![Phase::BuildIr],
            |ctx| {
                let ir = ctx
                    .inputs
                    .get(&Phase::BuildIr)
                    .and_then(|o| o.as_ir_tree())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "BuildIr output not found",
                        )]
                    })?;

                let mut mutable_ir = ir.clone();
                let mut pass_manager = motarjim_optimizer::PassManager::new();
                motarjim_optimizer::register_default_passes(&mut pass_manager);
                pass_manager.run_all(&mut mutable_ir);

                Ok(PhaseOutput::IrTree(mutable_ir))
            },
        ))
        .unwrap();

        // --- Level 6: all depend on OptimizeIr, run in parallel ---

        // GenerateFlutter
        dag.add_node(CompilationNode::new(
            Phase::GenerateFlutter,
            vec![Phase::OptimizeIr],
            |ctx| {
                let ir = ctx
                    .inputs
                    .get(&Phase::OptimizeIr)
                    .and_then(|o| o.as_ir_tree())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "OptimizeIr output not found",
                        )]
                    })?;

                let gen = motarjim_gen_flutter::FlutterGenerator::new();
                let code = gen.generate(ir);
                Ok(PhaseOutput::Code(code))
            },
        ))
        .unwrap();

        // GenerateCompose
        dag.add_node(CompilationNode::new(
            Phase::GenerateCompose,
            vec![Phase::OptimizeIr],
            |ctx| {
                let ir = ctx
                    .inputs
                    .get(&Phase::OptimizeIr)
                    .and_then(|o| o.as_ir_tree())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "OptimizeIr output not found",
                        )]
                    })?;

                let gen = motarjim_gen_compose::ComposeGenerator::new();
                let code = gen.generate(ir);
                Ok(PhaseOutput::Code(code))
            },
        ))
        .unwrap();

        // GenerateSwiftui
        dag.add_node(CompilationNode::new(
            Phase::GenerateSwiftui,
            vec![Phase::OptimizeIr],
            |ctx| {
                let ir = ctx
                    .inputs
                    .get(&Phase::OptimizeIr)
                    .and_then(|o| o.as_ir_tree())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing input"),
                            "OptimizeIr output not found",
                        )]
                    })?;

                let gen = motarjim_gen_swiftui::SwiftUIGenerator::new();
                let code = gen.generate(ir);
                Ok(PhaseOutput::Code(code))
            },
        ))
        .unwrap();

        dag
    }
}

impl Default for CompilationDag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Test helpers
    // ------------------------------------------------------------------

    fn make_ok_node(phase: Phase, deps: Vec<Phase>) -> CompilationNode {
        CompilationNode::new(phase.clone(), deps, move |ctx| {
            ctx.cancel_token.check().map_err(|_| {
                vec![Diagnostic::new(
                    Severity::Error,
                    motarjim_diag::DiagnosticCode::new(700, "Cancelled"),
                    "cancelled",
                )]
            })?;
            Ok(PhaseOutput::Code(format!("{:?} done", phase)))
        })
    }

    fn make_failing_node(phase: Phase, deps: Vec<Phase>) -> CompilationNode {
        CompilationNode::new(phase.clone(), deps, move |_| {
            Err(vec![Diagnostic::new(
                Severity::Error,
                motarjim_diag::DiagnosticCode::new(1, "Test error"),
                format!("{:?} failed", phase),
            )])
        })
    }

    fn empty_context() -> CompilationContext {
        CompilationContext::new(String::new(), None)
    }

    // ------------------------------------------------------------------
    // Topological order tests
    // ------------------------------------------------------------------

    #[test]
    fn test_topological_order_single_node() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![]))
            .unwrap();
        let order = dag.topological_order().unwrap();
        assert_eq!(order, vec![Phase::ParseHtml]);
    }

    #[test]
    fn test_topological_order_chain() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![]))
            .unwrap();
        dag.add_node(make_ok_node(Phase::BuildIr, vec![Phase::ParseHtml]))
            .unwrap();
        let order = dag.topological_order().unwrap();
        assert_eq!(order, vec![Phase::ParseHtml, Phase::BuildIr]);
    }

    #[test]
    fn test_topological_order_diamond() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![]))
            .unwrap();
        dag.add_node(make_ok_node(Phase::ParseCss, vec![])).unwrap();
        dag.add_node(make_ok_node(
            Phase::CascadeStyles,
            vec![Phase::ParseHtml, Phase::ParseCss],
        ))
        .unwrap();
        let order = dag.topological_order().unwrap();
        // Both ParseHtml and ParseCss must come before CascadeStyles
        assert!(order[..2].contains(&Phase::ParseHtml));
        assert!(order[..2].contains(&Phase::ParseCss));
        assert_eq!(order[2], Phase::CascadeStyles);
    }

    #[test]
    fn test_topological_order_cycle_detected() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![Phase::BuildIr]))
            .unwrap();
        dag.add_node(make_ok_node(Phase::BuildIr, vec![Phase::ParseHtml]))
            .unwrap();
        assert!(dag.topological_order().is_err());
    }

    #[test]
    fn test_topological_order_missing_dependency() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::BuildIr, vec![Phase::ParseHtml]))
            .unwrap();
        assert!(dag.topological_order().is_err());
    }

    // ------------------------------------------------------------------
    // Level computation tests
    // ------------------------------------------------------------------

    #[test]
    fn test_compute_levels_simple() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![]))
            .unwrap();
        dag.add_node(make_ok_node(Phase::ParseCss, vec![])).unwrap();
        dag.add_node(make_ok_node(
            Phase::CascadeStyles,
            vec![Phase::ParseHtml, Phase::ParseCss],
        ))
        .unwrap();
        let levels = dag.compute_levels().unwrap();
        assert_eq!(levels.len(), 2);
        assert!(levels[0].contains(&Phase::ParseHtml));
        assert!(levels[0].contains(&Phase::ParseCss));
        assert_eq!(levels[1], vec![Phase::CascadeStyles]);
    }

    #[test]
    fn test_compute_levels_multi_level() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![]))
            .unwrap();
        dag.add_node(make_ok_node(
            Phase::CssSelectorMatching,
            vec![Phase::ParseHtml],
        ))
        .unwrap();
        dag.add_node(make_ok_node(
            Phase::CascadeStyles,
            vec![Phase::CssSelectorMatching],
        ))
        .unwrap();
        dag.add_node(make_ok_node(Phase::BuildIr, vec![Phase::CascadeStyles]))
            .unwrap();
        let levels = dag.compute_levels().unwrap();
        assert_eq!(levels.len(), 4);
        assert_eq!(levels[0], vec![Phase::ParseHtml]);
        assert_eq!(levels[1], vec![Phase::CssSelectorMatching]);
        assert_eq!(levels[2], vec![Phase::CascadeStyles]);
        assert_eq!(levels[3], vec![Phase::BuildIr]);
    }

    // ------------------------------------------------------------------
    // Sequential execution tests
    // ------------------------------------------------------------------

    #[test]
    fn test_execute_single_node() {
        let mut dag = CompilationDag::new();
        dag.add_node(CompilationNode::new(Phase::ParseHtml, vec![], |_| {
            Ok(PhaseOutput::Code("hello".to_string()))
        }))
        .unwrap();
        let mut ctx = empty_context();
        dag.execute(&mut ctx).unwrap();
        assert_eq!(
            ctx.inputs.get(&Phase::ParseHtml).and_then(|o| o.as_code()),
            Some("hello")
        );
    }

    #[test]
    fn test_execute_chain() {
        let mut dag = CompilationDag::new();
        dag.add_node(CompilationNode::new(Phase::ParseHtml, vec![], |_| {
            Ok(PhaseOutput::Code("parsed".to_string()))
        }))
        .unwrap();
        dag.add_node(CompilationNode::new(
            Phase::BuildIr,
            vec![Phase::ParseHtml],
            |ctx| {
                let input = ctx
                    .inputs
                    .get(&Phase::ParseHtml)
                    .and_then(|o| o.as_code())
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Severity::Error,
                            motarjim_diag::DiagnosticCode::new(999, "Missing"),
                            "",
                        )]
                    })?;
                Ok(PhaseOutput::Code(format!("built from {}", input)))
            },
        ))
        .unwrap();
        let mut ctx = empty_context();
        dag.execute(&mut ctx).unwrap();
        let output = ctx.inputs.get(&Phase::BuildIr).and_then(|o| o.as_code());
        assert_eq!(output, Some("built from parsed"));
    }

    #[test]
    fn test_execute_failing_phase_returns_error() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![]))
            .unwrap();
        dag.add_node(make_failing_node(Phase::BuildIr, vec![Phase::ParseHtml]))
            .unwrap();
        let mut ctx = empty_context();
        let result = dag.execute(&mut ctx);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|d| d.message.contains("BuildIr failed")));
    }

    #[test]
    fn test_execute_aborts_after_failure() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_failing_node(Phase::ParseHtml, vec![]))
            .unwrap();
        dag.add_node(make_ok_node(Phase::BuildIr, vec![Phase::ParseHtml]))
            .unwrap();
        let mut ctx = empty_context();
        assert!(dag.execute(&mut ctx).is_err());
        // BuildIr should NOT execute because ParseHtml failed
        assert!(ctx.inputs.get(&Phase::BuildIr).is_none());
    }

    // ------------------------------------------------------------------
    // Parallel execution tests
    // ------------------------------------------------------------------

    #[test]
    fn test_execute_parallel_single_level() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![]))
            .unwrap();
        dag.add_node(make_ok_node(Phase::ParseCss, vec![])).unwrap();
        let mut ctx = empty_context();
        dag.execute_parallel(&mut ctx).unwrap();
        assert!(ctx.inputs.contains_key(&Phase::ParseHtml));
        assert!(ctx.inputs.contains_key(&Phase::ParseCss));
    }

    #[test]
    fn test_execute_parallel_multi_level() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![]))
            .unwrap();
        dag.add_node(make_ok_node(Phase::ParseCss, vec![])).unwrap();
        dag.add_node(make_ok_node(
            Phase::CascadeStyles,
            vec![Phase::ParseHtml, Phase::ParseCss],
        ))
        .unwrap();
        let mut ctx = empty_context();
        dag.execute_parallel(&mut ctx).unwrap();
        assert!(ctx.inputs.contains_key(&Phase::ParseHtml));
        assert!(ctx.inputs.contains_key(&Phase::ParseCss));
        assert!(ctx.inputs.contains_key(&Phase::CascadeStyles));
    }

    #[test]
    fn test_execute_parallel_error_stops_level() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_failing_node(Phase::ParseHtml, vec![]))
            .unwrap();
        dag.add_node(make_ok_node(Phase::ParseCss, vec![])).unwrap();
        let mut ctx = empty_context();
        let result = dag.execute_parallel(&mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_parallel_independent_phases_run_concurrently() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let counter = Arc::new(AtomicUsize::new(0));

        let mut dag = CompilationDag::new();

        let c1 = Arc::clone(&counter);
        dag.add_node(CompilationNode::new(
            Phase::SemanticInference,
            vec![],
            move |_| {
                c1.fetch_add(1, Ordering::SeqCst);
                Ok(PhaseOutput::Code("sem".to_string()))
            },
        ))
        .unwrap();

        let c2 = Arc::clone(&counter);
        dag.add_node(CompilationNode::new(
            Phase::LayoutInference,
            vec![],
            move |_| {
                c2.fetch_add(1, Ordering::SeqCst);
                Ok(PhaseOutput::Code("lay".to_string()))
            },
        ))
        .unwrap();

        let mut ctx = empty_context();
        dag.execute_parallel(&mut ctx).unwrap();
        // Both should have executed
        assert!(ctx.inputs.contains_key(&Phase::SemanticInference));
        assert!(ctx.inputs.contains_key(&Phase::LayoutInference));
    }

    // ------------------------------------------------------------------
    // Default DAG tests
    // ------------------------------------------------------------------

    #[test]
    fn test_default_dag_has_all_phases() {
        let dag = CompilationDag::build_default();
        assert_eq!(dag.len(), 13);
        assert!(dag.nodes.contains_key(&Phase::ReadFiles));
        assert!(dag.nodes.contains_key(&Phase::ParseHtml));
        assert!(dag.nodes.contains_key(&Phase::ParseCss));
        assert!(dag.nodes.contains_key(&Phase::CssSelectorMatching));
        assert!(dag.nodes.contains_key(&Phase::CascadeStyles));
        assert!(dag.nodes.contains_key(&Phase::SemanticInference));
        assert!(dag.nodes.contains_key(&Phase::LayoutInference));
        assert!(dag.nodes.contains_key(&Phase::AccessibilityAnalysis));
        assert!(dag.nodes.contains_key(&Phase::BuildIr));
        assert!(dag.nodes.contains_key(&Phase::OptimizeIr));
        assert!(dag.nodes.contains_key(&Phase::GenerateFlutter));
        assert!(dag.nodes.contains_key(&Phase::GenerateCompose));
        assert!(dag.nodes.contains_key(&Phase::GenerateSwiftui));
    }

    #[test]
    fn test_default_dag_topological_order() {
        let dag = CompilationDag::build_default();
        let order = dag.topological_order().unwrap();
        assert_eq!(order.len(), 13);

        // Verify dependency ordering: each phase must appear after its deps
        for (phase, node) in &dag.nodes {
            for dep in &node.dependencies {
                let phase_pos = order.iter().position(|p| p == phase).unwrap();
                let dep_pos = order.iter().position(|p| p == dep).unwrap();
                assert!(
                    dep_pos < phase_pos,
                    "{:?} must come before {:?}",
                    dep,
                    phase
                );
            }
        }
    }

    #[test]
    fn test_default_dag_levels() {
        let dag = CompilationDag::build_default();
        let levels = dag.compute_levels().unwrap();

        // Level 0: ReadFiles, ParseHtml, ParseCss
        assert!(levels[0].contains(&Phase::ReadFiles));
        assert!(levels[0].contains(&Phase::ParseHtml));
        assert!(levels[0].contains(&Phase::ParseCss));

        // Level 1: CssSelectorMatching
        assert!(levels[1].contains(&Phase::CssSelectorMatching));

        // Level 2: CascadeStyles
        assert!(levels[2].contains(&Phase::CascadeStyles));

        // Level 3: SemanticInference, LayoutInference, AccessibilityAnalysis
        assert!(levels[3].contains(&Phase::SemanticInference));
        assert!(levels[3].contains(&Phase::LayoutInference));
        assert!(levels[3].contains(&Phase::AccessibilityAnalysis));

        // Level 4: BuildIr
        assert!(levels[4].contains(&Phase::BuildIr));

        // Level 5: OptimizeIr
        assert!(levels[5].contains(&Phase::OptimizeIr));

        // Level 6: GenerateFlutter, GenerateCompose, GenerateSwiftui
        assert!(levels[6].contains(&Phase::GenerateFlutter));
        assert!(levels[6].contains(&Phase::GenerateCompose));
        assert!(levels[6].contains(&Phase::GenerateSwiftui));
    }

    #[test]
    fn test_default_dag_execute() {
        let dag = CompilationDag::build_default();
        let mut ctx = CompilationContext::new(
            "<div>Hello</div>".to_string(),
            Some("div { color: red; }".to_string()),
        );
        let result = dag.execute(&mut ctx);
        assert!(result.is_ok(), "DAG execution failed: {:?}", result);
        // Verify outputs exist for key phases
        assert!(ctx.inputs.contains_key(&Phase::ParseHtml));
        assert!(ctx.inputs.contains_key(&Phase::ParseCss));
        assert!(ctx.inputs.contains_key(&Phase::CssSelectorMatching));
        assert!(ctx.inputs.contains_key(&Phase::CascadeStyles));
        assert!(ctx.inputs.contains_key(&Phase::BuildIr));
        assert!(ctx.inputs.contains_key(&Phase::OptimizeIr));
        // Generation outputs
        assert!(ctx.inputs.contains_key(&Phase::GenerateFlutter));
        assert!(ctx.inputs.contains_key(&Phase::GenerateCompose));
        assert!(ctx.inputs.contains_key(&Phase::GenerateSwiftui));

        // Verify generated code is non-empty
        let flutter_code = ctx
            .inputs
            .get(&Phase::GenerateFlutter)
            .and_then(|o| o.as_code());
        assert!(flutter_code.is_some());
        assert!(
            !flutter_code.unwrap().is_empty(),
            "Flutter code should not be empty"
        );
    }

    #[test]
    fn test_default_dag_execute_parallel() {
        let dag = CompilationDag::build_default();
        let mut ctx = CompilationContext::new(
            "<div>Hello</div>".to_string(),
            Some("div { color: red; }".to_string()),
        );
        // Note: parallel execution requires all phase closures to be Sync,
        // which they are because they only capture Send + Sync data.
        let result = dag.execute_parallel(&mut ctx);
        assert!(
            result.is_ok(),
            "Parallel DAG execution failed: {:?}",
            result
        );
        assert!(ctx.inputs.contains_key(&Phase::GenerateFlutter));
        assert!(ctx.inputs.contains_key(&Phase::GenerateCompose));
        assert!(ctx.inputs.contains_key(&Phase::GenerateSwiftui));
    }

    // ------------------------------------------------------------------
    // Edge-case tests
    // ------------------------------------------------------------------

    #[test]
    fn test_empty_dag_execute_succeeds() {
        let dag = CompilationDag::new();
        let mut ctx = empty_context();
        assert!(dag.execute(&mut ctx).is_ok());
    }

    #[test]
    fn test_empty_dag_execute_parallel_succeeds() {
        let dag = CompilationDag::new();
        let mut ctx = empty_context();
        assert!(dag.execute_parallel(&mut ctx).is_ok());
    }

    #[test]
    fn test_duplicate_node_returns_error() {
        let mut dag = CompilationDag::new();
        dag.add_node(make_ok_node(Phase::ParseHtml, vec![]))
            .unwrap();
        let result = dag.add_node(make_ok_node(Phase::ParseHtml, vec![]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already registered"));
    }

    #[test]
    fn test_cancellation_stops_execution() {
        let mut dag = CompilationDag::new();
        let token = CancelToken::new();
        let token_clone = token.clone();

        dag.add_node(CompilationNode::new(Phase::ParseHtml, vec![], move |ctx| {
            // Cancel before this phase runs
            token_clone.cancel();
            ctx.cancel_token.check().map_err(|_| {
                vec![Diagnostic::new(
                    Severity::Error,
                    motarjim_diag::DiagnosticCode::new(700, "Cancelled"),
                    "cancelled",
                )]
            })?;
            Ok(PhaseOutput::Code("done".to_string()))
        }))
        .unwrap();

        dag.add_node(CompilationNode::new(
            Phase::BuildIr,
            vec![Phase::ParseHtml],
            |ctx| {
                ctx.cancel_token.check().map_err(|_| {
                    vec![Diagnostic::new(
                        Severity::Error,
                        motarjim_diag::DiagnosticCode::new(700, "Cancelled"),
                        "cancelled",
                    )]
                })?;
                Ok(PhaseOutput::Code("done".to_string()))
            },
        ))
        .unwrap();

        let mut ctx = CompilationContext::new("<div></div>".to_string(), None);
        ctx.cancel_token = token;
        let result = dag.execute(&mut ctx);
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------
    // PhaseOutput accessor tests
    // ------------------------------------------------------------------

    #[test]
    fn test_phase_output_document_accessors() {
        let doc = Document::new();
        let output = PhaseOutput::Document(doc);
        assert!(output.as_document().is_some());
        assert!(output.as_stylesheet().is_none());
        assert!(output.as_code().is_none());
    }

    #[test]
    fn test_phase_output_code_accessor() {
        let output = PhaseOutput::Code("test".to_string());
        assert_eq!(output.as_code(), Some("test"));
        assert!(output.as_document().is_none());
    }

    // ------------------------------------------------------------------
    // CancelToken tests
    // ------------------------------------------------------------------

    #[test]
    fn test_cancel_token_default_not_cancelled() {
        let token = CancelToken::new();
        assert!(!token.is_cancelled());
        assert!(token.check().is_ok());
    }

    #[test]
    fn test_cancel_token_cancel() {
        let token = CancelToken::new();
        token.cancel();
        assert!(token.is_cancelled());
        assert!(token.check().is_err());
    }

    #[test]
    fn test_cancel_token_clone_shares_state() {
        let t1 = CancelToken::new();
        let t2 = t1.clone();
        t1.cancel();
        assert!(t2.is_cancelled());
    }
}
