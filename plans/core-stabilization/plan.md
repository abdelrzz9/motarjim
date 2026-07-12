# Core Stabilization Engineering Plan

## Objective
Fix the compiler's foundation so it produces correct output for basic HTML+CSS inputs, with a clean architecture that can be extended in later milestones.

## Scope
- Finalize html5ever integration + remove duplicate parser
- CSS combinator matching
- Wire IncrementalEngine + ArtifactCache into pipeline
- Fix generator correctness bugs
- Fix pipeline diagnostic flow
- Remove dead code

## Excluded (do NOT touch)
- CSS Grid, flexbox improvements, media queries, CSS variables, responsive IR
- LSP, watch mode, source maps
- Animations, performance optimization, arenas, string interning
- Any new features or redesigns

---

## Execution Order
1. TASK 7: Architecture cleanup (easy wins, no risk)
2. TASK 6: Pipeline diagnostics (fix observability before deep changes)
3. TASK 5: Generator bug fixes (quick correctness fixes)
4. TASK 2: CSS combinator matching (bigger change, test thoroughly)
5. TASK 1: Finalize html5ever (remove dead code once pipeline is clean)
6. TASKS 3+4: Wire incremental/cache (depends on clean pipeline)

---

## TASK 1: Finalize html5ever Integration

### Context
`Compiler::compile()` at `crates/motarjim-core/src/lib.rs:209` already uses `NewHtmlParser::parse(input)` (html5ever). But:
- CSS is extracted via a brittle regex (`extract_css_from_html()` at line 536-560)
- The old custom parser in `motarjim-parser/src/html.rs` (648 LOC) is dead code
- The HTML lexer in `motarjim-lexer` may have unused HTML-specific token types

### Sub-tasks

**1.1 Remove `extract_css_from_html()`**
- File: `crates/motarjim-core/src/lib.rs` (lines 536-560)
- The html5ever parser already produces a tree with `<style>` elements containing text children
- Replace: After parsing and converting to arena AST (`tree_doc_to_arena`), walk the arena document to find `<style>` elements and extract their text content
- Remove `fn extract_css_from_html()`
- Update the CSS parse phase (around line 242) to use AST-based extraction instead

**1.2 Remove or gate the custom HTML parser**
- File: `crates/motarjim-parser/src/html.rs` (entire file, ~648 LOC)
- File: `crates/motarjim-parser/src/lib.rs` (remove re-exports of custom parser)
- Remove `pub mod html;` and any re-exports from `lib.rs`
- Remove `pub use html::{...}` exports
- If any tests depend on the custom parser, update them or remove them

**1.3 Clean up unused HTML lexer types**
- File: `crates/motarjim-lexer/src/` (check `HtmlTokenKind`, `HtmlTokenizer`)
- If nothing depends on them after removing the custom parser, remove them
- Keep CSS lexer intact

**1.4 Update tests**
- Any `motarjim-parser/tests/` that depend on custom HTML parser → update or remove
- Run `cargo build` to identify any lingering references

### Verification
- `cargo test` passes
- `cargo build --workspace` passes
- All 9 example HTML files compile and produce output

---

## TASK 2: CSS Combinator Traversal

### Context
`crates/motarjim-css/src/matching.rs` checks only simple selectors against an element (lines 20-28). Combinators (descendant ` `, child `>`, next-sibling `+`, subsequent-sibling `~`) are silently ignored. The `motarjim-selectors` crate already parses combinators into `Selector.combinators`.

### Sub-tasks

**2.1 Change `selector_matches_element()` signature**
- Current: `fn selector_matches_element(selector: &Selector, element: &Element) -> bool`
- New: `fn selector_matches_element(selector: &Selector, node: &HtmlNode, nodes: &[HtmlNode]) -> bool`
- Need access to parent/children links to walk the DOM tree
- Import `motarjim_ast_html::HtmlNode` or use the existing `Element` + node context

