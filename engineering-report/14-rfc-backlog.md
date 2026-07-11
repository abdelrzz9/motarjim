# RFC Backlog

## Purpose

RFCs (Request for Comments) formalize significant architectural decisions before implementation. Each RFC should be a document in `docs/rfcs/` following the template:

```
# RFC-NNNN: Title

## Summary
## Motivation
## Design
## Alternatives Considered
## Implementation Plan
## Migration (if applicable)
## Open Questions
```

---

## RFC-0001: IR Design (HIR/MIR/LIR Split)

**Priority:** Critical | **Milestone:** M3 | **Author:** TBD

### Problem
The current single `IrNode` with three inline layers (Semantic, Layout, Target) is insufficient for a production compiler. It makes backends harder to add, prevents shared optimization, and mixes platform concerns with platform-neutral data.

### Scope
- Define HIR (High-Level IR), MIR (Mid-Level IR), LIR (Low-Level IR)
- NodeId stability across lowering passes
- Visitor traits for each IR level
- Slotmap vs Vec storage for nodes
- Lowering pass trait and registration

### Key Questions
- Should HIR/MIR/LIR be separate crates or modules?
- How do lowering passes compose?
- What is the stability guarantee for NodeId?
- How does the optimization pass system map to the new IR levels?

### References
- `ARCHITECTURE-v2.md` — existing proposal for HIR/MIR/LIR
- `motarjim-ast-ir/src/ir.rs` — current IR types
- `motarjim-ast-ir/src/layout.rs` — current layout types (including dead LayoutStrategy)

---

## RFC-0002: CSS Engine Architecture

**Priority:** Critical | **Milestone:** M2 | **Author:** TBD

### Problem
The CSS engine has fundamental gaps: no combinator traversal, no variable resolution, no media query evaluation, no calc() evaluation. The architecture needs to be clarified for how these features compose.

### Scope
- Variable resolution strategy (when, where, how many passes)
- Media query evaluation (viewport input, feature evaluation)
- Calc() evaluation (AST-based evaluator)
- Combinator-aware selector matching (DOM tree walk)
- Property value computation pipeline

### Key Questions
- Should variable resolution be a pre-cascade pass or a post-cascade pass?
- How does viewport information flow into the cascade?
- Should calc() be evaluated during value parsing or during style computation?
- How does the cascade handle unresolved var() references?

---

## RFC-0003: Backend Generator API

**Priority:** High | **Milestone:** M4 | **Author:** TBD

### Problem
The current `Generator` trait is simple (`fn name()`, `fn generate()`) but doesn't provide enough structure for complex generators. CSS property mapping, platform hint consumption, and multi-file output need standardization.

### Scope
- `Generator` trait v2 API
- `CSSPropertyMapper` abstraction
- Platform hint consumption from TargetIr
- Multi-file output support (one file per component/class)
- Generator configuration (format, output dir, options)

### Key Questions
- Should generators produce a file tree (Vec<(Path, String)>) instead of a single String?
- How do generators communicate with each other?
- What is the plugin discovery mechanism?

### References
- `motarjim-core/src/plugin.rs` — current Generator/Plugin traits
- `docs/PLUGIN_GUIDE.md` — current plugin documentation

---

## RFC-0004: Plugin System v2

**Priority:** High | **Milestone:** M4 | **Author:** TBD

### Problem
The plugin system works for built-in generators but needs to be production-ready for third-party plugins: versioning, discovery, sandboxing, and configuration.

### Scope
- Plugin manifest format (motarjim-plugin.toml)
- Plugin discovery (filesystem search, environment variable)
- Version compatibility (semver matching)
- Generator composition (multiple generators for one platform)
- Plugin configuration (how plugins receive config)

### Key Questions
- Should plugins be dynamic libraries (dlopen) or source-level (cargo dependency)?
- How do we ensure API stability for plugins?
- What is the plugin sandboxing model?

---

## RFC-0005: Diagnostics System

**Priority:** Medium | **Milestone:** M6 | **Author:** TBD

### Problem
The diagnostic system has 22 error codes, a working emitter, but lacks: JSON output, diagnostic suppression, auto-fix infrastructure, and error code documentation.

### Scope
- Error code allocation scheme (reserve ranges for plugins)
- JSON diagnostic protocol (for LSP and IDE integration)
- Warning suppression (`#[allow(...)]` equivalents for HTML/CSS)
- Auto-fix suggestions (structured replacements)
- `--explain` command
- Diagnostic phases (which phase produced which diagnostic)

### Key Questions
- Should error messages be translatable?
- How do plugins register error codes?
- What is the format for structured auto-fixes?
- Should there be a `Diagnostic::merge()` for child diagnostics?

### References
- `motarjim-errors/src/diagnostic.rs` — current Diagnostic struct
- `motarjim-errors/src/code.rs` — current DiagnosticCode
- `motarjim-diag/src/codes.rs` — predefined error codes
- `motarjim-diag/src/emitter.rs` — current emitter

---

## RFC-0006: Layout Engine

**Priority:** Medium | **Milestone:** M3 | **Author:** TBD

### Problem
The layout inference is minimal: it maps CSS display/flex/position to LayoutIr variants but doesn't understand flexbox semantics (cross size, main size, wrapping behavior) or grid layout (track sizing, placement).

### Scope
- Flexbox layout model in MIR (flex-container + flex-item properties)
- Grid layout model (tracks, lines, areas, placement)
- Absolute positioning model (containing block, offsets)
- Z-index and stacking context
- Overflow behavior (scroll, clip, visible, auto)
- Intrinsic sizing (min-content, max-content, fit-content)

