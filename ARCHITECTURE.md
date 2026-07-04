# Architecture

## High-Level Overview

motarjim is a **source-to-source compiler** that translates HTML and CSS into native UI code for Flutter (Dart), Jetpack Compose (Kotlin), and SwiftUI (Swift). It follows a classic multi-stage compiler architecture with discrete, composable passes.

The compiler is built as a **Rust workspace** of single-responsibility crates. Each crate is independently publishable, testable, and benchmarkable. The TypeScript surface (VS Code extension, web playground, website) consumes the Rust engine via WASM or FFI and contains no compiler logic.

```
HTML + CSS
    │
    ▼
┌─────────────────────────────────────────────────┐
│                  Lexer Stage                     │
│  ┌─────────────┐  ┌─────────────┐               │
│  │  HtmlLexer  │  │  CssLexer   │               │
│  └─────────────┘  └─────────────┘               │
│         │               │                        │
│         ▼               ▼                        │
│  ┌─────────────┐  ┌─────────────┐               │
│  │ HtmlParser  │  │  CssParser  │               │
│  │ (recursive  │  │ (recursive  │               │
│  │  descent)   │  │  descent)   │               │
│  └─────────────┘  └─────────────┘               │
└─────────────────────────────────────────────────┘
         │               │
         ▼               ▼
┌─────────────────────────────────────────────────┐
│                  Style Stage                     │
│  ┌──────────────┐  ┌────────────┐               │
│  │  Selectors   │  │  Cascade   │               │
│  │  (matching)  │──▶│  (resolve) │               │
│  └──────────────┘  └────────────┘               │
│                           │                      │
│                           ▼                      │
│                    ┌──────────────┐              │
│                    │  Computed    │              │
│                    │  Style       │              │
│                    └──────────────┘              │
└─────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────┐
│                   IR Stage                       │
│  ┌──────────────┐  ┌──────────┐  ┌───────────┐ │
│  │  Semantic    │  │  Layout  │  │  Target   │ │
│  │  Inference   │  │Inference │  │  Hints    │ │
│  └──────────────┘  └──────────┘  └───────────┘ │
│         │               │              │         │
│         └───────────────┴──────────────┘         │
│                         │                        │
│                         ▼                        │
│                  ┌──────────────┐                │
│                  │  IrBuilder   │                │
│                  │  (IrNode)    │                │
│                  └──────────────┘                │
└─────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────┐
│                Optimizer Stage                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │  Merge   │ │  Flatten │ │  Dedup   │  ...    │
│  │  Text    │ │Containers│ │  Styles  │        │
│  └──────────┘ └──────────┘ └──────────┘        │
└─────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────┐
│               Generator Stage                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │  Flutter │ │  Compose │ │  SwiftUI │        │
│  │  (Dart)  │ │ (Kotlin) │ │  (Swift) │        │
│  └──────────┘ └──────────┘ └──────────┘        │
└─────────────────────────────────────────────────┘
```

## Crate Dependency Graph

```
motarjim-diag          (standalone - diagnostics, spans, severity)
motarjim-ast           (standalone - AST/IR type definitions)
motarjim-config        → diag, fs
motarjim-fs            → diag
motarjim-serialize     → ast, ir, config

motarjim-lexer         → diag, ast
motarjim-parser        → diag, ast, lexer

motarjim-selectors     → diag, ast
motarjim-css           → diag, ast, lexer, selectors

motarjim-ir            → ast, css, selectors
motarjim-optimizer     → diag, ir
motarjim-formatter     → diag, ast

motarjim-gen-flutter   → ast, ir, formatter
motarjim-gen-compose   → ast, ir, formatter
motarjim-gen-swiftui   → ast, ir, formatter

motarjim-cache         → diag, fs, serialize
motarjim-incremental   → cache, fs, parser, css
motarjim-profiling     (standalone)

motarjim-core          → ALL crates above (facade)
motarjim-cli           → core, config, fs, profiling, cache
motarjim-lsp           → core, cache, config
motarjim-ffi           → core
motarjim-wasm          → core, config
```

