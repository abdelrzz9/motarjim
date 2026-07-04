# Motarjim — Full Architecture Review

## 1. Current Architecture Overview

The project is a TypeScript monorepo (npm workspaces) with 16 packages implementing an
HTML/CSS → Native UI compiler targeting Flutter, Jetpack Compose, and SwiftUI.

### Pipeline Flow (current)

```
HTML ──► Parser ──► HtmlNode ──► CSS Analyzer ──► StyledNode ──► IR ──► Optimizer ──► Generator ──► Code
            │                        │                              │
         parse5                   PostCSS                      UiNode/IrNode
```

### Packages and Responsibilities

| Package | Lines | Role |
|---------|-------|------|
| `shared` | ~1,600 | Types dumping ground: diagnostics, HTML AST, CSS types, IR types, layout types, accessibility, AI, semantic types |
| `parser` | 147 | Thin `parse5` wrapper — converts parse5 AST to `HtmlNode` |
| `css-analyzer` | ~2,500+ | Monolithic: CSS parsing, selector matching, cascade, computed style, layout bridge, responsive, intent, mappers |
| `ir` | 483 | Legacy UiNode IR + AI intent enrichment (Ollama) |
| `ir-v2` | ~700 | New IrNode three-layer IR (semantic + layout + target) |
| `optimizer` | 191 | Sequential optimization passes (merge text, flatten, prune) |
| `generator-core` | 280 | Shared emitter interfaces + tree walkers + utilities |
| `generators/flutter` | ~600 | UiNode + IrNode emitters for Dart |
| `generators/compose` | ~700 | UiNode + IrNode emitters for Kotlin |
| `generators/swiftui` | ~700 | UiNode + IrNode emitters for Swift |
| `cli` | ~1,500 | Commander-based CLI with wizard, config, validation, templates |
| `pipeline-core` | 242 | Orchestrator — wires all packages together |
| `compiler-core` | ~250 | Pass manager abstractions + plugin API (largely unused) |
| `semantic-analyzer` | ~200 | Rule-based + AI semantic detection |
| `accessibility-analyzer` | ~100 | ARIA attribute analysis |

---

## 2. Architectural Problems

### CRITICAL

#### P1: Dual IR systems with no clear migration path
- `ir/` produces `UiNode` (legacy, `type: string`, `properties: Record<string, unknown>`)
- `ir-v2/` produces `IrNode` (three-layer typed IR)
- Both are wired in `pipeline-core` — `runPipeline` uses UiNode, `runIrPipeline` uses IrNode
- The IrNode pipeline has no optimizer integration (`componentsDetected: 0`, `optimizationSavings: 0`)
- The IrNode pipeline still uses legacy `parseHtml` + `applyStyles` then converts to IrNode

#### P2: Shared package is a dumping ground
- `packages/shared/index.ts` (427 lines) exports: diagnostics, HTML types, CSS types, IR types, selector types, semantic types, layout types, responsive types, accessibility types, AI types
- `packages/shared/ir-v2.ts` (669 lines) contains the complete three-layer IR — mixing concerns across semantic, layout, and target domains
- No cohesion — this violates the Single Responsibility Principle at package level

#### P3: No real CSS pipeline — everything goes through PostCSS
- CSS parsing is a thin `postcss.parse()` wrapper (css-analyzer/index.ts)
- No CSS tokenizer exists
- No CSS parser — relying on PostCSS means:
  - No error recovery (PostCSS either parses or throws)
  - No incremental parsing
  - No source of truth for CSS spec conformance
  - Unnecessary dependency on a Node.js-specific library

#### P4: Parser is a non-entity
- 147 lines that just wrap parse5
- No error recovery
- No incremental parsing
- Only extracts `<body>` children, discarding `<head>`, `<title>`, etc.
- Has a global mutable counter (`nodeCounter`) — not thread-safe
- Loses all document structure (no `<html>`, `<head>`)

#### P5: Generator code duplication
- Each generator has TWO complete implementations: `index.ts` (UiNode) and `ir-generate.ts` (IrNode)
- Flutter: 2 files × ~300 lines each = ~600 lines of near-identical emitter logic
- Compose: 2 files × ~350 lines each = ~700 lines
- SwiftUI: 2 files × ~350 lines each = ~700 lines
- The `IrEmitter` interface in generator-core has 15 methods — every generator must implement all of them
- Modifier/formatting logic is duplicated across all three generators

