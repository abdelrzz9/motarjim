# Roadmap

## Current Status (v0.1.0)

The motarjim compiler has been rewritten from a TypeScript/parse5/PostCSS prototype into a production-quality Rust workspace of single-responsibility crates. The core compiler pipeline (parse → style → IR → optimize → generate) is complete and functional.

### Current Capabilities

- **HTML parsing** — Recursive-descent parser with error recovery
- **CSS parsing** — Full CSS 3 parser with cascade, specificity, and typed values
- **Selector matching** — Class, ID, tag, universal, attribute, pseudo-class, pseudo-element
- **IR construction** — Three-layer IR (SemanticIR / LayoutIR / TargetIR) with inference
- **Optimization** — 7 optimization passes via modular PassManager
- **Code generation** — Flutter (Dart), Jetpack Compose (Kotlin), SwiftUI (Swift)
- **JavaScript front end** — Lexer, parser, AST, semantic analysis, DOM event extraction
- **Diagnostics** — Professional error codes (E0001-E0799) with spans and severities
- **CLI** — `compile`, `watch` (stub), `init`, `check` commands
- **LSP** — Language server scaffold with tower-lsp
- **WASM** — WebAssembly bindings scaffold
- **Caching** — Artifact cache and incremental compilation engine
- **Profiling** — Phase timing and performance reporting
- **Configuration** — JSON and TOML config file support
- **Plugin system** — Generator trait, Plugin trait, PluginRegistry
- **Testing** — 493+ unit/integration tests, fuzz targets, Criterion benchmarks
- **Linting** — Workspace-wide clippy::all, deny(missing_docs), forbid(unsafe_code)

## Short-Term Goals (Next 3 Months)

### Rust Engine

- [ ] Wire `motarjim-js` DOM event bindings into `motarjim-ir` — connect `find_dom_event_bindings()` output to generators so `click` → `onPressed`/`onClick`/`.onTapGesture`
- [ ] CSS value mapping — Colors, padding/margin shorthands, typography, border values converted to platform-native expressions (e.g., `Colors.blue` in Dart, `Color.Blue` in Kotlin)
- [ ] Responsive design generation — Media query hints already captured in IR; generators should emit responsive widgets (e.g., `LayoutBuilder` in Flutter, `BoxWithConstraints` in Compose)
- [ ] Advanced CSS selectors — `:nth-child()`, `:not()`, `:has()`, pseudo-elements
- [ ] `motarjim watch` — File watcher with debounced recompilation
- [ ] `motarjim compile` for multiple files — Accept directory input, compile all HTML files
- [ ] Source maps — Map generated code locations back to source HTML/CSS

### LSP

- [ ] Diagnostics handler — Push compiler diagnostics on file save
- [ ] Completion handler — CSS property/value autocompletion
- [ ] Hover handler — Documentation on hover for CSS properties
- [ ] Go to definition — Navigate from CSS class references to `<style>` blocks
- [ ] Semantic tokens — Syntax highlighting via compiler lexer

### WASM

- [ ] Full public API — `parse()`, `compile()`, `format()`, `lint()`, `ast()` methods
- [ ] TypeScript type definitions — Proper `.d.ts` for the WASM API
- [ ] npm package — Publish `motarjim-wasm` to npm

### Web & Editor

- [ ] Playground rewrite — Monaco editor, real-time compilation, AST viewer, diagnostics panel
- [ ] Website redesign — Modern documentation site with search, examples, and API reference
- [ ] VS Code extension — LSP wiring, diagnostics view, preview panel
- [ ] Dark/light theme in playground

### Tooling & CI

- [ ] `cargo deny` — License and advisory checking
- [ ] `cargo machete` — Unused dependency detection
- [ ] `cargo nextest` — Faster, more reliable test execution
- [ ] Coverage reporting — `cargo llvm-cov` in CI
- [ ] Markdown lint / spellcheck / license check
- [ ] Docker image for CI/CD

## Medium-Term Goals (3-6 Months)

### New Platforms

