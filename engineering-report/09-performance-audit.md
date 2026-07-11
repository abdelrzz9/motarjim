# Performance Audit

## Current Performance

| Scenario | Reported Current | Target | Headroom |
|----------|:----------------:|:------:|:--------:|
| Small page (50 nodes) | ~2ms | ~1ms | 2× |
| Medium page (500 nodes) | ~10ms | ~5ms | 2× |
| Large page (5000 nodes) | ~98ms | ~30ms | 3.3× |
| Batch (100 pages) | ~1s | ~500ms | 2× |

Target performance is achievable, but requires addressing the issues below.

---

## Allocations

### No Arena Allocation (High Impact)

**Problem:** All AST and IR nodes are individually heap-allocated via `Vec<HtmlNode>` and `Vec<IrNode>`. No typed arena with bump allocation.

**Location:** Every crate that creates nodes (parser → `HtmlNode`, IR builder → `IrNode`)

**Impact:** For a 5000-node page: 5000+ individual Vec allocations, plus per-field allocations for Strings.

**Fix:** Introduce typed arenas (e.g., `typed-arena` or `bumpalo`) for `HtmlNode`, `IrNode`, `ComputedStyle`, and `Attribute` allocations. Replace `Vec<HtmlNode>` with `Arena<HtmlNode>` + indices.

**Estimated gain:** 2-3× speedup for large pages, 30-40% memory reduction.

### String Cloning (High Impact)

**Problem:** Raw strings are cloned extensively throughout the pipeline:
- Tokenizer produces owned `String` for each token's `raw` field
- Parser clones attribute values
- CSS value parsing clones strings
- Generator emits new strings for each property

**Location:** `motarjim-lexer` (Token.raw), `motarjim-parser` (attribute values), `motarjim-css` (property values), all generators

**Impact:** For a 5000-node page with average 5 attributes each: 25,000+ string allocations.

**Fix:** Use `SmolStr` consistently (already used in many places but not all). Where possible, use `&str` slices into the original source text (requires lifetime changes).

**Estimated gain:** 15-25% reduction in allocation count.

### No String Interning (Medium Impact)

**Problem:** Identifiers (tag names, attribute names, CSS property names, CSS values) are stored as `SmolStr` or `String` with no global interning.

**Location:** `TagName`, `Attribute::name`, `ComputedStyle` field names, property names in cascade

**Impact:** Duplicate string storage — `<div>` appears 500 times, stored 500 times.

**Fix:** Introduce a `SymbolId` type backed by a global string interner (e.g., `lasso` or `string-interner`). All identifiers reference `SymbolId` instead of `SmolStr`.

**Estimated gain:** 20-30% memory reduction for repetitive HTML/CSS.

---

## Parallelism

### Single-Threaded by Default (High Impact)

**Problem:** The DAG scheduler (`motarjim-core/src/dag.rs`, 1,475 LOC) supports parallel compilation with 13 phases across 7 levels, but it's behind the `dag` feature flag (disabled by default). The default `Compiler::compile()` runs phases sequentially.

**Location:** `motarjim-core/src/lib.rs` — `Compiler::compile()` method

**Impact:** Multiple cores idle during compilation. CSS selector matching and IR inference are embarrassingly parallel.

**Fix:** Enable `dag` feature by default, or make the DAG scheduler the default path. Use `rayon::par_iter()` for per-node operations.

### PassManager Runs Passes Sequentially (Medium Impact)

**Problem:** All 6 optimization passes run sequentially despite thread-safe `PassStatistics` (atomics).

**Location:** `motarjim-optimizer/src/pass_manager.rs`

**Impact:** Independent passes (e.g., `RemoveUnusedStyles` and `CollapseWhitespace`) could run concurrently.

**Fix:** Analyze pass dependencies and run independent passes in parallel via `rayon`.

### CSS Selector Matching Uses Rayon but Only When Called (Medium Impact)

**Problem:** `StyleResolver::resolve_parallel()` uses `rayon::par_iter()` but `resolve()` (single-threaded) is the default.

**Location:** `motarjim-css/src/resolver.rs`

**Impact:** Selector matching for large pages doesn't parallelize by default.

**Fix:** Make `resolve_parallel` the default in the pipeline.

---

## Memory

### ComputedStyle Cloned Per Node (Medium Impact)

**Problem:** `ComputedStyle` is a large struct (50+ fields, some `Option<String>`) computed for every node. Identical styles are not shared.

**Location:** `motarjim-css/src/resolver.rs`, `motarjim-ast-html/src/style.rs`

**Impact:** Duplicate style storage — 10 heading elements with identical computed styles are stored 10 times.

**Fix:** Implement style deduplication (hash-consed `ComputedStyle` via `Arc` or interned index). The `style_deduplication` pass exists in the optimizer but only removes identical styles at the IR level, not during construction.

### No Lazy Style Computation (Medium Impact)