#### P6: No unified CSS-to-layout bridge
- `layout-bridge-v2.ts` in css-analyzer re-implements layout inference separately from `ir-v2/transform.ts`
- The layout inference in `ir-v2/transform.ts:inferLayout()` duplicates the logic in `css-analyzer/layout-bridge-v2.ts`
- CSS mappers (spacing-mapper, typography-mapper, etc.) are used by computed-style.ts but the generators also have their own formatting logic

#### P7: compiler-core is unused scaffolding
- `packages/compiler-core/` defines a complete pass manager, plugin API, pipeline executor
- It is IMPORTED by nothing except its own index.ts
- The actual pipeline in `pipeline-core/index.ts` is a hard-coded sequential function
- This is ~250 lines of dead code

### HIGH

#### P8: No proper diagnostics system
- `DiagnosticBag` captures diagnostics but they're ad-hoc
- No diagnostic codes are registered or documented
- No source location tracking through the pipeline
- No colored output
- No suggestions
- No error recovery (parser throws on error)

#### P9: No incremental compilation
- Every invocation re-parses everything from scratch
- No file watcher integration with caching
- No change detection

#### P10: No caching
- No compilation cache at any level
- No AST caching
- No resolved style caching
- No generated code caching

#### P11: Testing is minimal
- 3 test files totaling ~1,175 lines
- Parser tests: 131 lines, basic happy paths only
- CSS analyzer tests: 634 lines (best coverage)
- Generator tests: 410 lines, basic structural checks
- No golden tests for generated output
- No snapshot tests for intermediate representations
- No benchmarks for performance
- No fuzz testing
- No property-based testing

#### P12: No LSP
- No language server exists
- No hover, completion, diagnostics, go-to-definition, rename
- This makes the tool unusable in IDE contexts

#### P13: AI integration is awkward
- `ir/ai-intent.ts` makes Ollama HTTP calls with retry logic inside the compiler pipeline
- `semantic-analyzer/ai.ts` duplicates Ollama integration
- The AI enhancement is bolted on, not architected as a plugin
- Timeouts and network failures can crash the build

#### P14: Layout system is over-engineered and duplicated
- `layout-types.ts`, `layout-engine.ts`, `layout-mapping.ts`, `layout-constraints.ts`, `layout-bridge-v2.ts`
- The layout system has 5 separate files but only `layout-bridge-v2.ts` is actually used
- `layout-engine.ts` (69 lines) defines `resolveLayoutTree()` but it's never called by the pipeline
- `layout-mapping.ts` (92 lines) defines platform mappings but they duplicate `generator-core/widget-engine.ts`

#### P15: Any-typed properties everywhere
- `UiNode.properties: Record<string, unknown>` — completely untyped
- CSS values are `string` throughout — no typed CSS property/value system
- The selector parser uses loose string matching for combinator types
- Many `as any` casts in tests

#### P16: CLI has inconsistent UX
- `convert` requires wizard if no args — confusing for CI/CD
- `watch` depends on `chokidar` but doesn't integrate with incremental compilation
- `batch` and `convert` duplicate pipeline logic
- `explain` is a one-off ASCII art command
- No `build`, `format`, `doctor`, `benchmark`, `analyze`, `version` commands exist (only in docs)

### MEDIUM

#### P17: No proper CSS selector parser
- The selector parser in `css-analyzer/selector.ts` is hand-written but minimal
- No support for `:nth-child()`, `:not()`, `:has()`, `::before/after`
- No pseudo-element support
- Pseudo-classes are accepted but ignored in matching
- No error recovery in parsing

#### P18: No CSS specificity calculator properly integrated
- `calculateSpecificity()` exists but is never called from the cascade
- The cascade sorts entries but doesn't use specificity for comparison
- The `cascadeCompare` function compares specificity manually but doesn't import `specificityCompare`

#### P19: Performance measurement is ad-hoc
- `performance.now()` is used in generators but not consistently
- No profiling infrastructure
- No benchmark suite integration

#### P20: Module resolution issues
- Uses `NodeNext` module resolution with `.js` extensions in imports
- But files are `.ts` at source — this works with tsc but causes issues with other tooling
- Path aliases needed in both vitest and vite configs

---

## 3. Dependency Graph (Current)

