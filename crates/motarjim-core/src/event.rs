#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

//! Event system for the Motarjim compiler.
//!
//! Each compilation phase emits lifecycle events via the [`EventBus`].
//! Consumers subscribe by implementing [`EventHandler`] and registering
//! with [`EventBus::subscribe`].
//!
//! # Examples
//!
//! ```
//! use motarjim_core::event::{CompilerEvent, EventBus, EventHandler};
//!
//! struct Logger;
//! impl EventHandler for Logger {
//!     fn handle(&self, event: &CompilerEvent) -> Result<(), String> {
//!         println!("Event: {event:?}");
//!         Ok(())
//!     }
//! }
//!
//! let mut bus = EventBus::new();
//! bus.subscribe(Box::new(Logger));
//! ```

use motarjim_ast::css::CssStylesheet;
use motarjim_ast::ir::IrTree;
use motarjim_ast::{Document, SemanticDocument, StyledDocument};
use motarjim_diag::Diagnostic;

/// Lifecycle events emitted during each phase of compilation.
///
/// Each compilation phase has a `Before*` and `After*` event, allowing
/// subscribers to observe intermediate state, collect diagnostics, or
/// (via cancellation) abort the compilation.
#[derive(Debug, Clone)]
pub enum CompilerEvent {
    /// Emitted before HTML/CSS parsing begins.
    BeforeParse {
        /// The raw source input string.
        source: String,
    },
    /// Emitted after HTML/CSS parsing completes.
    AfterParse {
        /// The parse result — either a document or parse errors.
        result: Result<Document, Vec<Diagnostic>>,
    },
    /// Emitted before style resolution begins.
    BeforeStyle {
        /// The parsed document.
        document: Document,
        /// The parsed stylesheet.
        stylesheet: CssStylesheet,
    },
    /// Emitted after style resolution completes.
    AfterStyle {
        /// The style resolution result.
        result: Result<StyledDocument, Vec<Diagnostic>>,
    },
    /// Emitted before semantic analysis begins.
    BeforeSemantics {
        /// The styled document.
        styled: StyledDocument,
    },
    /// Emitted after semantic analysis completes.
    AfterSemantics {
        /// The semantic analysis result.
        result: Result<SemanticDocument, Vec<Diagnostic>>,
    },
    /// Emitted before IR construction begins.
    BeforeIr {
        /// The semantically-annotated document.
        semantic: SemanticDocument,
    },
    /// Emitted after IR construction completes.
    AfterIr {
        /// The IR construction result.
        result: Result<IrTree, Vec<Diagnostic>>,
    },
    /// Emitted before an optimization pass runs.
    BeforeOptimize {
        /// The IR tree to be optimized.
        tree: IrTree,
        /// The name of the optimization pass.
        pass: &'static str,
    },
    /// Emitted after an optimization pass completes.
    AfterOptimize {
        /// The optimization result — either the optimized tree or errors.
        result: Result<IrTree, Vec<Diagnostic>>,
        /// The name of the optimization pass.
        pass: &'static str,
    },
    /// Emitted before code generation for a target begins.
    BeforeGenerate {
        /// The optimized IR tree.
        tree: IrTree,
        /// The target platform name (e.g. "flutter", "compose", "swiftui").
        target: String,
    },
    /// Emitted after code generation completes.
    AfterGenerate {
        /// The generation result — either the generated code or errors.
        result: Result<String, Vec<Diagnostic>>,
        /// The target platform name.
        target: String,
    },
    /// Emitted when a cancellation has been requested for the given phase.
    CancelRequested {
        /// The name of the phase being cancelled.
        phase: &'static str,
    },
}

/// Trait for handling compiler lifecycle events.
///
/// Implementors receive [`CompilerEvent`]s via the [`EventBus`] and can
/// observe, log, or intervene in the compilation pipeline.
pub trait EventHandler: Send + Sync {
    /// Called for each emitted compiler event.
    ///
    /// # Errors
    /// Returns `Err(String)` if the handler wishes to signal an error
    /// that should abort or otherwise be reported for the compilation.
    fn handle(&self, event: &CompilerEvent) -> Result<(), String>;
}

/// A simple event bus that distributes [`CompilerEvent`]s to all
/// registered [`EventHandler`] subscribers.
///
/// # Examples
///
/// ```
/// use motarjim_core::event::{CompilerEvent, EventBus, EventHandler};
///
/// struct MyHandler;
/// impl EventHandler for MyHandler {
///     fn handle(&self, event: &CompilerEvent) -> Result<(), String> {
///         Ok(())
///     }
/// }
///
/// let mut bus = EventBus::new();
/// bus.subscribe(Box::new(MyHandler));
/// ```
pub struct EventBus {
    /// Registered handlers, notified in subscription order.
    handlers: Vec<Box<dyn EventHandler>>,
}