**2.2 Implement combinator evaluation**
- When `selector.combinators` is non-empty, the selectors chain is: `[simple0, combinator0, simple1, combinator1, ..., simpleN]`
- `simpleN` must match the current element (last group)
- Walk backward: for each `(combinator, simple_selector_group)` pair, verify the DOM relationship matches:
  - **Descendant** (space): Walk `parent` links up the tree; any ancestor must match the simple selectors
  - **Child** (`>`): Check immediate parent only
  - **Next-sibling** (`+`): Check immediately preceding sibling in parent's children
  - **Subsequent-sibling** (`~`): Check any preceding sibling in parent's children

**2.3 Update `StyleResolver`**
- `collect_matching_declarations()` currently calls `rule_matches_element(style_rule, element)` which takes `&Element`
- Change to pass `&HtmlNode` + `&[HtmlNode]` context
- `resolve_with_parent()` iterates `ast.nodes` — pass the node and the full `nodes` slice

**2.4 Update `rule_matches_element()` callers**
- Current callers in `matching.rs` and `resolver.rs` that pass `element` directly
- `rule_max_specificity` stays the same (no context needed)

**2.5 Pseudo-classes**
- Keep returning `true` for now (out of current scope)
- This ensures existing behavior is preserved

### Verification
- `.container .button` only matches buttons inside `.container`
- `div > p` matches only direct children
- `h1 + p` matches only `p` immediately after `h1`
- `h1 ~ p` matches any `p` after `h1` with same parent
- All existing CSS tests pass

---

## TASK 3: Wire IncrementalEngine Into Pipeline

### Context
`motarjim-incremental/src/lib.rs` (380 LOC) has a fully implemented `IncrementalEngine` with:
- `FileState` tracking (path + SHA-256 hash)
- `CompilationStatus` (UpToDate / Stale / New)
- CSV-based persistence (`save()` / `load()`)

But `Compiler::compile()` never instantiates or calls it. Zero references in `motarjim-core/src/lib.rs`.

### Sub-tasks

**3.1 Add `IncrementalEngine` to Session**
- File: `crates/motarjim-session/src/lib.rs`
- Add field: `incremental: IncrementalEngine`
- Initialize in `Session::new()` — call `IncrementalEngine::load()` to restore previous state
- On `Session` drop (or explicit save), call `engine.save()`

**3.2 Wrap `compile_file()` with incremental check**
- File: `crates/motarjim-core/src/lib.rs` (around line 433-446)
- Before reading the file, compute its hash and call `engine.status_of(path)`
- If `UpToDate`: skip compilation, return cached result (or last known output)
- If `Stale` or `New`: compile normally, then call `engine.record_compilation(path, hash)`

**3.3 Feature gate**
- Add `incremental` feature to `Cargo.toml`
- Enable by default

### Verification
- Compiling same file twice: second call skips work
- Modifying file between compiles: recompiles
- `cargo test` passes

---

## TASK 4: Wire ArtifactCache Into Pipeline

### Context
`motarjim-cache/src/lib.rs` (372 LOC) has a fully implemented `ArtifactCache` with:
- `CacheKey` (source_hash + platform + config_hash)
- `store()` / `load()` methods
- Content-addressable disk storage

But `Compiler::compile()` never calls it. Zero references in `motarjim-core/src/lib.rs`.

### Sub-tasks

**4.1 Add `ArtifactCache` to Session**
- File: `crates/motarjim-session/src/lib.rs`
- Add field: `artifact_cache: ArtifactCache`
- Initialize in `Session::new()` with cache directory from config

**4.2 Cache check at beginning of `compile()`**
- File: `crates/motarjim-core/src/lib.rs` (around line 200, before any phase)
- Compute `CacheKey` from input source hash + platform + config hash
- Call `cache.load(&key)`
- On cache hit: return the cached `CompileResult` immediately (skip all phases)

**4.3 Cache store at end of `compile()`**
- After successful compilation (around line 420), call `cache.store(&key, &result)`