```
@motarjim/shared                       [no internal deps]
    │
    ├── @motarjim/parser              → shared
    ├── @motarjim/css-analyzer        → shared
    ├── @motarjim/semantic-analyzer   → shared
    ├── @motarjim/ir                  → shared, css-analyzer
    ├── @motarjim/ir-v2             → shared, css-analyzer
    ├── @motarjim/optimizer           → shared
    ├── @motarjim/accessibility-analyzer → shared
    ├── @motarjim/compiler-core       → shared
    ├── @motarjim/generator-core      → shared
    │
    ├── @motarjim/generator-flutter   → shared, generator-core
    ├── @motarjim/generator-compose   → shared, generator-core
    ├── @motarjim/generator-swiftui   → shared, generator-core
    │
    ├── @motarjim/pipeline-core       → ALL 10 other packages
    │
    ├── @motarjim/cli                 → parser, css-analyzer, semantic-analyzer,
    │                                     ir, optimizer, all 3 generators
    │
    └── @motarjim/web                 → pipeline-core (via Vite)
```

**Key observation**: `pipeline-core` depends on EVERYTHING — it's a god package.
The dependency graph is a flat star with `shared` at center and `pipeline-core` as a second hub.

---

## 4. Proposed Rust Architecture

### High-Level Module Structure

