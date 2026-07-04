use crate::*;

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
