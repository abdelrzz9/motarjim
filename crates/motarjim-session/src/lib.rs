#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Centralised compiler context for the Motarjim compiler.
//!
//! The [`Session`] struct owns all compiler-wide state — configuration,
//! diagnostics, source map, filesystem, caching, profiling, and
//! incremental compilation — so that pipeline phases receive a single
//! `&Session` reference instead of many disjoint objects.
//!
//! # Usage
//!
//! ```rust
//! use std::sync::Arc;
//! use motarjim_config::Config;
//! use motarjim_fs::VirtualFileSystem;
//! use motarjim_session::Session;
//!
//! let config = Config::new();
//! let fs: Arc<VirtualFileSystem> = Arc::new(VirtualFileSystem::new());
//! let session = Session::new(config, fs);
//! assert!(!session.has_errors());
//! ```

mod source_map;

pub use source_map::SourceMap;

use std::path::PathBuf;
use std::sync::Arc;

use motarjim_cache::ArtifactCache;
use motarjim_config::Config;
use motarjim_diag::{Diagnostic, DiagnosticBag};
use motarjim_fs::FileSystem;
use motarjim_incremental::IncrementalEngine;
use motarjim_profiling::{PhaseTimer, ProfilingSession};
use motarjim_span::SourceFile;

/// Error returned when a cancelled operation is detected.
///
/// Available only when the `cancellation` feature is enabled.
#[cfg(feature = "cancellation")]
#[derive(Debug, Clone)]
pub struct Cancelled {
    /// Human-readable reason for the cancellation.
    pub message: String,
}

#[cfg(feature = "cancellation")]
impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(feature = "cancellation")]
impl std::error::Error for Cancelled {}

/// A token that signals cancellation of a long-running operation.
///
/// Cloning shares the underlying cancellation state — cancelling any clone
/// cancels all of them.
///
/// Available only when the `cancellation` feature is enabled.
#[cfg(feature = "cancellation")]
#[derive(Clone)]
pub struct CancelToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(feature = "cancellation")]
impl CancelToken {
    /// Creates a new token that is **not** cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Returns `true` if [`cancel`](Self::cancel) has been called on any clone.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Requests cancellation. All clones of this token will now report
    /// [`is_cancelled`](Self::is_cancelled) as `true` and
    /// [`check`](Self::check) will return `Err`.
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Returns `Ok(())` if not cancelled, or `Err(Cancelled)` if cancelled.
    ///
    /// # Errors
    /// Returns [`Cancelled`] when [`cancel`](Self::cancel) has been called.
    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled {
                message: String::from("compilation cancelled"),
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "cancellation")]
impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Centralised compiler context that owns all compiler-wide state.
///
/// A single `Session` is created at the start of compilation and passed
/// by shared reference (`&Session`) to every pipeline stage. This replaces
/// the pattern of passing [`Config`], [`Arc<dyn FileSystem>`],
/// [`ProfilingSession`], etc. as separate arguments.
///
/// Fields that require mutation during compilation (diagnostics, source map,
/// profiling) use interior mutability so that all public methods take
/// `&self`.
pub struct Session {
    /// Compiler configuration.
    config: Config,
    /// Accumulated diagnostics (interior mutability).
    diagnostics: std::sync::Mutex<DiagnosticBag>,
    /// Registry of source files (interior mutability).
    source_map: std::sync::Mutex<SourceMap>,
    /// Abstract filesystem for I/O.
    file_system: Arc<dyn FileSystem>,
    /// Optional content-addressable artifact cache.
    cache: Option<ArtifactCache>,
    /// Optional incremental compilation engine.
    incremental: std::sync::Mutex<Option<IncrementalEngine>>,
    /// Performance profiling session (interior mutability).
    profiling: std::sync::Mutex<ProfilingSession>,
    /// Token for cooperative cancellation.
    #[cfg(feature = "cancellation")]
    cancel_token: CancelToken,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diag_len = self.diagnostics.lock().map_or(0, |b| b.len());
        let sm_len = self.source_map.lock().map_or(0, |m| m.len());
        let profiling_label = self
            .profiling
            .lock()
            .map_or_else(|_| "?".to_string(), |p| p.label().to_string());