```
crates/
├── motarjim-core/              # Public API facade
│   ├── src/lib.rs              # Compiler entry points
│   ├── src/pipeline.rs         # Pipeline orchestrator
│   ├── src/options.rs          # Compiler options
│   └── src/result.rs           # Compiler result types
│
├── motarjim-ast/               # AST Definitions (no logic)
│   ├── src/
│   │   ├── lib.rs              # Re-exports
│   │   ├── html.rs             # HtmlNode, HtmlAttribute, Document
│   │   ├── css.rs              # CssStylesheet, CssRule, CssDeclaration
│   │   ├── selector.rs         # Selector AST (simple, compound, complex)
│   │   ├── ir.rs               # Intermediate Representation
│   │   ├── layout.rs           # Layout types
│   │   ├── style.rs            # Resolved and computed styles
│   │   ├── semantic.rs         # Semantic IR types (button, text, heading, etc.)
│   │   └── accessibility.rs    # Accessibility metadata types
│   │
│   └── Cargo.toml
│
├── motarjim-diag/              # Diagnostic system (standalone crate)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── diagnostic.rs       # Diagnostic, Severity, Code
│   │   ├── bag.rs              # DiagnosticBag
│   │   ├── span.rs             # SourceSpan, SourceFile
│   │   ├── emitter.rs          # Colored terminal output
│   │   └── codes.rs            # Registered diagnostic codes
│   │
│   └── Cargo.toml
│
├── motarjim-lexer/             # HTML + CSS lexer
│   ├── src/
│   │   ├── lib.rs
│   │   ├── html.rs             # HTML tokenizer
│   │   ├── css.rs              # CSS tokenizer
│   │   ├── token.rs            # Token definitions
│   │   └── cursor.rs           # Character cursor with position tracking
│   │
│   └── Cargo.toml
│
├── motarjim-parser/            # HTML parser (recursive descent)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── html.rs             # HTML document parser
│   │   ├── html/fragment.rs    # Fragment parser
│   │   └── error.rs            # Parse errors with recovery
│   │
│   └── Cargo.toml
│
├── motarjim-css/               # Complete CSS engine
│   ├── src/
│   │   ├── lib.rs
│   │   ├── lexer.rs            # CSS tokenizer
│   │   ├── parser.rs           # CSS parser (rules, at-rules, declarations)
│   │   ├── selector.rs         # Selector parser + matcher
│   │   ├── specificity.rs      # Specificity calculation
│   │   ├── cascade.rs          # Cascade resolution
│   │   ├── inheritance.rs      # Property inheritance rules
│   │   ├── computed.rs         # ComputedStyle resolution
│   │   ├── values/             # Typed CSS values
│   │   │   ├── mod.rs
│   │   │   ├── color.rs        # Color parsing + conversion
│   │   │   ├── length.rs       # Length (px, em, rem, %)
│   │   │   ├── spacing.rs      # Padding, margin shorthand
│   │   │   ├── typography.rs   # Font properties
│   │   │   ├── border.rs       # Border shorthand
│   │   │   ├── background.rs   # Background shorthand
│   │   │   └── transform.rs    # Transform parsing
│   │   │
│   │   ├── media.rs            # @media query parsing + matching
│   │   └── at_rules.rs         # @font-face, @keyframes, etc.
│   │
│   └── Cargo.toml
│
├── motarjim-selectors/         # CSS selector engine (extracted)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── parser.rs           # Standalone selector parser
│   │   ├── specificity.rs      # Specificity
│   │   └── matching.rs         # DOM node matching
│   │
│   └── Cargo.toml
│
├── motarjim-ir/                # IR construction + transformation
│   ├── src/
│   │   ├── lib.rs
│   │   ├── builder.rs          # HtmlNode + ResolvedStyle → IrNode
│   │   ├── semantic.rs         # Semantic role inference
│   │   ├── layout.rs           # Layout strategy inference
│   │   ├── responsive.rs       # Responsive variant attachment
│   │   └── target.rs           # Target platform hints
│   │
│   └── Cargo.toml
│
├── motarjim-optimizer/         # Optimization passes
│   ├── src/
│   │   ├── lib.rs
│   │   ├── pass.rs             # Pass trait definition
│   │   ├── passes/
│   │   │   ├── mod.rs
│   │   │   ├── merge_text.rs           # Merge adjacent text nodes
│   │   │   ├── remove_empty.rs         # Remove empty containers
│   │   │   ├── flatten_containers.rs   # Flatten single-child containers
│   │   │   ├── style_dedup.rs          # Remove redundant style properties
│   │   │   ├── constant_fold.rs        # Fold constant expressions
│   │   │   ├── dead_node.rs            # Remove unreferenced nodes
│   │   │   ├── merge_nested.rs         # Merge nested same-type containers
│   │   │   └── responsive.rs           # Responsive-specific optimizations
│   │   │
│   │   └── manager.rs          # Pass scheduling + ordering
│   │
│   └── Cargo.toml
│
├── motarjim-formatter/         # Code output formatting
│   ├── src/
│   │   ├── lib.rs
│   │   ├── writer.rs           # CodeWriter with indentation tracking
│   │   ├── dart.rs             # Dart formatting rules
│   │   ├── kotlin.rs           # Kotlin formatting rules
│   │   └── swift.rs            # Swift formatting rules
│   │
│   └── Cargo.toml
│
├── motarjim-gen-flutter/       # Flutter code generator
│   ├── src/
│   │   ├── lib.rs
│   │   ├── emitter.rs          # Dart widget emitter
│   │   ├── modifiers.rs        # Widget modifier chain builder
│   │   ├── layout.rs           # Flex/Stack/Scroll → Row/Column/Stack/ListView
│   │   └── imports.rs          # Import management
│   │
│   └── Cargo.toml
│
├── motarjim-gen-compose/       # Compose code generator
│   ├── src/
│   │   ├── lib.rs
│   │   ├── emitter.rs          # Kotlin composable emitter
│   │   ├── modifiers.rs        # Modifier chain builder
│   │   ├── layout.rs           # Row/Column/Box → Compose equivalents
│   │   └── imports.rs          # Import management
│   │
│   └── Cargo.toml
│
├── motarjim-gen-swiftui/       # SwiftUI code generator
│   ├── src/
│   │   ├── lib.rs
│   │   ├── emitter.rs          # Swift View emitter
│   │   ├── modifiers.rs        # View modifier chain
│   │   ├── layout.rs           # HStack/VStack/ZStack
│   │   └── imports.rs          # Import management
│   │
│   └── Cargo.toml
│
├── motarjim-cli/               # CLI application
│   ├── src/
│   │   ├── main.rs             # Entry point
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── build.rs        # motarjim build
│   │   │   ├── watch.rs        # motarjim watch
│   │   │   ├── format.rs       # motarjim format
│   │   │   ├── doctor.rs       # motarjim doctor
│   │   │   ├── init.rs         # motarjim init
│   │   │   ├── benchmark.rs    # motarjim benchmark
│   │   │   ├── version.rs      # motarjim version
│   │   │   └── analyze.rs      # motarjim analyze
│   │   │
│   │   ├── config.rs           # Config loading (serde)
│   │   ├── watch.rs            # File watcher
│   │   └── progress.rs         # Progress bar
│   │
│   └── Cargo.toml
│
├── motarjim-lsp/               # Language Server
│   ├── src/
│   │   ├── main.rs             # LSP entry point
│   │   ├── server.rs           # LSP server (tower-lsp)
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── completion.rs   # Completion handler
│   │   │   ├── hover.rs        # Hover handler
│   │   │   ├── diagnostics.rs  # Push diagnostics
│   │   │   ├── goto_def.rs     # Go to definition
│   │   │   ├── rename.rs       # Rename
│   │   │   ├── symbols.rs      # Document symbols
│   │   │   └── semantic_tokens.rs
│   │   │
│   │   ├── documents.rs        # Document manager (incremental)
│   │   └── index.rs            # File index
│   │
│   └── Cargo.toml
│
├── motarjim-cache/             # Compilation cache
│   ├── src/
│   │   ├── lib.rs
│   │   ├── artifact.rs         # Cache artifact types
│   │   ├── storage.rs          # File-based cache storage
│   │   └── incremental.rs      # Incremental compilation support
│   │
│   └── Cargo.toml
│
├── motarjim-incremental/       # Incremental compilation engine
│   ├── src/
│   │   ├── lib.rs
│   │   ├── dependency.rs       # Dependency tracking
│   │   ├── change.rs           # Change detection
│   │   └── rebuild.rs          # Minimal rebuild
│   │
│   └── Cargo.toml
│
├── motarjim-config/            # Configuration loading
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs           # Config struct (serde)
│   │   ├── loader.rs           # Config file discovery
│   │   └── defaults.rs         # Default values
│   │
│   └── Cargo.toml
│
├── motarjim-fs/                # Filesystem abstraction
│   ├── src/
│   │   ├── lib.rs
│   │   ├── file.rs             # Virtual file system
│   │   └── watcher.rs          # File watcher abstraction
│   │
│   └── Cargo.toml
│
├── motarjim-profiling/         # Performance profiling
│   ├── src/
│   │   ├── lib.rs
│   │   ├── timer.rs            # Phase timing
│   │   ├── counter.rs          # Event counters
│   │   └── reporter.rs         # Report generation
│   │
│   └── Cargo.toml
│
├── motarjim-serialize/         # Serialization helpers
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ir_json.rs          # IR to/from JSON
│   │   └── config.rs           # Config serialization
│   │
│   └── Cargo.toml
│
└── motarjim-ffi/               # FFI for TypeScript integration
    ├── src/
    │   ├── lib.rs              # C-compatible API
    │   └── types.rs            # FFI-safe type conversions
    │
    └── Cargo.toml
```