## Compiler Pipeline

### Phase 1: Lexing

**Input:** Raw HTML/CSS source text
**Output:** Stream of tokens with source positions
**Crates:** `motarjim-lexer`

Both HTML and CSS share a common `Cursor` abstraction that provides character-by-character iteration with position tracking. The lexer produces `Token<TokenKind>` values:

- `HtmlTokenKind`: `TagOpen`, `TagClose`, `AttributeName`, `AttributeValue`, `Text`, `Comment`, `Doctype`
- `CssTokenKind`: `Ident`, `AtKeyword`, `Hash`, `String`, `Number`, `Percentage`, `Dimension`, `Whitespace`, `Delim`, `Function`, `Colon`, `Semicolon`, etc.

The lexer supports error recovery: malformed tokens produce an `Error` token instead of panicking, allowing the parser to continue and collect multiple diagnostics.

### Phase 2: Parsing

**Input:** Token streams
**Output:** Typed ASTs
**Crates:** `motarjim-parser`

#### HTML Parser

Recursive-descent parser that produces a `Document` containing `HtmlNode` elements:

```rust
pub struct HtmlNode {
    pub id: NodeId,
    pub tag_name: SmolStr,
    pub attributes: Vec<Attribute>,
    pub children: Vec<HtmlNode>,
    pub value: Option<String>,       // Text content for #text nodes
    pub source_span: SourceSpan,
}
```

Supports: void elements, optional closing tags, implicit tag insertion, error recovery with diagnostic reporting.

#### CSS Parser

Recursive-descent parser that produces a `CssStylesheet` with `CssRule` and `CssDeclaration` types:

```rust
pub struct CssRule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<CssDeclaration>,
    pub source_span: SourceSpan,
}
```

Supports: class, ID, tag, universal, attribute, pseudo-class, pseudo-element selectors; `@media`, `@font-face`, `@keyframes` at-rules; selector lists; cascade layering.

### Phase 3: Style Resolution

**Input:** `Document` + `CssStylesheet`
**Output:** `HashMap<NodeId, ComputedStyle>`
**Crates:** `motarjim-css`, `motarjim-selectors`

Three sub-phases:

1. **Selector Matching** — For each HTML node, find all matching CSS rules. Uses the `motarjim-selectors` crate for selector parsing and specificity calculation. Parallelizable per node via rayon.

2. **Cascade Resolution** — Sort matching rules by origin, specificity, and source order. Apply declarations in cascade order, resolving `inherit`/`initial`/`unset` values.

3. **Computed Style** — Convert resolved declarations into typed `ComputedStyle` with parsed CSS values (colors, lengths, etc.). Typed value parsing happens in `motarjim-css::values`.

### Phase 4: IR Construction

**Input:** `Document` + `HashMap<NodeId, ComputedStyle>`
**Output:** `IrTree`
**Crates:** `motarjim-ir`

The IR (Intermediate Representation) is a platform-neutral tree that bridges styled HTML and platform code generation. Each `IrNode` contains three layers:

```rust
pub struct IrNode {
    pub id: NodeId,
    pub semantic: SemanticIr,     // Button, Text, Card, NavBar, etc.
    pub layout: LayoutIr,         // FlexColumn, FlexRow, Grid, Stack, etc.
    pub target: TargetIr,         // Platform-specific mapping hints
    pub computed_style: ComputedStyle,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
}
```

- **SemanticIR** — Inferred from tag name, class names, ARIA roles, and CSS patterns. `<nav>` → `NavigationBar`, `.card` with shadow → `Card`.
- **LayoutIR** — Inferred from CSS `display`, `flex-direction`, `grid-template`, and element dimensions.
- **TargetIR** — Platform-specific hints (e.g., which Flutter widget to use for a given semantic role).

### Phase 5: Optimization

**Input:** `IrTree`
**Output:** Optimized `IrTree`
**Crates:** `motarjim-optimizer`

