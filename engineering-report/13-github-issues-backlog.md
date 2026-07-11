# GitHub Issues Backlog

## Epic Issues

### EPIC-001: HTML Parser Migration
**Milestone:** M1 | **Priority:** Critical | **Labels:** `epic`, `parser`, `html`

Description: Replace the custom HTML parser with html5ever as the sole parser. Wire the existing `motarjim-html` crate into the main compiler pipeline.

**Sub-tasks:**
- [ ] Convert `motarjim-html::Document` to `motarjim-ast-html::Document` (or vice versa)
- [ ] Wire `motarjim-html::HtmlParser` into `motarjim-core::Compiler`
- [ ] Remove or archive `motarjim-parser/src/html.rs`
- [ ] Remove HTML-specific code from `motarjim-lexer` (keep CSS lexer)
- [ ] Update all tests that depend on custom parser behavior
- [ ] Update fuzz targets

### EPIC-002: CSS Engine Completion
**Milestone:** M2 | **Priority:** Critical | **Labels:** `epic`, `css`

Description: Complete the CSS engine to handle real-world CSS: combinators, variables, media queries, grid, positioning offsets.

### EPIC-003: Generator Quality
**Milestone:** M4 | **Priority:** High | **Labels:** `epic`, `generator`

Description: Make all three generators produce correct, idiomatic platform code with full CSS property mapping.

### EPIC-004: Performance Optimization
**Milestone:** M5 | **Priority:** High | **Labels:** `epic`, `performance`

Description: Arena allocation, string interning, lazy styles, parallel execution.

---

## Feature Issues

### FEAT-001: Wire html5ever into main pipeline
**Milestone:** M1 | **Priority:** Critical | **Labels:** `feature`, `parser`, `html` | **Estimate:** 2 weeks