### Workspace Layout

```
motarjim/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── motarjim-core/          # Facade / public API
│   ├── motarjim-ast/           # Types only
│   ├── motarjim-diag/          # Diagnostics
│   ├── motarjim-lexer/         # HTML + CSS tokenizer
│   ├── motarjim-parser/        # HTML parser
│   ├── motarjim-css/           # CSS engine
│   ├── motarjim-selectors/     # Selector engine
│   ├── motarjim-ir/            # IR construction
│   ├── motarjim-optimizer/     # Optimization passes
│   ├── motarjim-formatter/     # Code formatter
│   ├── motarjim-gen-flutter/   # Flutter generator
│   ├── motarjim-gen-compose/   # Compose generator
│   ├── motarjim-gen-swiftui/   # SwiftUI generator
│   ├── motarjim-cli/           # CLI
│   ├── motarjim-lsp/           # LSP
│   ├── motarjim-cache/         # Compilation cache
│   ├── motarjim-incremental/   # Incremental compilation
│   ├── motarjim-config/        # Configuration
│   ├── motarjim-fs/            # Filesystem
│   ├── motarjim-profiling/     # Profiling
│   ├── motarjim-serialize/     # Serialization
│   └── motarjim-ffi/           # FFI bridge
│
└── xtask/                      # Build scripts
    ├── Cargo.toml
    └── src/
        ├── main.rs             # Codegen, benchmarks, etc.
        └── codegen.rs          # AST code generation from spec
```

### Dependency Graph (Rust)

```
motarjim-diag       # No deps (standalone)
motarjim-ast        # No deps (standalone)
motarjim-config     # → diag, fs
motarjim-fs         # → diag
motarjim-serialize  # → ast, ir, config

motarjim-lexer      # → diag, ast
motarjim-parser     # → diag, ast, lexer

motarjim-selectors  # → diag, ast
motarjim-css        # → diag, ast, lexer, selectors

motarjim-ir         # → ast, css, selectors
motarjim-optimizer  # → diag, ir
motarjim-formatter  # → diag, ast

motarjim-gen-flutter  → ast, ir, formatter
motarjim-gen-compose  → ast, ir, formatter
motarjim-gen-swiftui  → ast, ir, formatter

motarjim-cache        → diag, fs, serialize
motarjim-incremental  → cache, fs, parser, css
motarjim-profiling    # No deps (standalone)

motarjim-core         → ALL crates above (facade)
motarjim-cli          → core, config, fs, profiling, cache
motarjim-lsp          → core, cache, config
motarjim-ffi           → core
```

---

## 5. Migration Roadmap

