#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Performance profiling for the Motarjim compiler.
//!
//! Provides timing, counting, and reporting infrastructure for measuring
//! compiler performance across all phases.
//!
//! # Example
//!
//! ```rust
//! use motarjim_profiling::{PhaseTimer, ProfilingSession};
//!
//! let mut session = ProfilingSession::new("test");
//! let mut timer = session.start_phase("parse");
//! // ... do work ...
//! timer.stop();
//! let report = session.report();
//! assert!(!report.is_empty());
//! ```
//!
//! ## Feature flags
//!
//! - `trace` — Enables tracing subscriber integration.
//! - `flamegraph` — Enables flamegraph output generation.

use std::collections::HashMap;
use std::fmt::Write;
use std::time::{Duration, Instant};

/// A timer for a single compilation phase.
#[derive(Debug, Clone)]
pub struct PhaseTimer {
    /// The phase name.
    name: &'static str,
    /// The start instant.
    start: Instant,
    /// The accumulated duration (for paused/resumed timers).
    accumulated: Duration,
    /// Whether the timer is currently running.
    running: bool,
}

impl PhaseTimer {
    /// Creates a new started phase timer.
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            accumulated: Duration::ZERO,
            running: true,
        }
    }

    /// Stops the timer and returns the elapsed duration.
    pub fn stop(&mut self) -> Duration {
        if self.running {
            self.accumulated += self.start.elapsed();
            self.running = false;
        }
        self.accumulated
    }

    /// Pauses the timer without resetting.
    pub fn pause(&mut self) {
        if self.running {
            self.accumulated += self.start.elapsed();
            self.running = false;
        }
    }

    /// Resumes a paused timer.
    pub fn resume(&mut self) {
        if !self.running {
            self.start = Instant::now();
            self.running = true;
        }
    }

    /// Returns the phase name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the elapsed duration (without stopping).
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        if self.running {
            self.accumulated + self.start.elapsed()
        } else {
            self.accumulated
        }
    }
}

/// A profiling session that aggregates phase timings.
#[derive(Debug, Clone)]
pub struct ProfilingSession {
    /// Session label.
    label: String,
    /// Phase timings.
    phases: HashMap<&'static str, Duration>,
    /// Event counters.
    counters: HashMap<&'static str, u64>,
    /// Memory tracking.
    allocations: u64,
    /// Total bytes allocated.
    allocation_bytes: u64,
}

impl ProfilingSession {
    /// Creates a new profiling session with the given label.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            phases: HashMap::new(),
            counters: HashMap::new(),
            allocations: 0,
            allocation_bytes: 0,
        }
    }

    /// Starts a new phase timer.
    #[must_use]
    pub fn start_phase(&mut self, name: &'static str) -> PhaseTimer {
        PhaseTimer::new(name)
    }

    /// Records a completed phase timing.
    pub fn record_phase(&mut self, name: &'static str, duration: Duration) {
        self.phases.insert(name, duration);
    }

    /// Increments a named counter.
    pub fn increment_counter(&mut self, name: &'static str, count: u64) {
        *self.counters.entry(name).or_insert(0) += count;
    }

    /// Records an allocation.
    pub const fn record_allocation(&mut self, count: u64, bytes: u64) {
        self.allocations += count;
        self.allocation_bytes += bytes;
    }

    /// Returns the session label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the recorded phase timings.
    #[must_use]
    pub const fn phases(&self) -> &HashMap<&'static str, Duration> {
        &self.phases
    }

    /// Returns the event counters.
    #[must_use]
    pub const fn counters(&self) -> &HashMap<&'static str, u64> {
        &self.counters
    }

    /// Returns a human-readable report string.
    #[must_use]
    pub fn report(&self) -> String {
        let mut output = format!("Profiling Report: {}\n", self.label);
        let _ = writeln!(output, "  Allocations: {} ({} bytes)", self.allocations, self.allocation_bytes);
        output.push_str("  Phases:\n");
        let mut phases: Vec<_> = self.phases.iter().collect();
        phases.sort_by(|a, b| a.0.cmp(b.0));
        for (name, duration) in &phases {
            let ms = duration.as_secs_f64() * 1000.0;
            let _ = writeln!(output, "    {name}: {ms:.2}ms");
        }
        if !self.counters.is_empty() {
            output.push_str("  Counters:\n");
            let mut counters: Vec<_> = self.counters.iter().collect();
            counters.sort_by(|a, b| a.0.cmp(b.0));
            for (name, count) in &counters {
                let _ = writeln!(output, "    {name}: {count}");
            }
        }
        output
    }
}

impl Default for ProfilingSession {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Phase telemetry recorded for each compilation phase.
#[derive(Debug, Clone)]
pub struct PhaseTelemetry {
    /// Phase name.
    pub phase: &'static str,
    /// Duration of the phase.
    pub duration: Duration,
    /// Number of allocations made.
    pub allocations: u64,
    /// Total bytes allocated.
    pub allocation_bytes: u64,
    /// Cache hits during this phase.
    pub cache_hits: u64,
    /// Cache misses during this phase.
    pub cache_misses: u64,
    /// Diagnostics emitted during this phase.
    pub diagnostics_emitted: u64,
    /// Input node count.
    pub nodes_input: u64,
    /// Output node count.
    pub nodes_output: u64,
    /// Peak memory usage in bytes.
    pub peak_memory: u64,
}

impl PhaseTelemetry {
    /// Creates new phase telemetry.
    #[must_use]
    pub const fn new(phase: &'static str) -> Self {
        Self {
            phase,
            duration: Duration::ZERO,
            allocations: 0,
            allocation_bytes: 0,
            cache_hits: 0,
            cache_misses: 0,
            diagnostics_emitted: 0,
            nodes_input: 0,
            nodes_output: 0,
            peak_memory: 0,
        }
    }
}

/// A subscriber that receives telemetry events.
pub trait TelemetrySubscriber: Send + Sync {
    /// Called when a phase completes.
    fn on_phase_complete(&self, telemetry: &PhaseTelemetry);
    /// Called on a cache query.
    fn on_cache_query(&self, query: &str, hit: bool, duration: Duration);
}

/// A bus that distributes telemetry events to subscribers.
#[derive(Default)]
pub struct TelemetryBus {
    /// Registered telemetry subscribers.
    subscribers: Vec<Box<dyn TelemetrySubscriber>>,
}

impl TelemetryBus {
    /// Creates a new empty telemetry bus.
    #[must_use]
    pub fn new() -> Self {
        Self { subscribers: Vec::new() }
    }