- [ ] **React Native** generator — JavaScript/TypeScript with JSX output
- [ ] **UIKit** generator — Objective-C/Swift for iOS native
- [ ] **Jetpack Views** generator — XML-based Android layouts
- [ ] **Tauri** generator — Optimized web content for Tauri apps

### Language Features

- [ ] CSS Grid layout support
- [ ] CSS Animations and transitions
- [ ] CSS Variables (custom properties)
- [ ] CSS `@container` queries
- [ ] SCSS/SASS preprocessing
- [ ] HTML `<template>` and Web Component support

### Performance

- [ ] Arena allocation — Typed arenas with bump allocators for all AST/IR nodes
- [ ] Zero-copy parsing — `&str` slices throughout, no string copies
- [ ] String interning — Interned `SymbolId` for all identifiers
- [ ] Parallel CSS matching — rayon-parallel selector matching
- [ ] Lazy style computation — Only compute requested properties
- [ ] Compilation DAG — Replace sequential pipeline with a DAG scheduler

### Incremental Compilation

- [ ] File-level dependency tracking
- [ ] Minimal rebuild — Only recompile affected phases
- [ ] Persisted cache — Cache artifacts across sessions
- [ ] Query system — Full Salsa-inspired query cache

### Community

- [ ] Plugin ecosystem — Publish 2-3 third-party generators as examples
- [ ] `motarjim.toml` as primary config format
- [ ] Interactive `motarjim init` with project templates
- [ ] `motarjim format` — Format generated code with platform tools
- [ ] `motarjim doctor` — Diagnostic check of environment
- [ ] `motarjim analyze` — Static analysis with suggestions

## Long-Term Vision (6-12 Months)

### Platform Parity

Target state: The generated code is indistinguishable from hand-written platform code. CSS properties map to fully typed native values. Responsive design produces adaptive layouts. Event handlers from JavaScript produce real platform event handlers.

### New Use Cases

- **Design-to-code pipeline** — Accept designs exported as HTML/CSS from Figma, Sketch, or web design tools
- **Component library generation** — Generate entire design systems from HTML/CSS component specifications
- **Migration tool** — Convert existing web UIs to native apps with human-readable output

### Performance Targets

| Scenario | Current | Target |
|----------|---------|--------|
| Small page (50 nodes) | ~2ms | <1ms |
| Medium page (500 nodes) | ~10ms | <5ms |
| Large page (5000 nodes) | ~98ms | <30ms |
| Batch (100 pages) | ~1s | <500ms |

### Advanced Features

- **Multi-file projects** — Compile HTML/CSS as a project, not single files
- **Third-party CSS frameworks** — Bootstrap, Tailwind CSS support via plugins
- **Source-to-source debugging** — Step through generated code mapped to original HTML/CSS
- **AI-assisted generation** — Enhanced semantic detection via Ollama/local models
- **Web playground with sharing** — Shareable compilation URLs

## Community Goals

### Documentation

- [ ] Complete Rust doc coverage on all public API surfaces
- [ ] Tutorial series: "Converting a Real App from Web to Native"
- [ ] Video guides for visual learners
- [ ] FAQ and troubleshooting expansion

### Ecosystem

- [ ] GitHub Actions for CI/CD integration
- [ ] Pre-commit hook for HTML/CSS validation
- [ ] Editor integrations beyond VS Code (JetBrains, Neovim, Helix)
- [ ] Community plugin registry

### Governance

- [ ] Contributor ladder (first-time, regular, maintainer)
- [ ] RFC process for significant changes
- [ ] Regular release cadence
- [ ] Security disclosure process

## How to Contribute

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines. 

Priority areas for community contribution:

1. **Wiring `motarjim-js` DOM events** into IR/generators — The extraction code exists; downstream consumption is the gap.
2. **CSS value mapping** — Colors, spacing, typography per platform
3. **Responsive design** — Media query → platform code
4. **Documentation** — Rewriting stale TypeScript-era docs
5. **`motarjim watch`** — File watching + incremental recompilation
6. **Fuzz targets** — Adding `motarjim-js` fuzz targets
7. **Third-party generators** — React Native, UIKit, others