### Phase 0: Preparation (Week 1-2)
- [x] Read entire codebase (DONE)
- [x] Identify architectural problems (DONE)
- [ ] Set up Rust workspace with Cargo
- [ ] Port `motarjim-diag` — complete diagnostic system
- [ ] Port `motarjim-ast` — all type definitions
- [ ] Write golden test framework
- [ ] Set up CI for Rust

### Phase 1: Foundation (Week 3-4)
- [ ] Port `motarjim-lexer` — HTML + CSS tokenizer
- [ ] Port `motarjim-parser` — recursive descent HTML parser
- [ ] Port `motarjim-selectors` — full CSS selector parser
- [ ] Port `motarjim-css` — CSS parser + cascade
- [ ] Write comprehensive tests for all parsers
- [ ] Benchmark parsers against existing TS implementation

### Phase 2: IR + Optimizer (Week 5-6)
- [ ] Port `motarjim-ir` — builder, semantic inference, layout inference
- [ ] Port `motarjim-optimizer` — pass manager + all passes
- [ ] Port `motarjim-formatter` — CodeWriter + platform formatting
- [ ] Write IR tests with golden output
- [ ] Port existing optimizer tests

### Phase 3: Generators (Week 7-8)
- [ ] Port `motarjim-gen-flutter` — full Dart generator
- [ ] Port `motarjim-gen-compose` — full Kotlin generator
- [ ] Port `motarjim-gen-swiftui` — full Swift generator
- [ ] Write golden tests for all generators
- [ ] Verify parity with existing TS generators

### Phase 4: Integration (Week 9-10)
- [ ] Port `motarjim-core` — pipeline orchestrator
- [ ] Port `motarjim-config` — configuration system
- [ ] Port `motarjim-fs` — filesystem abstraction
- [ ] Port `motarjim-cli` — all CLI commands
- [ ] Port `motarjim-cache` — compilation cache
- [ ] Port `motarjim-incremental` — incremental compilation
- [ ] Port `motarjim-profiling` — performance monitoring

### Phase 5: LSP + FFI (Week 11-12)
- [ ] Port `motarjim-lsp` — full language server
- [ ] Port `motarjim-ffi` — C FFI for TS integration
- [ ] Port `motarjim-serialize` — JSON serialization
- [ ] Update VS Code extension to use FFI
- [ ] Update web playground to use WASM build

### Phase 6: Polish (Week 13-14)
- [ ] Remove obsolete TypeScript compiler code
- [ ] TypeScript remains only for: VS Code extension, playground, website, docs
- [ ] Run `motarjim benchmark` against real-world HTML/CSS
- [ ] Performance tuning
- [ ] Fuzz testing
- [ ] Documentation

---

## 6. Performance Opportunities

| Opportunity | Current | Target | Impact |
|------------|---------|--------|--------|
| **Zero-copy parsing** | String copies everywhere | `&str` slices + arena allocation | High |
| **Arena allocation** | Heap alloc per node | bump allocation in typed arena | High |
| **Parallel CSS matching** | Sequential selector matching | rayon-parallel per node | Medium |
| **Lazy style computation** | Always computes full style | Only compute requested properties | Medium |
| **String interning** | Duplicate strings (class names, prop names) | Interned `StringId` | High |
| **IR tree reuse** | New IR tree every time | Reuse nodes when unchanged (incremental) | High |
| **Codegen string building** | String concatenation per node | `fmt::Write` + pre-allocated buffer | Medium |
| **CSS value caching** | Re-parses values per node | Cache parsed values by property+raw | High |
| **Parse once, generate N** | Parses per-platform | Parse once, share IR, generate per platform | Medium |
| **File watching** | chokidar (JS) | `inotify`/`kqueue` native | Medium |
| **Serialization** | JSON.stringify big objects | MessagePack or custom binary format | Low |
| **SIMD CSS parsing** | N/A | SIMD-accelerated number parsing | Low |

### Estimated Performance Gains

| Scenario | Current (TS) | Target (Rust) | Factor |
|----------|-------------|---------------|--------|
| Small page (50 nodes) | ~50ms | ~1ms | 50x |
| Medium page (500 nodes) | ~200ms | ~5ms | 40x |
| Large page (5000 nodes) | ~2s | ~30ms | 66x |
| Batch (100 pages) | ~20s | ~500ms | 40x |

---

## 7. Memory Optimization Opportunities