A **PassManager** runs a sequence of modular optimization passes. Each pass implements the `OptimizationPass` trait:

```rust
pub trait OptimizationPass: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, tree: &mut IrTree, context: &PassContext) -> PassResult;
}
```

Default passes (in order):

| Pass | Description | Cost |
|------|-------------|------|
| `merge_text_nodes` | Merge adjacent text nodes | O(n) |
| `remove_empty_nodes` | Remove empty containers/text | O(n) |
| `flatten_containers` | Flatten single-child wrappers | O(n) |
| `style_deduplication` | Deduplicate identical styles | O(n log n) |
| `constant_folding` | Fold constant style expressions | O(n) |
| `dead_node_elimination` | Remove unreachable nodes | O(n) |
| `simplify_layout` | Simplify redundant layout wrappers | O(n) |

### Phase 6: Code Generation

**Input:** Optimized `IrTree`
**Output:** Platform source code (Dart/Kotlin/Swift)
**Crates:** `motarjim-gen-flutter`, `motarjim-gen-compose`, `motarjim-gen-swiftui`

Each generator crate walks the IR tree and emits platform-native code. Generators use the `motarjim-formatter` crate for consistent code output (indentation, line breaks, imports).

## Compilation Targets

```
                     ┌──────────────────┐
                     │  motarjim-core   │
                     │  (single source) │
                     └────────┬─────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
     ┌────────────┐  ┌──────────────┐  ┌──────────────┐
     │ Native CLI │  │ WebAssembly  │  │ Dynamic Lib  │
     │ (motarjim- │  │ (motarjim-   │  │ (motarjim-   │
     │   cli)     │  │   wasm)      │  │   ffi)       │
     └────────────┘  └──────────────┘  └──────────────┘
                              │               │
                              ▼               ▼
                     ┌──────────────┐  ┌──────────────┐
                     │ Web Playground│  │ VS Code Ext  │
                     │ (apps/       │  │ (packages/   │
                     │  playground) │  │ extension)   │
                     └──────────────┘  └──────────────┘
```

## Data Flow

```
Source Files
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  1. Read Files (motarjim-fs)                            │
│     - Abstract file system for testability              │
│     - Supports real FS, virtual FS, and remote FS       │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  2. Parse (motarjim-parser)                             │
│     - HtmlParser: tokens → Document                     │
│     - CssParser:  tokens → CssStylesheet                │
│     - Error recovery on both paths                      │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  3. Resolve Styles (motarjim-css + motarjim-selectors) │
│     - Selector matching (parallel via rayon)            │
│     - Cascade resolution                                │
│     - Computed style with typed values                  │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  4. Build IR (motarjim-ir)                              │
│     - Semantic inference                                │
│     - Layout inference                                  │
│     - Target platform hints                             │
│     - Responsive variant attachment                     │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  5. Optimize (motarjim-optimizer)                      │
│     - Pass manager with ordered passes                  │
│     - Each pass transforms tree in place                │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  6. Generate (motarjim-gen-*)                           │
│     - Platform-specific emitter                         │
│     - Formatted output via motarjim-formatter           │
└──────────────────────────────────────────────────────────┘
    │
    ▼
Generated Platform Code
    │
    ├── Dart/Flutter   →  lib/generated.dart
    ├── Kotlin/Compose →  app/.../GeneratedView.kt
    └── Swift/SwiftUI  →  GeneratedView.swift
```

## Design Decisions

### Why Rust?

- **Performance** — 40-66× faster than the TypeScript predecessor. Targets: 1ms for small pages, 30ms for large pages (5000 nodes).
- **Memory efficiency** — Arena allocation, zero-copy parsing, small string optimization. Target: 64-96 bytes per AST node vs. 200-400 bytes in JS.
- **Correctness** — Strong type system, ownership model, `#[deny(unsafe_code)]`, exhaustive pattern matching.
- **Ecosystem** — Cargo workspace, criterion benchmarks, proptest, cargo-fuzz, clippy.

