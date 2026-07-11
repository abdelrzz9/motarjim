# Top 20 Highest-Priority Improvements

Ranked by impact × urgency, with architectural dependencies respected.

---

## 1. Wire html5ever into main pipeline

**Why:** The custom HTML parser has limited error recovery, no character references, no CDATA. html5ever is HTML5 spec-compliant and already exists in the codebase (`motarjim-html`). This is the single highest-impact change because it fixes the weakest point in the pipeline.

**Where:** `motarjim-parser`, `motarjim-html`, `motarjim-core`

**Effort:** 2 weeks | **Risk:** Low | **Depends on:** Nothing

---

## 2. Implement CSS combinator traversal

**Why:** CSS selector matching currently ignores combinators (descendant ` `, child `>`, sibling `+`/`~`). This means `.container .button` matches ALL buttons, not just descendants. This is a correctness bug that affects every CSS rule with combinators.

**Where:** `motarjim-css/src/matching.rs`, `motarjim-css/src/resolver.rs`

**Effort:** 1-2 weeks | **Risk:** Low | **Depends on:** Nothing

---

## 3. Implement CSS variable resolution

**Why:** `var(--primary, blue)` passes through the entire pipeline as the literal string `"var(--primary)"`. Any project using CSS custom properties produces incorrect output. This is the most common modern CSS feature that's broken.

**Where:** `motarjim-css` (new module: `variable.rs`)

**Effort:** 2-3 weeks | **Risk:** Medium | **Depends on:** Nothing

---

## 4. Add media query evaluation

**Why:** `@media (max-width: 768px)` rules are always unconditionally included. Responsive designs compile as if all breakpoints apply simultaneously. This breaks responsive CSS entirely.

**Where:** `motarjim-css/src/resolver.rs`, `motarjim-session` (viewport info)

**Effort:** 2-3 weeks | **Risk:** Medium | **Depends on:** Nothing

---

## 5. Wire ResponsiveInferrer from media queries

**Why:** The `ResponsiveInferrer` returns empty vec. Even with media query evaluation, responsive breakpoints won't flow into the IR without this.

**Where:** `motarjim-ir/src/responsive.rs`

**Effort:** 1-2 weeks | **Risk:** Low | **Depends on:** #4 (media query evaluation)

---

## 6. Fix all generator bugs

**Why:** Generated code is sometimes invalid (SwiftUI) or semantically wrong (Flutter table cell). Before adding features, fix what's broken.

**Where:**
- `motarjim-gen-flutter/src/generator.rs:467` — emit_table_cell
- `motarjim-gen-swiftui/src/generator.rs:379` — hstack alignment
- `motarjim-gen-swiftui/src/generator.rs:456-464` — dialog modifier
- `motarjim-gen-swiftui/src/generator.rs:163-165` — navbar modifier chain

**Effort:** 1 week | **Risk:** Low | **Depends on:** Nothing

---

## 7. Map CSS properties to generated code

**Why:** Only 2-6 of 50+ CSS properties produce platform code. Generators produce unstyled widgets. Minimum viable set: width, height, background, border, border-radius, font-size, font-family, text-align, gap, flex-grow, opacity, overflow, position, box-shadow.

**Where:** All 3 generator crates

**Effort:** 4-6 weeks | **Risk:** Low | **Depends on:** #6 (bug fixes first)

---

## 8. Wire IncrementalEngine into Compiler

**Why:** `IncrementalEngine` is fully implemented (380 LOC) but never instantiated. Enables minimal rebuilds when HTML/CSS changes.

**Where:** `motarjim-core/src/lib.rs` (Compiler::compile, Compiler::compile_file)

**Effort:** 1-2 weeks | **Risk:** Low | **Depends on:** #1 (html5ever integration for proper change detection)

---

## 9. Wire ArtifactCache into Compiler

**Why:** `ArtifactCache` is fully implemented (372 LOC) but never used. Enables disk caching between sessions.

**Where:** `motarjim-core/src/lib.rs` (Compiler::compile)

**Effort:** 1 week | **Risk:** Low | **Depends on:** Nothing

---

## 10. Enable DAG scheduler by default

**Why:** Parallel DAG scheduler (1,475 LOC) is behind a feature flag. Moving it to default enables parallel compilation without users opting in.

**Where:** `motarjim-core/Cargo.toml` (move `dag` to default features), `motarjim-core/src/lib.rs` (use DAG path by default)

**Effort:** 1 week | **Risk:** Low | **Depends on:** Nothing

---

## 11. Implement golden/snapshot tests

**Why:** No expected output for any platform. Every generator change risks regressions. insta snapshots would catch them.

**Where:** All 3 generator crates (new `tests/golden/` directories)

**Effort:** 2-3 weeks | **Risk:** Low | **Depends on:** #6, #7 (generators must be correct first)

---

## 12. Support CLI watch mode

**Why:** The most requested UX feature. `motarjim watch` is a stub. File watcher + debounced recompilation with `IncrementalEngine`.