**Problem:** All CSS properties are computed for every node, even if the generator only uses a subset (e.g., only width, color, padding).

**Location:** `motarjim-css/src/properties.rs`

**Impact:** Wasted computation: parsing all 50+ properties when only 5-10 are needed.

**Fix:** Implement lazy/computed-on-demand style properties. Only parse values when the generator (or downstream) requests them.

---

## Data Structures

### `HashMap<..>` for Style Map (Low Impact)

**Problem:** Styles stored as `HashMap<NodeId, ComputedStyle>`. Lookup and insertion have hash overhead.

**Location:** `motarjim-core/src/lib.rs` (output of style resolution)

**Fix:** Use `Vec<Option<ComputedStyle>>` indexed by `NodeId` (which is already an index into the arena). Avoid hashing entirely.

### `SmallVec<[NodeId; 4]>` Already Good

Children use `SmallVec` with inline storage for up to 4 items. For larger child counts, switches to heap. This is correct. No change needed.

### `SmolStr` Already Good

Small string optimization (SSO) is used throughout. Strings under 22 bytes (on 64-bit) are stored inline, avoiding heap allocation. Tag names, class names, and attribute values all benefit.

---

## Caching

### No Pipeline Caching Wired (High Impact)

**Problem:** `ArtifactCache` exists but is never called from the pipeline. Every compilation is from scratch.

**Location:** `motarjim-cache/src/lib.rs`

**Impact:** Identical source produces identical compilation work every time.

**Fix:** Wire `ArtifactCache` into `Compiler::compile()` — check cache before each phase, store results after.

### No Incremental Rebuilds Wired (High Impact)

**Problem:** `IncrementalEngine` exists but is never used by `Compiler::compile()`.

**Location:** `motarjim-incremental/src/lib.rs`

**Impact:** Changing one CSS rule recompiles the entire page.

**Fix:** Wire `IncrementalEngine` into `Compiler::compile_file()`. Track per-file dependencies. Only recompile affected phases.

### No Query Cache Wired

**Problem:** The `Query` trait and `QueryCache` exist in `motarjim-core/src/query.rs` (503 LOC) but require the `query-system` feature flag (disabled by default).

**Fix:** Enable by default and wire into pipeline phases.

---

## Benchmarking

### Current Benchmarks

| Crate | Benchmark | Harness |
|-------|-----------|---------|
| `motarjim-lexer` | `lexer_bench` | Criterion |
| `motarjim-parser` | `parser_bench` | Criterion |
| `motarjim-css` | `css_bench` | Criterion |
| `motarjim-ir` | `ir_bench` | Criterion |
| `motarjim-optimizer` | `optimizer_bench` | Criterion |
| `motarjim-gen-flutter` | `gen_bench` | Criterion |
| `motarjim-gen-compose` | `gen_bench` | Criterion |
| `motarjim-gen-swiftui` | `gen_bench` | Criterion |

All generators use the same benchmark harness name (`gen_bench`) — they produce different Criterion benchmarks via different crate compilation.

### Missing Benchmarks

| Missing Benchmark | Impact |
|-------------------|--------|
| End-to-end pipeline benchmark (all stages combined) | High — current split benchmarks don't show real-world performance |
| Incremental compilation benchmark | Medium — shows benefit of caching |
| Memory allocation benchmark (allocations per compilation) | Medium — tracks regression |
| WASM loading time benchmark | Low — important for web playground UX |

---

## SIMD Opportunities (Future)

| Location | Opportunity | Impact |
|----------|-------------|--------|
| CSS number parsing | Use SIMD to skip whitespace and parse digits in parallel | Medium |
| CSS selector matching | Use SIMD for attribute value comparison | Low |
| HTML entity decoding | Use SIMD for `&` lookup | Low |
| String trimming/splitting | SIMD-accelerated whitespace detection | Low |

These are optimization opportunities for later milestones (post-1.0).

---

## Priority Performance Improvements

| # | Improvement | Est. Gain | Effort | Priority |
|--:|-------------|:---------:|:------:|:--------:|
| 1 | Wire incremental compilation | 5-10× for rebuilds | Medium | Critical |
| 2 | Enable DAG scheduler by default | 2-4× parallel speedup | Low | High |
| 3 | Arena allocation for AST/IR | 2-3× on large pages | Medium | High |
| 4 | Wire artifact cache | 2-5× for repeat compilation | Low | High |
| 5 | String interning (SymbolId) | 20-30% memory | Medium | Medium |
| 6 | Lazy computed style | 30-40% on large pages | Medium | Medium |
| 7 | Vec-indexed style map | ~5% on large pages | Low | Medium |
| 8 | Parallel optimization passes | ~20% on optimizer | Low | Low |
| 9 | End-to-end benchmarks | Tracking only | Low | Low |

**Performance Score: 5/10** — Good foundations (SmolStr, SmallVec, rayon, Criterion) but missing arenas, string interning, lazy computation, and pipeline caching.