### Why Separate Crates?

Each crate is independently publishable on crates.io. This enables:
- **Reusability** — `motarjim-diag` can be used by other Rust tools. `motarjim-selectors` can be embedded in browser testing frameworks.
- **Parallel compilation** — Cargo compiles crates in parallel.
- **Focused testing** — Each crate has its own test suite, benchmarks, and fuzz targets.
- **Feature gating** — Users select only the generators they need (`gen-flutter`, `gen-compose`, `gen-swiftui`).

### Why a Plugin System?

Generators are plugins registered via the `Generator` trait. This allows:
- **Third-party generators** — React Native, .NET MAUI, Qt, Tauri, etc. without modifying core.
- **Independent development** — Each generator lives in its own crate.
- **Feature selection** — Users only compile the generators they need.

### Why a Single IR?

Early prototypes had dual IR systems (legacy `UiNode` and new `IrNode`), causing confusion and duplication. The single `IrNode` with three layers (SemanticIR, LayoutIR, TargetIR) provides:
- A stable API contract between phases
- Platform-neutral abstraction before platform-specific generation
- Single optimization pass that benefits all generators

### Why Not Runtime/Interpretation?

motarjim generates static source files. There is no runtime library, no WebView, no interpretation layer. The output is:
- **Idiomatic** — Uses standard platform APIs (Material Design widgets, Compose modifiers, SwiftUI views).
- **Editable** — Output is meant to be checked into version control and maintained by hand if desired.
- **Performant** — No overhead from a runtime bridge or DOM emulation.

## Key Architecture Patterns

### Query System (Incremental Cache)

Inspired by rustc's query system and Salsa. Each compilation phase is a `Query` with a key, value, and invalidation pattern. Results are cached and invalidated based on dependency changes.

| Query | Key | Value | Invalidation |
|-------|-----|-------|-------------|
| `ParseHtml` | FilePath | Document | OnFileChange |
| `ParseCss` | FilePath | Stylesheet | OnFileChange |
| `CascadeStyles` | NodeId | ComputedStyle | OnDependencyChange |
| `BuildIr` | (Document, Stylesheet) | IrTree | OnDependencyChange |
| `GenerateCode` | (IrTree, Target) | String | AlwaysExecute |

### Event System

Each phase emits lifecycle events. The LSP, plugins, and profiling infrastructure subscribe to these events:

- `BeforeParse` / `AfterParse`
- `BeforeStyle` / `AfterStyle`
- `BeforeIr` / `AfterIr`
- `BeforeOptimize` / `AfterOptimize`
- `BeforeGenerate` / `AfterGenerate`

### Cancellation Token

Long-running operations check a shared `CancelToken`. When the user edits a file (in LSP mode), the previous compilation is cancelled and a new one starts. No work is wasted.

### Telemetry

Every phase emits structured telemetry: duration, allocations, cache hits/misses, nodes processed. Subscribers include console output, JSON file, Prometheus metrics, and Chrome tracing.

## Future Architecture Plans

1. **Compilation DAG** — Replace the sequential pipeline with a Directed Acyclic Graph scheduler. Independent nodes (semantic inference, layout inference, accessibility analysis) execute concurrently via rayon.
2. **Incremental Recompilation** — Track per-file dependencies. Only recompile phases whose inputs changed.
3. **Arena Allocation** — Use typed arenas with bump allocators for all AST/IR nodes. Eliminate individual heap allocations.
4. **SIMD CSS Parsing** — Accelerate number parsing and string matching with SIMD instructions.
5. **Lazy Style Computation** — Only compute requested CSS properties instead of full computed style for every node.

## Performance Targets

| Scenario | Current (Rust) | Target |
|----------|---------------|--------|
| Small page (50 nodes) | ~2ms | ~1ms |
| Medium page (500 nodes) | ~10ms | ~5ms |
| Large page (5000 nodes) | ~98ms | ~30ms |
| Batch (100 pages) | ~1s | ~500ms |
