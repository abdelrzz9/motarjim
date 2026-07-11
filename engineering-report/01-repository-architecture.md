# Repository Architecture

## Workspace Structure

```
motarjim/
├── crates/                    # 31 Rust crates (compiler engine)
│   ├── motarjim-span          # Source positions, spans, file abstraction
│   ├── motarjim-errors        # Diagnostic data model
│   ├── motarjim-diag          # Predefined error codes + pretty printer
│   ├── motarjim-ast-html      # Arena-based HTML AST, ComputedStyle
│   ├── motarjim-ast-css       # CSS AST types (stylesheet, selectors, values)
│   ├── motarjim-ast-ir        # IR types (IrNode, SemanticIr, LayoutIr, TargetIr)
│   ├── motarjim-ast           # Facade re-exporting all ast-* crates
│   ├── motarjim-lexer         # HTML and CSS tokenizers
│   ├── motarjim-html          # html5ever-based HTML parser (tree-based AST)
│   ├── motarjim-parser        # Custom HTML parser + LightningCSS wrapper
│   ├── motarjim-selectors     # Selector parser + specificity calculator
│   ├── motarjim-css           # CSS cascade, selector matching, computed style
│   ├── motarjim-ir            # IR builder + inference passes (semantic, layout, responsive, accessibility)
│   ├── motarjim-optimizer     # PassManager + 6 optimization passes
│   ├── motarjim-formatter     # CodeWriter + platform-specific helpers
│   ├── motarjim-gen-flutter   # Flutter/Dart code generator
│   ├── motarjim-gen-compose   # Jetpack Compose/Kotlin code generator
│   ├── motarjim-gen-swiftui   # SwiftUI code generator
│   ├── motarjim-fs            # Filesystem abstraction (real + virtual)
│   ├── motarjim-config        # Configuration loading (JSON + TOML)
│   ├── motarjim-session       # Centralized compiler context
│   ├── motarjim-cache         # Content-addressable artifact cache
│   ├── motarjim-incremental   # Incremental compilation tracker
│   ├── motarjim-profiling     # Phase timing, telemetry bus
│   ├── motarjim-serialize     # JSON/binary serialization helpers
│   ├── motarjim-core          # Compiler facade, Pipeline, plugin/event/dag/query systems
│   ├── motarjim-cli           # CLI binary (clap)
│   ├── motarjim-lsp           # LSP server (tower-lsp)
│   ├── motarjim-ffi           # C FFI bridge
│   ├── motarjim-wasm          # WASM bindings
│   ├── motarjim-js            # ECMAScript frontend (6,972 LOC)
│   └── motarjim-test-utils    # Test harness helpers
├── apps/
│   ├── web                    # Web playground (React 18 + Vite + Monaco)
│   └── vscode-extension       # VS Code extension
├── docs/                      # 14 documentation files
├── examples/                  # 9 HTML/CSS example files
├── fuzz/                      # 5 cargo-fuzz targets
├── scripts/                   # 3 dev/test scripts
├── docker/                    # 2 Dockerfiles + 2 compose files
├── .github/                   # 12 CI/CD + community files
└── xtask/                     # Build task helpers
```

## Crate Dependency Graph

```
Foundation Layer:
  motarjim-span  (standalone)
  motarjim-errors (→ span)
  motarjim-diag   (→ errors, span)

AST Layer:
  motarjim-ast-html (standalone)
  motarjim-ast-css  (→ ast-html, span)
  motarjim-ast-ir   (→ ast-html, ast-css)
  motarjim-ast      (facade → all ast-*)

Parsing Layer:
  motarjim-lexer    (→ diag, ast)
  motarjim-html     (standalone, html5ever)
  motarjim-parser   (→ diag, ast, lexer)

CSS Engine Layer:
  motarjim-selectors (→ diag, ast)
  motarjim-css       (→ diag, ast, lexer, selectors)

IR Layer:
  motarjim-ir        (→ diag, ast, selectors)

Optimization Layer:
  motarjim-optimizer (→ diag, ast)

Output Layer:
  motarjim-formatter (→ diag, ast)
  motarjim-gen-flutter (→ ast, ir, formatter)
  motarjim-gen-compose (→ ast, ir, formatter)
  motarjim-gen-swiftui (→ ast, ir, formatter)

Infrastructure Layer:
  motarjim-fs         (→ diag)
  motarjim-config     (→ diag, fs)
  motarjim-session    (→ config, diag, fs, cache, incremental, profiling)
  motarjim-cache      (→ diag, fs, serialize)
  motarjim-incremental (→ cache, fs)
  motarjim-profiling  (standalone)
  motarjim-serialize  (→ ast)

Integration Layer:
  motarjim-core      (→ ALL above)
  motarjim-cli       (→ core, config, fs)
  motarjim-lsp       (→ core, cache, config)
  motarjim-ffi       (→ core, config)
  motarjim-wasm      (→ core, config)
  motarjim-js        (→ diag, lexer, span)
  motarjim-test-utils (→ core, diag, fs, config)
```