| Opportunity | Current Problem | Solution |
|------------|----------------|----------|
| **Arena allocation** | Each `HtmlNode`/`IrNode` individually heap-allocated | Typed arena with bump allocator |
| **Small string optimization** | Many short strings on heap | Use `compact_str` or `smol_str` |
| **Thin node representation** | Large structs with many Option fields | Split into required/optional structs, use `Option<Box<...>>` for rare fields |
| **String interning** | Duplicate attribute names, class names, tag names | Interned `SymbolId` for all identifiers |
| **Compact spans** | SourceSpan with String file + start/end line/col | `u32`-based indices into source map |
| **Shared computed styles** | Each node has its own ComputedStyle | Share via `Arc<ComputedStyle>` when identical |
| **SmallVec for children** | `Vec<IrNode>` for all child counts | `SmallVec<[IrNode; 4]>` for typical small children |
| **Bitfield flags** | Boolean fields as separate `bool` | Bitflags for common properties |
| **CSS property IDs** | String-keyed property maps | Enum-based property IDs with `IntMap` |

### Memory Budget (target)

| AST Node | Current (TS est.) | Target (Rust) |
|----------|------------------|---------------|
| HtmlNode | ~200-400 bytes | 64-96 bytes |
| IrNode | ~400-800 bytes | 128-192 bytes |
| ComputedStyle | ~300-500 bytes | 64-96 bytes |
| Stylesheet (100 rules) | ~100-200 KB | 20-40 KB |
| Per-node overhead | High (V8 hidden classes) | Zero (arena) |

---

## 8. API Redesign

### Public API (motarjim-core)

```rust
// === Compiler Entry Point ===

/// Main compiler configuration
pub struct CompilerOptions {
    pub html: SourceFile,
    pub css: Option<SourceFile>,
    pub target: Target,
    pub passes: PassOptions,
    pub output: OutputOptions,
}

pub enum Target {
    Flutter,
    JetpackCompose,
    SwiftUI,
}

pub struct CompilerResult {
    pub code: GeneratedCode,
    pub stats: CompileStats,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct GeneratedCode {
    pub dart: Option<String>,    // Flutter
    pub kotlin: Option<String>,  // Jetpack Compose
    pub swift: Option<String>,   // SwiftUI
}

/// Single entry point for all compilation
pub fn compile(options: CompilerOptions) -> CompilerResult;

/// Multi-target compilation (parse once, generate N)
pub fn compile_all(options: CompilerOptions) -> CompilerResult;

/// Incremental recompilation
pub fn recompile(changes: &[FileChange], cache: &mut Cache) -> CompilerResult;

// === Individual phase APIs (for advanced use) ===

pub mod parse {
    pub fn html(source: &SourceFile) -> ParseResult<Document>;
    pub fn css(source: &SourceFile) -> ParseResult<Stylesheet>;
}

pub mod style {
    pub fn cascade(doc: &Document, sheet: &Stylesheet) -> StyledDocument;
    pub fn compute(styled: &StyledNode) -> ComputedStyle;
}

pub mod ir {
    pub fn build(doc: &StyledDocument) -> IrTree;
}

pub mod optimize {
    pub fn run(tree: IrTree, passes: &[OptimizationPass]) -> IrTree;
}

pub mod generate {
    pub fn flutter(tree: &IrTree) -> String;
    pub fn compose(tree: &IrTree) -> String;
    pub fn swiftui(tree: &IrTree) -> String;
}

// === Diagnostic API ===

pub enum Severity { Error, Warning, Info, Hint }

pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub span: Option<SourceSpan>,
    pub suggestions: Vec<String>,
    pub notes: Vec<String>,
}

pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

// === Pass API ===

pub trait CompilerPass {
    fn name(&self) -> &'static str;
    fn run(&self, tree: &mut IrTree, context: &PassContext) -> PassResult;
}

// === Plugin API ===

pub trait Plugin {
    fn name(&self) -> &'static str;
    fn register(&self, registry: &mut PluginRegistry);
}
```

### CLI Design

```
$ motarjim build [options] <input>
  --css <path>          CSS file path
  --target <platform>   flutter | compose | swiftui
  --output <path>       Output path
  --watch               Watch mode
  --format              Format output

$ motarjim watch [options] <input>
  --css <path>
  --target <platform>
  --output <path>

$ motarjim format [options] <file>
  Check/format generated code

$ motarjim doctor [options]
  Diagnostic check of environment

$ motarjim init [template]
  Create new project

$ motarjim benchmark [options]
  Run performance benchmarks

$ motarjim analyze [options] <input>
  Static analysis with suggestions

$ motarjim version
  Version info

$ motarjim help
  Help
```

---

## 9. Testing Strategy

### Unit Tests (crate-level)