The custom HTML parser has limited error recovery, no character reference decoding, and no CDATA support. `motarjim-html` uses html5ever (Servo's spec-compliant parser) but isn't wired into the pipeline. This enables proper HTML5 parsing with full error recovery.

### FEAT-002: Wire IncrementalEngine into Compiler
**Milestone:** M1 | **Priority:** High | **Labels:** `feature`, `incremental`, `core` | **Estimate:** 1-2 weeks

`IncrementalEngine` at `motarjim-incremental/src/lib.rs` is fully implemented but never instantiated or used by `Compiler::compile()`. This enables file-level dependency tracking and minimal rebuilds.

### FEAT-003: Wire ArtifactCache into Compiler
**Milestone:** M1 | **Priority:** High | **Labels:** `feature`, `cache`, `core` | **Estimate:** 1 week

`ArtifactCache` at `motarjim-cache/src/lib.rs` is fully implemented but never called. This enables disk caching between sessions.

### FEAT-004: CSS variable resolution
**Milestone:** M2 | **Priority:** Critical | **Labels:** `feature`, `css` | **Estimate:** 2-3 weeks

`var(--name, fallback)` references pass through the cascade as raw strings. Implement a variable resolution pass that collects `--custom-property` declarations and substitutes `var()` references.

### FEAT-005: Media query evaluation
**Milestone:** M2 | **Priority:** High | **Labels:** `feature`, `css` | **Estimate:** 2-3 weeks

`@media` rules are unconditionally included. Pass viewport dimensions through `CompileOptions`/`Session` and evaluate media conditions in `StyleResolver`.

### FEAT-006: Responsive breakpoint → IR
**Milestone:** M3 | **Priority:** High | **Labels:** `feature`, `ir` | **Estimate:** 1-2 weeks

`ResponsiveInferrer::infer()` returns `Vec::new()` unconditionally. Wire media query min-width/max-width conditions to produce `ResponsiveVariant`s with appropriate `Breakpoint` values.

### FEAT-007: Source maps generation
**Milestone:** M6 | **Priority:** Medium | **Labels:** `feature`, `generator`, `core` | **Estimate:** 3-4 weeks

No mapping from generated code positions back to source HTML/CSS. Implement source map v3 output in all three generators.

### FEAT-008: CLI watch mode
**Milestone:** M6 | **Priority:** High | **Labels:** `feature`, `cli` | **Estimate:** 2-3 weeks

`motarjim watch` prints "not yet implemented". Implement file watcher with debounced recompilation using `motarjim-fs::watcher` and `IncrementalEngine`.

### FEAT-009: Multi-file / directory compilation
**Milestone:** M6 | **Priority:** Medium | **Labels:** `feature`, `cli` | **Estimate:** 1 week

`motarjim compile` accepts a single file. Add support for `motarjim compile ./pages/`.

### FEAT-010: Optimization levels
**Milestone:** M6 | **Priority:** Medium | **Labels:** `feature`, `optimizer`, `cli` | **Estimate:** 1-2 weeks

No `-O0`, `-O1`, `-O2` flag. Design optimization level system with pass selection.

### FEAT-011: CSS top/right/bottom/left positioning
**Milestone:** M2 | **Priority:** Medium | **Labels:** `feature`, `css`, `ast` | **Estimate:** 1 week

`top`, `right`, `bottom`, `left` properties are parsed by LightningCSS but not stored in `ComputedStyle`.

### FEAT-012: CSS Grid structured parsing
**Milestone:** M2 | **Priority:** Medium | **Labels:** `feature`, `css` | **Estimate:** 2-3 weeks

Grid properties (`grid-template-columns`, `grid-template-rows`, etc.) stored as raw strings. Parse into structured track listings with line names, sizes, and areas.

### FEAT-013: calc() evaluation
**Milestone:** M2 | **Priority:** Medium | **Labels:** `feature`, `css` | **Estimate:** 2-3 weeks

`calc()` expressions parsed by LightningCSS but not evaluated. Implement a calc evaluator that handles +, -, *, / with unit conversion.

### FEAT-014: CSS @keyframes → platform animation
**Milestone:** M2 | **Priority:** Medium | **Labels:** `feature`, `css`, `ir` | **Estimate:** 3-4 weeks

`@keyframes` rules are parsed but produce no output. Extract keyframe data into IR and map to platform animation APIs.

### FEAT-015: Wire JS event bindings → generators
**Milestone:** M3 | **Priority:** Medium | **Labels:** `feature`, `js`, `ir`, `generator` | **Estimate:** 2-3 weeks

`find_dom_event_bindings()` extracts event handlers from JavaScript but nothing consumes this output. Wire through IR → generators so `onclick` → `onPressed`/`onClick`/`.onTapGesture`.

### FEAT-016: Accessibility in generated code
**Milestone:** M4 | **Priority:** Medium | **Labels:** `feature`, `accessibility`, `generator` | **Estimate:** 2-3 weeks

`AccessibilityInfo` is extracted in IR but no generator uses it. Add semantic labels, traits, and hints to generated platform code.

### FEAT-017: HTML character reference decoding
**Milestone:** M6 | **Priority:** Medium | **Labels:** `feature`, `html`, `parser` | **Estimate:** 1-2 weeks

`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#123;`, etc. are not decoded by the custom HTML parser. (html5ever handles these automatically.)

### FEAT-018: CLI config file discovery
**Milestone:** M6 | **Priority:** Low | **Labels:** `feature`, `cli`, `config` | **Estimate:** 1 week

Config loading only checks CWD. Walk up directories to find `motarjim.json`/`motarjim.toml`.

### FEAT-019: WASM TypeScript types + npm package
**Milestone:** M8 | **Priority:** Medium | **Labels:** `feature`, `wasm` | **Estimate:** 1-2 weeks

WASM bindings exist but have no `.d.ts` TypeScript definitions and no npm package.

### FEAT-020: JSON diagnostic output
**Milestone:** M6 | **Priority:** Medium | **Labels:** `feature`, `diag` | **Estimate:** 1 week

`json` feature exists but emitter only supports terminal output. Support `--format json` for IDE integration.

---

## Bug Issues

### BUG-001: Flutter emit_table_cell produces invalid Dart
**Milestone:** M1 | **Priority:** High | **Labels:** `bug`, `gen-flutter` | **Estimate:** 1 day

File: `motarjim-gen-flutter/src/generator.rs:467` — `emit_table_cell` writes `TableRow(` instead of a table cell widget. Copy-paste error.

### BUG-002: SwiftUI emit_hstack alignment syntax wrong
**Milestone:** M1 | **Priority:** High | **Labels:** `bug`, `gen-swiftui` | **Estimate:** 1 day

File: `motarjim-gen-swiftui/src/generator.rs:379` — `.alignmentGuide(.top) { _ in .top }` — `.top` is not a valid value; should be a CGFloat.

### BUG-003: SwiftUI emit_dialog produces orphan modifier
**Milestone:** M1 | **Priority:** High | **Labels:** `bug`, `gen-swiftui` | **Estimate:** 1 day

File: `motarjim-gen-swiftui/src/generator.rs:456-464` — `.alert(...)` modifier emitted as standalone line without parent view.

### BUG-004: SwiftUI emit_nav_bar modifier chain invalid
**Milestone:** M1 | **Priority:** High | **Labels:** `bug`, `gen-swiftui` | **Estimate:** 1 day

File: `motarjim-gen-swiftui/src/generator.rs:163-165` — Modifiers emitted as standalone lines from child handler; should be applied by parent NavigationStack.

### BUG-005: Compose LazyColumn uses fake data
**Milestone:** M4 | **Priority:** Medium | **Labels:** `bug`, `gen-compose` | **Estimate:** 1 day

`motarjim-gen-compose/src/generator.rs` — Lists and grids use `listOf(1)` instead of actual children from IR.

### BUG-006: VirtualFileSystem.write doesn't persist
**Milestone:** M1 | **Priority:** Medium | **Labels:** `bug`, `fs` | **Estimate:** 1 day

`motarjim-fs/src/lib.rs` — `VirtualFileSystem::write()` clones the internal map but due to `&self` the mutation is never stored.

### BUG-007: CSS selector matching ignores combinators
**Milestone:** M1 | **Priority:** Critical | **Labels:** `bug`, `css` | **Estimate:** 1-2 weeks

`motarjim-css/src/matching.rs` — Only simple selectors are checked against the element. Descendant/child/sibling combinators are silently ignored.

### BUG-008: CSS media queries unconditionally included
**Milestone:** M2 | **Priority:** Critical | **Labels:** `bug`, `css` | **Estimate:** 2-3 weeks

`motarjim-css/src/resolver.rs:109-113` — `CssRule::Media` always descends into nested rules without evaluating conditions.

### BUG-009: CSS variables pass through as raw strings
**Milestone:** M2 | **Priority:** Critical | **Labels:** `bug`, `css` | **Estimate:** 2-3 weeks

`var(--primary)` in CSS passes through the entire pipeline as the literal string `"var(--primary)"` instead of being resolved.

### BUG-010: ResponsiveInferrer always returns empty
**Milestone:** M3 | **Priority:** High | **Labels:** `bug`, `ir` | **Estimate:** 1-2 weeks

`motarjim-ir/src/responsive.rs` — `infer()` returns `Vec::new()` unconditionally.

---

## Refactor Issues

### REFACTOR-001: Eliminate dual HTML parser
**Milestone:** M1 | **Priority:** Critical | **Labels:** `refactor`, `parser`, `html` | **Estimate:** 2 weeks

Two incompatible HTML parsers exist (custom + html5ever). Choose html5ever as the sole parser and convert AST types.

### REFACTOR-002: Remove dead LayoutStrategy enum
**Milestone:** M3 | **Priority:** Low | **Labels:** `refactor`, `ast-ir` | **Estimate:** 1 day

`motarjim-ast-ir/src/layout.rs` has a `LayoutStrategy` enum that duplicates `LayoutIr` but is never used.

### REFACTOR-003: Use serde Deserialize in config
**Milestone:** M6 | **Priority:** Low | **Labels:** `refactor`, `config` | **Estimate:** 1-2 days

`motarjim-config/src/lib.rs` manually walks JSON instead of deriving `serde::Deserialize`.

### REFACTOR-004: Enable DAG scheduler by default
**Milestone:** M1 | **Priority:** Medium | **Labels:** `refactor`, `core` | **Estimate:** 1 week

Move `dag` feature to default. Make `CompilationDag::execute_parallel()` the default compilation path.

### REFACTOR-005: Wire validation pass into CSS parser
**Milestone:** M2 | **Priority:** Low | **Labels:** `refactor`, `parser`, `css` | **Estimate:** 1 day

`motarjim-parser/src/css/validation.rs` has `#[allow(dead_code)]` — the `validate()` function is never called.

### REFACTOR-006: Use formatter platform modules in generators
**Milestone:** M4 | **Priority:** Low | **Labels:** `refactor`, `formatter`, `generator` | **Estimate:** 1 week

`motarjim-formatter` has `dart::write_class`, `kotlin::write_fun`, `swift::write_struct` but generators hand-code output.

---

## Testing Issues

### TEST-001: Golden/snapshot tests for all generators
**Milestone:** M4 | **Priority:** High | **Labels:** `testing`, `generator` | **Estimate:** 2-3 weeks

No expected output for any platform. Implement insta snapshot tests for all 9 examples across all 3 platforms.

### TEST-002: Fuzz targets for optimizer, IR, generators
**Milestone:** M1 | **Priority:** Medium | **Labels:** `testing`, `fuzz` | **Estimate:** 1 week

Only lexer, parser, and selector parsers have fuzz targets. Add targets for cascade resolution, IR inference, optimization passes, and generators.

### TEST-003: CSS engine property-based tests
**Milestone:** M2 | **Priority:** Medium | **Labels:** `testing`, `css` | **Estimate:** 1-2 weeks

Add `proptest` tests for CSS value parsing, cascade ordering, selector matching against randomized HTML trees.

### TEST-004: motarjim-js unit tests
**Milestone:** M1 | **Priority:** High | **Labels:** `testing`, `js` | **Estimate:** 2-3 weeks

`motarjim-js` is the largest crate (6,972 LOC) but has only 2 unit tests. Add tests for all parser productions, semantic analysis cases, and event binding extraction.

### TEST-005: End-to-end compilation benchmarks
**Milestone:** M5 | **Priority:** Medium | **Labels:** `testing`, `benchmark` | **Estimate:** 1 week

Current benchmarks test individual stages. Add a Criterion benchmark that runs the full pipeline.

---

## Performance Issues

### PERF-001: Arena allocation for AST/IR nodes
**Milestone:** M5 | **Priority:** High | **Labels:** `performance`, `ast`, `ir`, `core` | **Estimate:** 4-6 weeks

All AST and IR nodes are individually heap-allocated. Use typed arenas with bump allocation.

### PERF-002: String interning (SymbolId)
**Milestone:** M5 | **Priority:** Medium | **Labels:** `performance` | **Estimate:** 2-3 weeks

No global string interning for identifiers. Tag names, attribute names, CSS property names stored as `SmolStr` per occurrence.

### PERF-003: Lazy computed style
**Milestone:** M5 | **Priority:** Medium | **Labels:** `performance`, `css` | **Estimate:** 3-4 weeks

All 50+ CSS properties computed for every node. Only compute properties when requested by generators.

### PERF-004: Vec-indexed style map
**Milestone:** M5 | **Priority:** Low | **Labels:** `performance`, `css` | **Estimate:** 1 week

Styles stored as `HashMap<NodeId, ComputedStyle>`. Replace with `Vec<Option<ComputedStyle>>` indexed by `NodeId`.

### PERF-005: Parallel optimization passes
**Milestone:** M5 | **Priority:** Low | **Labels:** `performance`, `optimizer` | **Estimate:** 1 week

PassManager runs passes sequentially despite thread-safe design. Run independent passes in parallel via rayon.

---

## Documentation Issues

### DOC-001: Fix api/public-surface.md stub
**Milestone:** M8 | **Priority:** Medium | **Labels:** `docs` | **Estimate:** 1 week

Contains 29 `- fn` entries instead of real content. Replace with auto-generated or manually curated API reference.

### DOC-002: Fix architecture/pass-graph.md stub
**Milestone:** M8 | **Priority:** Low | **Labels:** `docs` | **Estimate:** 1 day

Empty mermaid graph and empty table. Fill in optimization pass dependency graph.

### DOC-003: Create CHANGELOG.md
**Milestone:** M8 | **Priority:** Medium | **Labels:** `docs` | **Estimate:** 1 day

No changelog exists. Create one following Keep a Changelog format.

### DOC-004: Add CONTRIBUTING.md migration guide
**Milestone:** M8 | **Priority:** Low | **Labels:** `docs` | **Estimate:** 1 week

Contributing guide references a TypeScript-era architecture. Update for current Rust workspace.

### DOC-005: Missing golden output in examples
**Milestone:** M4 | **Priority:** Medium | **Labels:** `docs`, `examples` | **Estimate:** 1-2 weeks

9 examples exist but none have expected output files for any platform.

---

## Infrastructure Issues

### INFRA-001: Add dependabot.yml
**Milestone:** M1 | **Priority:** Medium | **Labels:** `infra`, `ci` | **Estimate:** 1 day

No automated dependency update workflow.

### INFRA-002: Add CODEOWNERS
**Milestone:** M8 | **Priority:** Low | **Labels:** `infra`, `ci` | **Estimate:** 1 day

No ownership assignment for repository files.

### INFRA-003: Add SECURITY.md
**Milestone:** M8 | **Priority:** Medium | **Labels:** `infra`, `security` | **Estimate:** 1 day

No security reporting process documented.

### INFRA-004: Add dockerignore
**Milestone:** M1 | **Priority:** Low | **Labels:** `infra`, `docker` | **Estimate:** 1 day

Full repo copied into Docker build context; unnecessary.

### INFRA-005: Add test.sh (bash equivalent of test.ps1)
**Milestone:** M1 | **Priority:** Low | **Labels:** `infra`, `scripts` | **Estimate:** 1 day

Only PowerShell test script exists; Linux/macOS users cannot run it without PowerShell.
