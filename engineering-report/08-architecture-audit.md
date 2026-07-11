# Architecture Audit

## Critical Issues

### 1. Dual HTML Parsers (Critical)

Two independent HTML parsers exist:
- **Custom parser** at `motarjim-parser/src/html.rs` (648 LOC) — used by default in pipeline
- **html5ever parser** at `motarjim-html` (2,118 LOC) — not wired into pipeline

**Problem:** They produce incompatible AST types (arena-based vs tree-based). The custom parser has limited error recovery, no character reference decoding, and no CDATA support. The html5ever parser is full HTML5 spec-compliant but unused.

**Impact:** Users get a subpar HTML parser despite a superior one existing in the codebase.

**Fix:** Wire `motarjim-html` into `motarjim-core` as the default HTML parser. Remove or archive the custom parser.

### 2. CSS Selector Combinators Ignored (Critical)

`motarjim-css/src/matching.rs` checks only simple selectors against the element. Combinators (descendant space, child `>`, sibling `+`/`~`) are silently ignored.

**Impact:** CSS rules like `.container .button { ... }` match all buttons, not just descendants of `.container`.

**Fix:** Implement DOM tree walk in `selector_matches_element()` using the parent/children links in `HtmlNode`.

### 3. CSS Variables Not Resolved (Critical)

`var(--name)` references pass through the cascade as raw string values. No substitution from custom properties.

**Impact:** Any CSS using custom properties produces incorrect output.

**Fix:** Add a `var()` resolution pass in `motarjim-css` that collects `--custom-property` declarations and substitutes `var()` references.

### 4. Media Queries Never Evaluated (Critical)

`@media` rules are always unconditionally included in the cascade. Viewport/environment information does not flow into the CSS engine.

**Impact:** Responsive designs compile as if all media rules apply simultaneously.

**Fix:** Pass viewport dimensions through the `Session`/`CompileOptions`. Add media query evaluation to `StyleResolver`.

### 5. Responsive IR is a Complete Stub (Critical)

`ResponsiveInferrer::infer()` returns `Vec::new()` unconditionally.

**Impact:** No responsive breakpoints are extracted. Generators cannot produce responsive layouts.

**Fix:** Wire media query conditions through to `ResponsiveInferrer`. Map min-width/max-width → Breakpoint variants.

### 6. Generators Produce Bugs (High)

- **Flutter:** `emit_table_cell` writes `TableRow(` instead of cell widget
- **SwiftUI:** `emit_hstack` alignment syntax wrong (produces invalid Swift)
- **SwiftUI:** `emit_dialog` modifier floats without parent
- **SwiftUI:** `emit_nav_bar` modifier chain invalid
- **Compose:** LazyColumn uses `listOf(1)` fake data

**Impact:** Generated code is sometimes invalid or semantically wrong.

---

## Moderate Issues

### 7. Incremental/Cache Not Wired (High)

`IncrementalEngine` and `ArtifactCache` are fully implemented but never instantiated or used by `Compiler::compile()`.

**Impact:** Every compilation starts from scratch. No rebuild optimization.

### 8. Feature Gates Disabled by Default (Medium)

- `dag` — parallel DAG scheduler (requires `rayon`)
- `cancellation` — cooperative cancellation
- `events` — lifecycle event bus
- `plugin-system` — dynamic generator dispatch
- `query-system` — Salsa-inspired query cache

**Impact:** The default compilation path is single-threaded with no cancellation support.

### 9. `LayoutStrategy` Dead Code (Low)

A second enum `LayoutStrategy` in `motarjim-ast-ir/src/layout.rs` (14 variants) duplicates `LayoutIr` (17 variants) but is never used by any inference code.

**Impact:** Confusion for developers; maintenance burden.

### 10. `LayoutConstraints` Dead Code (Low)

Struct defined in `motarjim-ast-ir/src/layout.rs` with min/max width/height and aspect ratio. Never instantiated outside tests.

### 11. Validation Pass Has `#[allow(dead_code)]` (Medium)

`motarjim-parser/src/css/validation.rs` (180 LOC) is a post-conversion validation pass with the `validate()` function never called in the main `parse()` flow.

### 12. `_diagnostics` Parameter Ignored (Medium)

`IrBuilder::build()` accepts `&DiagnosticBag` but the parameter is prefixed with `_` and never written to. Diagnostics from IR construction are silently dropped.

### 13. HTML Lexer Doesn't Produce Attribute Tokens (Medium)

`HtmlTokenizer` embeds tag attributes in the `OpenTagStart` token's `raw` string. The parser (`scan_tag_close`, `parse_attributes_from_str`) re-scans the raw text, effectively doing a second tokenization pass.

### 14. Feature Gates for Generators Defined but Unused (Low)

`gen-flutter`, `gen-compose`, `gen-swiftui` features exist in `motarjim-core/Cargo.toml` but are never checked with `#[cfg]` in source code.

### 15. Formatter Platform Modules Unused (Low)

`dart::write_class`, `kotlin::write_fun`, `swift::write_struct` in `motarjim-formatter` exist but generators hand-code output.

---

## Minor Issues

### 16. `SourceMap` No Path Normalization

`./foo.html` and `foo.html` are different keys. No path canonicalization.

### 17. TOML Config via JSON Round-Trip

`motarjim-config` converts TOML to `serde_json::Value` before deserializing. Loses TOML-specific types.

### 18. Manual JSON Walking in Config

Instead of deriving `serde::Deserialize`, config parsing manually walks the JSON tree. More verbose and error-prone.

### 19. `ChecksumMismatch` Error Variant Never Produced

`motarjim-cache` defines a `ChecksumMismatch` variant in its error enum but the `load()` method never produces it.

### 20. `VirtualFileSystem.write()` Mutation Bug

The `write()` method on `VirtualFileSystem` clones the internal map but due to `&self` the mutation is never stored. Writes disappear.

### 21. Cancellation Feature-Gated

`Session::cancel()` and `Session::is_cancelled()` require `#[cfg(feature = "cancellation")]`, forcing all downstream code to conditional-compile.

### 22. `PassManager` Dependency Declarations Unused

Each pass declares `prerequisites()` and `invalidated_by()`, but `PassManager::run_all()` runs passes in registration order without checking these.

### 23. `PassStatistics::memory_freed` Always 0

Counter exists but no pass ever increments it.

### 24. `SourceSpan::From<Range<usize>>` Sets Default Line/Col

Converting a byte range to `SourceSpan` sets line/col to 1/1 without resolution. Callers must separately resolve positions.

### 25. No EOF Sentinel Span

No `SourceSpan::EOF` or equivalent for end-of-file positions.

### 26. `Diagnostic` Has Only `&'static str` Messages

`DiagnosticCode::message` is `&'static str`, preventing dynamic error messages.

### 27. `Heading { level }` Uses `u32` Without Validation

HTML defines h1-h6 only. No clamping or validation.

### 28. `aria-labelledby` Not Resolved

Stored as raw string. No cross-reference to the referenced element's content.

---

## Summary

| Severity | Count | Key Examples |
|----------|:-----:|--------------|
| Critical | 6 | Dual parsers, no combinator traversal, no variables, no media queries, responsive stub, generator bugs |
| High | 2 | Incremental/cache not wired, feature gates useless by default |
| Medium | 6 | Dead code (LayoutStrategy, LayoutConstraints, validation pass), diagnostics ignored, lexer limitations, unused feature gates, unused formatter modules |
| Low | 14 | Path normalization, TOML round-trip, manual JSON, dead error variant, mutability bug, etc. |

**Architecture Score: 7/10** — Well-conceived at the macro level, but execution gaps at the micro level bring it down.