    /// Registers a subscriber.
    pub fn subscribe(&mut self, subscriber: Box<dyn TelemetrySubscriber>) {
        self.subscribers.push(subscriber);
    }

    /// Notifies all subscribers of phase completion.
    pub fn on_phase_complete(&self, telemetry: &PhaseTelemetry) {
        for subscriber in &self.subscribers {
            subscriber.on_phase_complete(telemetry);
        }
    }

    /// Notifies all subscribers of a cache query.
    pub fn on_cache_query(&self, query: &str, hit: bool, duration: Duration) {
        for subscriber in &self.subscribers {
            subscriber.on_cache_query(query, hit, duration);
        }
    }
}

/// A console subscriber that prints phase timings to stderr.
pub struct ConsoleSubscriber;

impl TelemetrySubscriber for ConsoleSubscriber {
    fn on_phase_complete(&self, telemetry: &PhaseTelemetry) {
        let ms = telemetry.duration.as_secs_f64() * 1000.0;
        eprintln!(
            "[profiling] {}: {:.2}ms ({} nodes in -> {} nodes out, {} diags)",
            telemetry.phase,
            ms,
            telemetry.nodes_input,
            telemetry.nodes_output,
            telemetry.diagnostics_emitted,
        );
    }

    fn on_cache_query(&self, query: &str, hit: bool, duration: Duration) {
        let ms = duration.as_secs_f64() * 1000.0;
        eprintln!("[profiling] cache {}: {} ({:.2}ms)", if hit { "hit" } else { "miss" }, query, ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_timer() {
        let mut timer = PhaseTimer::new("test");
        std::thread::sleep(Duration::from_millis(1));
        let elapsed = timer.stop();
        assert!(elapsed.as_millis() >= 1);
    }

    #[test]
    fn test_phase_timer_pause_resume() {
        let mut timer = PhaseTimer::new("test");
        timer.pause();
        let paused_elapsed = timer.elapsed();
        std::thread::sleep(Duration::from_millis(5));
        let still_paused = timer.elapsed();
        assert_eq!(paused_elapsed, still_paused);
        timer.resume();
        std::thread::sleep(Duration::from_millis(1));
        let elapsed = timer.stop();
        assert!(elapsed.as_millis() >= 1);
    }

    #[test]
    fn test_session() {
        let mut session = ProfilingSession::new("test");
        let mut timer = session.start_phase("phase1");
        std::thread::sleep(Duration::from_millis(1));
        timer.stop();
        session.record_phase("phase1", timer.elapsed());
        session.increment_counter("nodes_processed", 42);
        let report = session.report();
        assert!(report.contains("phase1"));
        assert!(report.contains("42"));
    }

    #[test]
    fn test_phase_telemetry() {
        let t = PhaseTelemetry::new("parse");
        assert_eq!(t.phase, "parse");
        assert_eq!(t.nodes_input, 0);
    }

    #[test]
    fn test_telemetry_bus() {
        struct TestSubscriber {
            called: std::sync::atomic::AtomicBool,
        }
        impl TelemetrySubscriber for TestSubscriber {
            fn on_phase_complete(&self, _: &PhaseTelemetry) {
                self.called.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            fn on_cache_query(&self, _: &str, _: bool, _: Duration) {}
        }
        let sub = Box::new(TestSubscriber {
            called: std::sync::atomic::AtomicBool::new(false),
        });
        let mut bus = TelemetryBus::new();
        bus.subscribe(sub);
        bus.on_phase_complete(&PhaseTelemetry::new("test"));
    }

    #[test]
    fn test_console_subscriber() {
        let sub = ConsoleSubscriber;
        let t = PhaseTelemetry::new("test");
        sub.on_phase_complete(&t);
        sub.on_cache_query("parse_html", true, Duration::from_millis(5));
    }

    #[test]
    fn test_session_default() {
        let session = ProfilingSession::default();
        assert_eq!(session.label(), "default");
    }

    #[test]
    fn test_session_allocations() {
        let mut session = ProfilingSession::new("alloc_test");
        session.record_allocation(10, 1024);
        let report = session.report();
        assert!(report.contains("1024 bytes"));
    }

    #[test]
    fn test_multiple_phases() {
        let mut session = ProfilingSession::new("multi");
        let mut t1 = session.start_phase("a");
        std::thread::sleep(Duration::from_millis(1));
        session.record_phase("a", t1.stop());
        let mut t2 = session.start_phase("b");
        std::thread::sleep(Duration::from_millis(1));
        session.record_phase("b", t2.stop());
        let report = session.report();
        assert!(report.contains("a"));
        assert!(report.contains("b"));
    }
}