impl EventBus {
    /// Creates a new empty event bus.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Registers an event handler to receive all future events.
    pub fn subscribe(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    /// Emits an event to all registered handlers.
    ///
    /// Each handler is called in subscription order. The first error
    /// returned by any handler is propagated immediately; remaining
    /// handlers are **not** called after an error.
    ///
    /// # Errors
    /// Returns `Err(String)` if any handler returned an error.
    pub fn emit(&self, event: &CompilerEvent) -> Result<(), String> {
        for handler in &self.handlers {
            handler.handle(event)?;
        }
        Ok(())
    }

    /// Returns the number of registered handlers.
    #[must_use]
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// A handler that records received events for later inspection.
    #[derive(Default)]
    struct Recorder {
        events: std::sync::Mutex<Vec<CompilerEvent>>,
    }

    impl Recorder {
        #[allow(dead_code)]
        fn count(&self) -> usize {
            self.events.lock().expect("lock poisoned").len()
        }

        #[allow(dead_code)]
        fn last(&self) -> Option<CompilerEvent> {
            let guard = self.events.lock().expect("lock poisoned");
            guard.last().cloned()
        }
    }

    impl EventHandler for Recorder {
        fn handle(&self, event: &CompilerEvent) -> Result<(), String> {
            self.events
                .lock()
                .expect("lock poisoned")
                .push(event.clone());
            Ok(())
        }
    }

    /// A handler that always returns an error.
    struct FailHandler;

    impl EventHandler for FailHandler {
        fn handle(&self, _event: &CompilerEvent) -> Result<(), String> {
            Err("handler failed".to_string())
        }
    }

    #[test]
    fn test_eventbus_new_is_empty() {
        let bus = EventBus::new();
        let event = CompilerEvent::CancelRequested { phase: "test" };
        // No handlers → no error
        assert!(bus.emit(&event).is_ok());
    }

    #[test]
    fn test_eventbus_subscriber_receives_event() {
        let mut bus = EventBus::new();
        bus.subscribe(Box::new(Recorder::default()));

        let event = CompilerEvent::BeforeParse {
            source: "<div>".to_string(),
        };
        assert!(bus.emit(&event).is_ok());
    }

    #[test]
    fn test_multiple_subscribers_all_receive_events() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        struct CountingHandler(Arc<AtomicUsize>);
        impl EventHandler for CountingHandler {
            fn handle(&self, _event: &CompilerEvent) -> Result<(), String> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let mut bus = EventBus::new();
        bus.subscribe(Box::new(CountingHandler(Arc::clone(&count))));
        bus.subscribe(Box::new(CountingHandler(Arc::clone(&count_clone))));

        let event = CompilerEvent::BeforeParse {
            source: "test".to_string(),
        };
        assert!(bus.emit(&event).is_ok());
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_error_handling_propagates() {
        let mut bus = EventBus::new();
        bus.subscribe(Box::new(Recorder::default()));
        bus.subscribe(Box::new(FailHandler));

        let event = CompilerEvent::BeforeParse {
            source: "test".to_string(),
        };
        let err = bus.emit(&event).unwrap_err();
        assert_eq!(err, "handler failed");
    }

    #[test]
    fn test_event_types_are_distinct() {
        let events = vec![
            CompilerEvent::BeforeParse {
                source: "a".to_string(),
            },
            CompilerEvent::AfterParse {
                result: Err(vec![]),
            },
            CompilerEvent::BeforeStyle {
                document: Document::new(),
                stylesheet: CssStylesheet {
                    rules: vec![],
                    source_path: None,
                },
            },
            CompilerEvent::AfterStyle {
                result: Err(vec![]),
            },
            CompilerEvent::BeforeSemantics {
                styled: StyledDocument {
                    nodes: vec![],
                    root_id: motarjim_ast::NodeId(0),
                },
            },
            CompilerEvent::AfterSemantics {
                result: Err(vec![]),
            },
            CompilerEvent::BeforeIr {
                semantic: SemanticDocument {
                    nodes: vec![],
                    root_id: motarjim_ast::NodeId(0),
                },
            },
            CompilerEvent::AfterIr {
                result: Err(vec![]),
            },
            CompilerEvent::BeforeOptimize {
                tree: IrTree {
                    nodes: vec![],
                    root_id: motarjim_ast::NodeId(0),
                    target_hints: vec![],
                },
                pass: "merge_text",
            },
            CompilerEvent::AfterOptimize {
                result: Err(vec![]),
                pass: "merge_text",
            },
            CompilerEvent::BeforeGenerate {
                tree: IrTree {
                    nodes: vec![],
                    root_id: motarjim_ast::NodeId(0),
                    target_hints: vec![],
                },
                target: "flutter".to_string(),
            },
            CompilerEvent::AfterGenerate {
                result: Err(vec![]),
                target: "flutter".to_string(),
            },
            CompilerEvent::CancelRequested { phase: "parse" },
        ];

        // Ensure all variants can be constructed and match
        assert_eq!(events.len(), 13);
    }

    #[test]
    fn test_recorder_captures_order() {
        let mut bus = EventBus::new();
        bus.subscribe(Box::new(Recorder::default()));

        let event1 = CompilerEvent::BeforeParse {
            source: "first".to_string(),
        };
        let event2 = CompilerEvent::AfterParse {
            result: Ok(Document::new()),
        };

        assert!(bus.emit(&event1).is_ok());
        assert!(bus.emit(&event2).is_ok());
    }

    #[test]
    fn test_cancel_requested_event() {
        let mut bus = EventBus::new();
        bus.subscribe(Box::new(Recorder::default()));

        let event = CompilerEvent::CancelRequested { phase: "style" };
        assert!(bus.emit(&event).is_ok());
    }

    #[test]
    fn test_eventbus_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<EventBus>();
        assert_sync::<EventBus>();
    }

    #[test]
    fn test_eventbus_default() {
        let bus = EventBus::default();
        assert!(bus.handlers.is_empty());
    }
}