**4.4 Feature gate**
- Add `cache` feature to `Cargo.toml`
- Enable by default

### Verification
- Compiling same input twice: second call returns cached output instantly
- Changing input produces fresh output
- Changing platform produces fresh output (different cache key)
- `cargo test` passes

---

## TASK 5: Fix Generator Bugs

### 5.1 Flutter: `emit_table_cell` writes wrong widget
- File: `crates/motarjim-gen-flutter/src/generator.rs` (lines 466-475)
- Bug: Writes `TableRow(` instead of `TableCell(`
- Fix: Change `TableRow(` to the correct Flutter widget. In Flutter, `TableCell` is a widget used inside `Table`. The child should be wrapped in `TableCell(child: ...)`.
- Before: `w.write_line("TableRow(");`
- After: `w.write_line("TableCell(");` with proper structure

### 5.2 SwiftUI: `emit_hstack` alignment syntax
- File: `crates/motarjim-gen-swiftui/src/generator.rs` (lines 372-380)
- Bug: `.alignmentGuide(.top) { _ in .top }` — `.top` is a `VerticalAlignment`, not a `CGFloat`
- Fix: Use `HStack(alignment: .top)` or `HStack(alignment: .bottom)` constructor parameter instead of `.alignmentGuide()` modifier
- Map: `FlexStart` → `.top`, `FlexEnd` → `.bottom`, `Center` → `.center`, default → `.center`

### 5.3 SwiftUI: `emit_dialog` orphan modifier
- File: `crates/motarjim-gen-swiftui/src/generator.rs` (lines 454-465)
- Bug: `.alert(...)` emitted as standalone line, not applied to any view
- Fix: The dialog element should apply `.alert()` as a modifier on the parent view, or emit a `.sheet()` with `Alert` content. Simplest approach: emit `.alert("Title", isPresented: $isPresented) { Button("OK") { } }` as a modifier on the parent container.

### 5.4 SwiftUI: `emit_nav_bar` static title
- File: `crates/motarjim-gen-swiftui/src/generator.rs` (lines 163-166)
- Bug: Always `.navigationTitle("Page")` regardless of actual content
- Fix: Look at the `NavigationBar` semantic IR's first text child and use that as the title. Pass the actual title from the IR node's text or semantic context.

### Verification
- Generated Flutter Dart compiles with `dart analyze`
- Generated Swift compiles with `swiftc`
- Existing generator unit tests pass
- Update any test expectations that change

---

## TASK 6: Pipeline Correctness

### Context
The pipeline silently drops all diagnostics. `all_diagnostics` at `lib.rs:396` is literally `Vec::new()`. Parse errors, CSS errors, and IR warnings never reach the user.

### Sub-tasks

**6.1 Fix `all_diagnostics` collection**
- File: `crates/motarjim-core/src/lib.rs`
- Replace `let all_diagnostics = Vec::new();` with a real `DiagnosticBag` or `Vec<Diagnostic>`
- Collect diagnostics from:
  - `NewHtmlParser::parse()` result — check if it returns diagnostics
  - `CssParser::parse()` — currently `.ok()` drops errors; capture them
  - `IrBuilder::build()` — thread a `&mut DiagnosticBag`
  - Each generator's `generate()` — if it returns errors

**6.2 Wire `_diagnostics` in `IrBuilder`**
- File: `crates/motarjim-ir/src/builder.rs`
- Remove `_` prefix from `_diagnostics: &DiagnosticBag`
- Add diagnostic reporting in inference passes:
  - Unsupported `SemanticIr` variants → warning
  - Unresolvable `aria-labelledby` references → warning
  - Unknown layout modes → warning

**6.3 Collect CSS parse errors**
- File: `crates/motarjim-core/src/lib.rs:244-246`
- Change `css_parser.parse().ok()` to capture the error
- Convert `CssParser` error to a `Diagnostic` and push it