        let mut d = f.debug_struct("Session");
        d.field("config", &self.config)
            .field(
                "diagnostics",
                &format_args!("Mutex<DiagnosticBag({diag_len})>"),
            )
            .field("source_map", &format_args!("Mutex<SourceMap({sm_len})>"))
            .field("cache", &self.cache)
            .field(
                "incremental",
                &self.incremental.lock().ok().map(|i| i.is_some()),
            )
            .field(
                "profiling",
                &format_args!("Mutex<ProfilingSession({profiling_label})>"),
            );
        #[cfg(feature = "cancellation")]
        {
            d.field("cancel_token", &self.cancel_token.is_cancelled());
        }
        d.finish()
    }
}

impl Session {
    /// Creates a new `Session` that owns all compiler-wide state.
    ///
    /// The cache directory and incremental engine are initialised from the
    /// [`Config`] values.
    #[must_use]
    pub fn new(config: Config, file_system: Arc<dyn FileSystem>) -> Self {
        let cache = config.global.cache_dir.clone().map(ArtifactCache::new);

        let incremental = if config.global.incremental {
            let dir = config
                .global
                .cache_dir
                .clone()
                .unwrap_or_else(|| PathBuf::from(".motarjim/cache"));
            std::sync::Mutex::new(Some(IncrementalEngine::new(dir.join("incremental"))))
        } else {
            std::sync::Mutex::new(None)
        };

        Self {
            config,
            diagnostics: std::sync::Mutex::new(DiagnosticBag::new()),
            source_map: std::sync::Mutex::new(SourceMap::new()),
            file_system,
            cache,
            incremental,
            profiling: std::sync::Mutex::new(ProfilingSession::new("compilation")),
            #[cfg(feature = "cancellation")]
            cancel_token: CancelToken::new(),
        }
    }

    // ------------------------------------------------------------------
    // Getters — immutable accessors
    // ------------------------------------------------------------------

    /// Returns a reference to the compiler configuration.
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a reference to the filesystem abstraction.
    #[must_use]
    pub fn file_system(&self) -> &Arc<dyn FileSystem> {
        &self.file_system
    }

    /// Returns a reference to the optional artifact cache.
    #[must_use]
    pub const fn cache(&self) -> &Option<ArtifactCache> {
        &self.cache
    }

    /// Returns a reference to the optional incremental engine.
    #[must_use]
    pub fn incremental(&self) -> std::sync::MutexGuard<'_, Option<IncrementalEngine>> {
        self.incremental.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Returns a mutable reference to the optional incremental engine.
    pub fn incremental_mut(&self) -> std::sync::MutexGuard<'_, Option<IncrementalEngine>> {
        self.incremental.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Returns a clone of the cancellation token for sharing with phases.
    #[cfg(feature = "cancellation")]
    #[must_use]
    pub fn cancel_token(&self) -> CancelToken {
        self.cancel_token.clone()
    }

    // ------------------------------------------------------------------
    // Profiling
    // ------------------------------------------------------------------

    /// Starts a timer for a named phase.
    ///
    /// The returned [`PhaseTimer`] should be stopped with
    /// [`record_phase`](Self::record_phase) once the phase completes.
    #[must_use]
    pub fn start_phase(&self, name: &'static str) -> PhaseTimer {
        PhaseTimer::new(name)
    }

    /// Records the duration of a completed phase.
    pub fn record_phase(&self, name: &'static str, duration: std::time::Duration) {
        if let Ok(mut profiling) = self.profiling.lock() {
            profiling.record_phase(name, duration);
        }
    }

    /// Returns a snapshot of the current profiling session.
    #[must_use]
    pub fn profiling(&self) -> ProfilingSession {
        self.profiling
            .lock()
            .map_or_else(|_| ProfilingSession::new("default"), |p| p.clone())
    }

    /// Increments a named counter in the profiling session.
    pub fn increment_counter(&self, name: &'static str, count: u64) {
        if let Ok(mut profiling) = self.profiling.lock() {
            profiling.increment_counter(name, count);
        }
    }

    // ------------------------------------------------------------------
    // Diagnostics
    // ------------------------------------------------------------------