| Crate | Tests | Coverage Target |
|-------|-------|-----------------|
| motarjim-lexer | Tokenize HTML/CSS -> tokens | 95% |
| motarjim-parser | Parse tokens -> AST | 95% |
| motarjim-selectors | Parse selectors, calculate specificity, match nodes | 95% |
| motarjim-css | Parse CSS, cascade, computed styles | 90% |
| motarjim-ir | Build IR from styled nodes | 90% |
| motarjim-optimizer | Each pass individually | 95% |
| motarjim-formatter | Formatting edge cases | 90% |
| motarjim-gen-flutter | Widget emission per semantic role | 90% |
| motarjim-gen-compose | Widget emission per semantic role | 90% |
| motarjim-gen-swiftui | Widget emission per semantic role | 90% |

### Golden Tests

```
tests/golden/
├── html/
│   ├── simple-div.html
│   ├── nested-elements.html
│   ├── form-with-inputs.html
│   ├── navigation-bar.html
│   ├── card-grid.html
│   ├── hero-section.html
│   ├── ecommerce-product.html
│   ├── dashboard-layout.html
│   ├── blog-article.html
│   └── complex-real-world.html
│
├── css/
│   ├── simple-rules.css
│   ├── cascade-specificity.css
│   ├── media-queries.css
│   ├── flexbox.css
│   ├── grid.css
│   ├── responsive.css
│   └── pseudo-selectors.css
│
├── output/
│   ├── flutter/          # *.dart golden files
│   ├── compose/          # *.kt golden files
│   └── swiftui/          # *.swift golden files
│
└── ir/                   # *.json golden files (debug output)
```

Golden tests auto-update with `UPDATE_EXPECT=1` env var.

### Integration Tests

```rust
#[test]
fn end_to_end_navigation_bar() {
    let result = compile(CompilerOptions {
        html: SourceFile::from_path("tests/golden/html/navigation-bar.html"),
        css: Some(SourceFile::from_path("tests/golden/css/navigation-bar.css")),
        target: Target::Flutter,
        ..Default::default()
    });
    assert!(result.diagnostics.is_empty());
    assert!(result.code.dart.unwrap().contains("AppBar"));
}
```

### Snapshot Tests

- Each generator has snapshot tests comparing full output
- Snapshots stored alongside code
- `cargo test --review` to review snapshot changes

### Fuzz Testing

```rust
// Uses cargo-fuzz
#[cfg(fuzz)]
fn fuzz_parser(data: &[u8]) {
    let source = SourceFile::from_bytes(data);
    let _ = parse::html(&source); // Must not panic
    let _ = parse::css(&source);  // Must not panic
}
```

### Benchmark Suite

```rust
// Uses criterion
fn bench_parse_small(b: &mut Bencher) {
    b.iter(|| parse::html(&SMALL_HTML));
}

fn bench_cascade_medium(b: &mut Bencher) {
    b.iter(|| cascade(&MEDIUM_DOC, &MEDIUM_CSS));
}

fn bench_generate_large(b: &mut Bencher) {
    b.iter(|| generate::flutter(&LARGE_IR));
}
```

### Property-Based Testing (proptest)

```rust
// Round-trip: parse -> serialize -> parse
proptest! {
    #[test]
    fn html_roundtrip(html in html_strategy()) {
        let doc = parse::html(&SourceFile::from_string(&html)).unwrap();
        let output = serialize::html(&doc);
        let re_doc = parse::html(&SourceFile::from_string(&output)).unwrap();
        assert_eq!(doc, re_doc);
    }
}
```

### CI Pipeline

```
cargo test
cargo test --release     # Slow tests with golden comparison
cargo bench              # Benchmark suite
cargo fuzz               # Fuzz testing (nightly)
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Summary of Key Decisions

1. **Rust owns 90%** — TypeScript only for VS Code extension, playground, website, docs
2. **Proper recursive descent parser** — no parse5, no PostCSS
3. **Strongly typed AST** — no `any`, no `Record<string, unknown>`, no dynamic maps
4. **Arena allocation** — zero-copy parsing, bump allocation
5. **Borrowing over cloning** — `&str` for all string data, `SymbolId` for identifiers
6. **Each generator is a separate crate** — no shared emitter interface
7. **Single IR** — no dual system, no legacy UiNode
8. **Each optimization is its own pass** — modular, testable, composable
9. **LSP from day one** — proper LSP with tower-lsp
10. **Incremental compilation** — dependency tracking + cache
11. **Parallelism** — rayon for CSS matching, code generation
12. **Professional diagnostics** — colored output, source snippets, suggestions, codes