### Key Questions
- How much of the CSS layout algorithm should the IR model vs. delegate to platforms?
- Should MIR include computed sizes (after layout) or just constraints?
- How do we handle platform layout differences (Flutter flex vs. SwiftUI layout priority)?

### References
- `motarjim-ir/src/layout.rs` — current LayoutInferrer
- `motarjim-ast-ir/src/ir.rs` — current LayoutIr

---

## RFC-0007: Incremental Compilation

**Priority:** High | **Milestone:** M1 | **Author:** TBD

### Problem
`IncrementalEngine` and `ArtifactCache` exist but are not wired into the compiler. The query system from ARCHITECTURE-v2.md is partially implemented in `motarjim-core/src/query.rs` behind a feature flag.

### Scope
- Query system formalization (keys, values, invalidation patterns)
- File-level dependency tracking
- Cached artifact format and versioning
- Cache eviction policy
- Cross-session cache persistence

### Key Questions
- Should we use a Salsa-like model or a simpler key-value cache?
- How do we detect invalidations from imported files (@import, <link>, <script src>)?
- What is the granularity of caching (per-phase? per-file? per-node?)?

### References
- `motarjim-core/src/query.rs` — current Query trait and QueryCache
- `motarjim-cache/src/lib.rs` — ArtifactCache
- `motarjim-incremental/src/lib.rs` — IncrementalEngine
- `ARCHITECTURE-v2.md` — Query Engine design

---

## RFC-0008: Arena Allocation

**Priority:** Medium | **Milestone:** M5 | **Author:** TBD

### Problem
All AST/IR nodes are individually heap-allocated. Arena allocation with bump allocators can significantly improve throughput and memory locality.

### Scope
- Typed arena crate selection (typed-arena, bumpalo, or custom)
- Arena initialization and teardown
- Node creation API (arena.alloc(node) vs Node::new())
- Interaction with incremental compilation (serialization of arena-allocated data)
- Memory safety guarantees

### Key Questions
- Should arenas be per-phase or shared across the entire compilation?
- How do we serialize arena-allocated graphs for caching?
- Can we use `#[derive(ArenaAlloc)]` or similar macro for ergonomics?
- How do weak references (parent pointers) interact with arena allocation?

---

## RFC-0009: Responsive Design

**Priority:** Medium | **Milestone:** M3 | **Author:** TBD

### Problem
Responsive design is a complete gap. Media queries are parsed but never evaluated. The ResponsiveInferrer is a stub.

### Scope
- Breakpoint detection from @media min-width/max-width
- Responsive variant IR representation
- Platform-specific responsive widget mapping (LayoutBuilder, BoxWithConstraints, GeometryReader)
- Multiple breakpoint compilation (compile once, output responsive code)

### Key Questions
- Should we compile separate output per breakpoint or responsive widgets?
- How do we handle complex media queries (orientation, prefers-color-scheme)?
- What is the unit for breakpoint values (px only, or any length)?

### References
- `motarjim-ir/src/responsive.rs` — current stub
- `motarjim-ast-ir/src/layout.rs` — Breakpoint, ResponsiveVariant types

---

## RFC-0010: Source Maps

**Priority:** Medium | **Milestone:** M6 | **Author:** TBD

### Problem
No mapping from generated platform code back to source HTML/CSS. Debugging generated code requires manual correlation.

### Scope
- Source map format (source-map v3, or custom?)
- Location tracking through pipeline phases
- Generator integration (emit source map alongside code)
- Debug information format per platform

### Key Questions
- Should we use the standard source-map v3 format or something simpler?
- How do we map a Flutter `Row` back to a CSS `display: flex`?
- How do we handle multiple source files (HTML + CSS) in one output file?
- Should source maps include semantic information (which CSS property produced which widget parameter)?

---

## RFC-0011: Configuration System

**Priority:** Low | **Milestone:** M6 | **Author:** TBD

### Scope
- Environment variable overrides
- Multi-level config (defaults → project → CLI)
- Config schema validation
- Config file discovery (walk up directories)
- Config file watching for live reload

### References
- `motarjim-config/src/lib.rs` — current Config implementation

---

## RFC-0012: Testing Strategy

**Priority:** Low | **Milestone:** M1 | **Author:** TBD

### Scope
- Snapshot testing conventions (insta)
- Fuzz target organization
- Benchmark organization and regression thresholds
- Property-based testing strategy (proptest)
- Integration test framework (golden files)
- Test environment (virtual filesystem, mock diagnostics)

### References
- `docs/TESTING_GUIDE.md` — current testing documentation
- `motarjim-test-utils/` — test utility crate

---

## Summary Table

| RFC | Title | Priority | Milestone | Dependencies |
|:---:|-------|:--------:|:---------:|:------------:|
| 0001 | IR Design (HIR/MIR/LIR) | Critical | M3 | None |
| 0002 | CSS Engine Architecture | Critical | M2 | None |
| 0003 | Backend Generator API | High | M4 | RFC-0001 |
| 0004 | Plugin System v2 | High | M4 | None |
| 0005 | Diagnostics System | Medium | M6 | None |
| 0006 | Layout Engine | Medium | M3 | RFC-0001 |
| 0007 | Incremental Compilation | High | M1 | None |
| 0008 | Arena Allocation | Medium | M5 | RFC-0001 |
| 0009 | Responsive Design | Medium | M3 | RFC-0002 |
| 0010 | Source Maps | Medium | M6 | RFC-0003 |
| 0011 | Configuration System | Low | M6 | None |
| 0012 | Testing Strategy | Low | M1 | None |

**Recommended RFC review order:** 0001 → 0002 → 0007 → 0003 → 0004 → 0006 → 0009 → 0005 → 0008 → 0010 → 0011 → 0012