**Where:** `motarjim-cli/src/lib.rs`, `motarjim-fs/src/lib.rs` (FileWatcher)

**Effort:** 2-3 weeks | **Risk:** Low | **Depends on:** #8 (incremental engine)

---

## 13. Implement source maps

**Why:** Generated code has no relation to source positions. Debugging requires manual cross-referencing. Source map v3 output from generators.

**Where:** All 3 generator crates, `motarjim-core` (SourceMap integration)

**Effort:** 3-4 weeks | **Risk:** Medium | **Depends on:** #7 (generator CSS mapping stable)

---

## 14. Add arena allocation

**Why:** Every AST/IR node is individually heap-allocated. Arena allocation with bump allocators can give 2-3× speedup on large pages.

**Where:** `motarjim-ast-html`, `motarjim-ast-ir`, `motarjim-parser`, `motarjim-ir`

**Effort:** 4-6 weeks | **Risk:** Medium | **Depends on:** RFC-0008

---

## 15. Implement CSS top/right/bottom/left

**Why:** Positioning offsets are parsed but not stored in `ComputedStyle`. Absolute positioning doesn't work.

**Where:** `motarjim-ast-html/src/style.rs`, `motarjim-css/src/properties.rs`

**Effort:** 1 week | **Risk:** Low | **Depends on:** Nothing

---

## 16. Wire JS event bindings to generators

**Why:** `find_dom_event_bindings()` extracts event handlers from JavaScript but nothing consumes them. Users expect `onclick="handle()"` → platform event handlers.

**Where:** `motarjim-js/src/events.rs`, `motarjim-ir/src/builder.rs`, all 3 generators

**Effort:** 2-3 weeks | **Risk:** Medium | **Depends on:** IR changes for event data

---

## 17. Implement CSS Grid structured parsing

**Why:** Grid properties stored as raw strings. Can't generate `GridView` with column definitions from `grid-template-columns: 1fr 1fr 1fr`.

**Where:** `motarjim-css/src/properties.rs`, `motarjim-ast-css/src/value.rs`

**Effort:** 2-3 weeks | **Risk:** Low | **Depends on:** Nothing

---

## 18. Enable LSP completion, hover, definition

**Why:** LSP is set up with tower-lsp but completion, hover, goto-definition, semantic tokens, and code actions are all stubs. This limits IDE integration.

**Where:** `motarjim-lsp/src/lib.rs`

**Effort:** 4-6 weeks | **Risk:** Medium | **Depends on:** #1, #2 (stable parsing first)

---

## 19. Add string interning (SymbolId)

**Why:** No global interning for identifiers. Repetitive tag names, class names, attribute names, and CSS property names are stored per-occurrence.

**Where:** New crate `motarjim-symbol` or extend `motarjim-span`

**Effort:** 2-3 weeks | **Risk:** Medium | **Depends on:** RFC-0008 (arena interaction)

---

## 20. Implement calc() evaluation

**Why:** `calc(100% - 40px)` is parsed by LightningCSS but passes through as raw string. Common CSS pattern produces no output.

**Where:** `motarjim-css` (new module: `calc.rs` or extend `value.rs`)

**Effort:** 2-3 weeks | **Risk:** Medium | **Depends on:** Nothing (standalone)

---

## Priority Matrix

```
                    HIGH IMPACT                    LOWER IMPACT
                  ┌──────────────────────────────────────────────┐
    URGENT        │  1. html5ever integration    12. CLI watch   │
                  │  2. CSS combinators          13. Source maps │
                  │  3. CSS variables             18. LSP stubs  │
                  │  4. Media queries                            │
                  │  6. Generator bug fixes                      │
                  │  8. Incremental engine                       │
                  │  9. Artifact cache                           │
                  ├──────────────────────────────────────────────┤
    IMPORTANT     │  5. Responsive IR             14. Arena      │
    BUT NOT       │  7. CSS property mapping      15. Positioning│
    URGENT        │ 10. DAG default               16. JS events  │
                  │ 11. Golden tests              17. Grid parsing│
                  │                               19. SymbolId   │
                  │                               20. calc()     │
                  └──────────────────────────────────────────────┘
```

## Recommended Sprint Plan

| Sprint | Focus | Issues |
|:------:|-------|--------|
| 1 | **Foundation** | #6 (bug fixes), #9 (cache), #10 (DAG) |
| 2 | **HTML + CSS correctness** | #1 (html5ever), #2 (combinators) |
| 3 | **CSS engine** | #3 (variables), #4 (media queries), #15 (positioning) |
| 4 | **IR + responsive** | #5 (responsive IR), #20 (calc), #17 (grid) |
| 5 | **Incremental + caching** | #8 (incremental engine), #12 (watch) |
| 6-7 | **Generator quality** | #7 (CSS mapping), #16 (JS events), #11 (golden tests) |
| 8 | **Performance** | #14 (arena), #19 (SymbolId) |
| 9 | **Tooling** | #13 (source maps), #18 (LSP) |