**6.4 Collector pattern**
- Create a `DiagnosticBag` at the start of `compile()`
- Pass `&mut` reference to each phase
- Drain it into `CompileResult.diagnostics` at the end

### Verification
- A page with invalid CSS produces a warning/error in diagnostics
- A page with an unknown HTML element parses cleanly, no diagnostics
- `CompileResult.diagnostics` is non-empty when there are issues
- All existing tests still pass

---

## TASK 7: Architecture Cleanup

### 7.1 Remove dead `LayoutStrategy` enum
- File: `crates/motarjim-ast-ir/src/layout.rs` (lines 1-109)
- Remove the `LayoutStrategy` enum entirely (14 variants, unused)
- Remove its unit tests for the enum variants
- Keep `LayoutConstraints`, `Breakpoint`, `ResponsiveVariant` — they may be used later

### 7.2 Mark `LayoutConstraints` as unused
- File: `crates/motarjim-ast-ir/src/layout.rs`
- Add `#[allow(dead_code)]` on `LayoutConstraints` struct
- Or remove it entirely if you prefer

### 7.3 Enable DAG scheduler by default
- File: `crates/motarjim-core/Cargo.toml`
- Change `default = []` to `default = ["dag"]`
- This activates the parallel DAG scheduler infrastructure

### 7.4 Remove unused generator feature gates
- File: `crates/motarjim-core/Cargo.toml` (lines 43-45)
- Remove `gen-flutter = []`, `gen-compose = [], `gen-swiftui = []`
- These are never checked with `#[cfg(feature = "...")]` in any source file

### Verification
- `cargo build` succeeds
- `cargo test` passes
- `cargo check --workspace` passes

---

## Files Changed Summary

| File | Task | Change |
|------|------|--------|
| `crates/motarjim-core/src/lib.rs` | 1.1, 6.1, 6.3 | Remove regex CSS extraction; collect end-to-end diagnostics |
| `crates/motarjim-parser/src/html.rs` | 1.2 | Entire file → remove |
| `crates/motarjim-parser/src/lib.rs` | 1.2 | Remove html module re-exports |
| `crates/motarjim-lexer/src/` | 1.3 | Remove unused HTML token types |
| `crates/motarjim-css/src/matching.rs` | 2.1-2.2 | Combinator tree walking |
| `crates/motarjim-css/src/resolver.rs` | 2.3 | Thread node context |
| `crates/motarjim-session/src/lib.rs` | 3.1, 4.1 | Add IncrementalEngine + ArtifactCache fields |
| `crates/motarjim-core/Cargo.toml` | 3.3, 4.4, 7.3, 7.4 | Features: incremental, cache, dag; remove unused gen-* |
| `crates/motarjim-gen-flutter/src/generator.rs` | 5.1 | emit_table_cell → TableCell |
| `crates/motarjim-gen-swiftui/src/generator.rs` | 5.2-5.4 | HStack alignment, dialog chain, nav bar title |
| `crates/motarjim-ir/src/builder.rs` | 6.2 | Remove `_` prefix, write diagnostics |
| `crates/motarjim-ast-ir/src/layout.rs` | 7.1-7.2 | Remove LayoutStrategy, mark LayoutConstraints |

---

## Test Plan

After each task:
```bash
cargo test -p <affected-crate>
cargo build --workspace
```

After all tasks:
```bash
cargo test --workspace
# Run examples
cargo run -- compile examples/blog.html --format dart
cargo run -- compile examples/blog.html --format kotlin
cargo run -- compile examples/blog.html --format swift
```

### New tests needed
- `motarjim-css`: Combinator matching against a mock DOM tree (descendant, child, sibling)
- `motarjim-gen-flutter`: Update `emit_table_cell` test expectation
- `motarjim-gen-swiftui`: Update HStack/dialog/nav-bar test expectations
- `motarjim-core`: Integration test for diagnostic collection (introduce CSS error → expect diagnostic)
- `motarjim-core`: Integration test for cache hit (compile twice → second faster)