## Data Flow

```
Source HTML + CSS
     │
     ▼
motarjim-fs (read files — real or virtual)
     │
     ▼
motarjim-parser (HTML via custom parser; CSS via LightningCSS wrapper)
     │
     ▼
motarjim-css + motarjim-selectors
  ├── Selector matching (simplified — no combinator traversal)
  ├── Cascade resolution (specificity, !important, source order)
  └── Computed style construction (HashMap<NodeId, ComputedStyle>)
     │
     ▼
motarjim-ir
  ├── SemanticAnalyzer (HTML tag + attributes → SemanticIr, 41 variants)
  ├── LayoutInferrer (CSS computed style → LayoutIr, 17 variants)
  ├── ResponsiveInferrer (STUB — always returns empty)
  └── AccessibilityInferrer (ARIA attributes → metadata)
     │
     ▼  IrTree
     │
motarjim-optimizer — PassManager
  ├── RemoveEmptyNodes
  ├── CollapseWhitespace
  ├── MergeAdjacentText
  ├── RemoveUnusedStyles
  ├── FlattenNestedContainers
  └── InlineConstantValues
     │
     ▼  Optimized IrTree
     │
motarjim-gen-flutter / gen-compose / gen-swiftui
  └── Platform source code (Dart / Kotlin / Swift)
```

## Compilation Targets (Interfaces)

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
```

## Key Architectural Observations

### 1. Two Parallel HTML Parsers

The codebase has two independent HTML parsers:
- **Custom parser** (`motarjim-parser/src/html.rs`, 648 LOC) — recursive descent over tokens from `motarjim-lexer`. Limited error recovery, no attribute tokens from lexer (forces string re-scanning).
- **html5ever parser** (`motarjim-html`, 2,118 LOC) — full HTML5 spec compliance via Servo's html5ever. Produces a **tree-based** AST incompatible with the custom parser's **arena-based** AST.

They produce incompatible AST types and are not integrated. This appears to be a migration in progress.

### 2. AST Duplication

- `motarjim-ast-css/src/selector.rs` has its own `Selector`/`SimpleSelector` types with `matches()` methods, duplicated from `motarjim-selectors/types.rs`.
- `motarjim-ast-ir/src/layout.rs` has a `LayoutStrategy` enum that duplicates `LayoutIr` but is never used (dead code).

### 3. Feature-Gated Architecture

Key features are gated behind Cargo features and disabled by default:
- `dag` — parallel DAG scheduler (requires rayon)
- `cancellation` — cooperative cancellation tokens
- `events` — lifecycle events and EventBus
- `plugin-system` — dynamic generator dispatch
- `query-system` — query cache (requires dashmap)

The default compilation path is the simpler sequential pipeline.

### 4. Feature Gates for Generators Defined but Unused

The `gen-flutter`, `gen-compose`, `gen-swiftui` features in `motarjim-core/Cargo.toml` are defined but never checked with `#[cfg]` in source code. They serve only as documentation.

### 5. Platform Modules vs Hand-Coded Output

`motarjim-formatter` has platform-specific modules (`dart::write_class`, `kotlin::write_fun`, `swift::write_struct`) but the generators don't use them — they hand-code output via the generic `CodeWriter`.

### 6. Pipeline Wired in Series

The DAG scheduler (`motarjim-core/src/dag.rs`, 1,475 LOC) supports parallel compilation with 13 phases across 7 levels, but it's behind the `dag` feature flag (disabled by default). The default `Compiler::compile()` runs phases sequentially with no parallelism.