    /// Emits a diagnostic into the session's diagnostic bag.
    pub fn emit(&self, diagnostic: Diagnostic) {
        if let Ok(mut bag) = self.diagnostics.lock() {
            bag.add(diagnostic);
        }
    }

    /// Emits multiple diagnostics at once.
    pub fn emit_many(&self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        if let Ok(mut bag) = self.diagnostics.lock() {
            bag.extend(diagnostics);
        }
    }

    /// Takes all accumulated diagnostics, leaving the internal bag empty.
    #[must_use]
    pub fn take_diagnostics(&self) -> DiagnosticBag {
        let mut guard = self.diagnostics.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut *guard)
    }

    /// Returns a clone of the current diagnostic bag (does **not** clear it).
    #[must_use]
    pub fn diagnostics(&self) -> DiagnosticBag {
        self.diagnostics
            .lock()
            .map_or_else(|_| DiagnosticBag::new(), |guard| guard.clone())
    }

    /// Returns `true` if any error-severity diagnostics have been emitted.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .lock()
            .ok()
            .is_some_and(|bag| bag.has_errors())
    }

    /// Returns the number of error-severity diagnostics.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.diagnostics.lock().map_or(0, |bag| bag.error_count())
    }

    // ------------------------------------------------------------------
    // Source Map
    // ------------------------------------------------------------------

    /// Registers a source file in the session's source map.
    ///
    /// If a file was already registered under the same path it is replaced.
    pub fn add_source_file(&self, file: SourceFile) {
        if let Ok(mut map) = self.source_map.lock() {
            map.add(file);
        }
    }

    /// Looks up a source file by path.
    #[must_use]
    pub fn get_source_file(&self, path: &std::path::Path) -> Option<SourceFile> {
        self.source_map.lock().ok()?.get(path).cloned()
    }

    /// Removes a source file from the map, returning it if it existed.
    pub fn remove_source_file(&self, path: &std::path::Path) -> Option<SourceFile> {
        self.source_map.lock().ok()?.remove(path)
    }

    /// Returns a snapshot of the current source map.
    #[must_use]
    pub fn source_map(&self) -> SourceMap {
        self.source_map
            .lock()
            .map_or_else(|_| SourceMap::new(), |guard| guard.clone())
    }

    // ------------------------------------------------------------------
    // Cancellation
    // ------------------------------------------------------------------

    /// Returns `true` if compilation has been cancelled.
    #[cfg(feature = "cancellation")]
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Requests cancellation of the current compilation.
    ///
    /// All phases that check [`is_cancelled`](Self::is_cancelled) will
    /// observe the cancellation and may abort early.
    #[cfg(feature = "cancellation")]
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_diag::Severity;
    use motarjim_fs::VirtualFileSystem;
    use std::path::Path;

    fn test_session() -> Session {
        let config = Config::new();
        let fs: Arc<VirtualFileSystem> = Arc::new(VirtualFileSystem::new());
        Session::new(config, fs)
    }

    #[test]
    fn test_session_creation() {
        let session = test_session();
        assert!(!session.has_errors());
        assert_eq!(session.error_count(), 0);
    }

    #[test]
    fn test_emit_diagnostic() {
        let session = test_session();
        let diag = Diagnostic::new(
            Severity::Error,
            motarjim_diag::DiagnosticCode::new(1, "Test"),
            "test error",
        );
        session.emit(diag);
        assert!(session.has_errors());
        assert_eq!(session.error_count(), 1);
    }

    #[test]
    fn test_take_diagnostics() {
        let session = test_session();
        session.emit(Diagnostic::new(
            Severity::Warning,
            motarjim_diag::DiagnosticCode::new(2, "Test"),
            "warning",
        ));
        let bag = session.take_diagnostics();
        assert_eq!(bag.len(), 1);
        assert!(!session.has_errors());
    }

    #[test]
    fn test_source_map() {
        let session = test_session();
        let sf = SourceFile::new("test.html", "<div>".to_string());
        session.add_source_file(sf);
        let retrieved = session.get_source_file(Path::new("test.html"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "<div>");
    }

    #[test]
    fn test_session_source_map_remove() {
        let session = test_session();
        let sf = SourceFile::new("test.html", "<div>".to_string());
        session.add_source_file(sf);
        let removed = session.remove_source_file(Path::new("test.html"));
        assert!(removed.is_some());
        assert!(session.get_source_file(Path::new("test.html")).is_none());
    }

    #[test]
    fn test_profiling() {
        let session = test_session();
        let mut timer = session.start_phase("test_phase");
        std::thread::sleep(std::time::Duration::from_millis(1));
        session.record_phase("test_phase", timer.stop());
        let profile = session.profiling();
        assert!(profile.phases().contains_key("test_phase"));
    }

    #[test]
    fn test_increment_counter() {
        let session = test_session();
        session.increment_counter("nodes", 42);
        let profile = session.profiling();
        assert_eq!(profile.counters().get("nodes"), Some(&42));
    }

    #[test]
    fn test_config_access() {
        let session = test_session();
        let config = session.config();
        assert_eq!(config.platforms.len(), 3);
    }

    #[test]
    fn test_file_system_access() {
        let session = test_session();
        let fs = session.file_system();
        assert!(!fs.exists(Path::new("nonexistent")));
    }

    #[test]
    fn test_source_map_snapshot() {
        let session = test_session();
        session.add_source_file(SourceFile::new("a.html", "a".to_string()));
        session.add_source_file(SourceFile::new("b.html", "b".to_string()));
        let snapshot = session.source_map();
        assert_eq!(snapshot.len(), 2);
    }

    #[test]
    fn test_source_map_new() {
        let sm = SourceMap::new();
        assert!(sm.is_empty());
        assert_eq!(sm.len(), 0);
    }

    #[test]
    fn test_source_map_add_and_get() {
        let mut sm = SourceMap::new();
        let sf = SourceFile::new("index.html", "<html>".to_string());
        sm.add(sf);
        let retrieved = sm.get(Path::new("index.html"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "<html>");
    }

    #[test]
    fn test_source_map_contains() {
        let mut sm = SourceMap::new();
        sm.add(SourceFile::new("index.html", "<html>".to_string()));
        assert!(sm.contains(Path::new("index.html")));
        assert!(!sm.contains(Path::new("missing.html")));
    }

    #[test]
    fn test_source_map_remove() {
        let mut sm = SourceMap::new();
        sm.add(SourceFile::new("index.html", "<html>".to_string()));
        let removed = sm.remove(Path::new("index.html"));
        assert!(removed.is_some());
        assert!(sm.is_empty());
    }

    #[test]
    fn test_source_map_extend() {
        let mut sm = SourceMap::new();
        let files = vec![
            SourceFile::new("a.html", "a".to_string()),
            SourceFile::new("b.html", "b".to_string()),
        ];
        sm.extend(files);
        assert_eq!(sm.len(), 2);
    }

    #[test]
    fn test_source_map_iter() {
        let mut sm = SourceMap::new();
        sm.add(SourceFile::new("a.html", "a".to_string()));
        sm.add(SourceFile::new("b.html", "b".to_string()));
        let paths: Vec<_> = sm
            .iter()
            .map(|sf| sf.path.as_os_str().to_os_string())
            .collect();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_source_map_default() {
        let sm = SourceMap::default();
        assert!(sm.is_empty());
    }

    #[test]
    fn test_emit_many() {
        let session = test_session();
        let diags = vec![
            Diagnostic::new(
                Severity::Error,
                motarjim_diag::DiagnosticCode::new(1, "E"),
                "err1",
            ),
            Diagnostic::new(
                Severity::Warning,
                motarjim_diag::DiagnosticCode::new(2, "W"),
                "warn1",
            ),
        ];
        session.emit_many(diags);
        assert!(session.has_errors());
        assert_eq!(session.error_count(), 1);
    }

    #[test]
    fn test_diagnostics_clone_does_not_clear() {
        let session = test_session();
        session.emit(Diagnostic::new(
            Severity::Error,
            motarjim_diag::DiagnosticCode::new(1, "Test"),
            "err",
        ));
        let _clone = session.diagnostics();
        assert!(session.has_errors());
    }

    #[test]
    fn test_session_debug_does_not_panic() {
        let session = test_session();
        let _debug = format!("{session:?}");
    }
}
