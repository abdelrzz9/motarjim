# Mortarjim Engineering Report — Executive Summary

**Project:** Mortarjim — HTML/CSS → Native UI Code compiler
**Version:** v0.1.0 | **Language:** Rust | **Workspace:** 31 crates | **Total LOC:** ~40,000+

## Overview

Mortarjim is a source-to-source compiler that translates HTML and CSS into native UI code for Flutter (Dart), Jetpack Compose (Kotlin), and SwiftUI (Swift). It follows a classic multi-stage compiler architecture with discrete, composable passes built as a Rust workspace of single-responsibility crates.

## Current Status

The project is in **early alpha (v0.1.0)** with a complete but immature pipeline. The architecture is well-conceived with strong safety conventions (`#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`, `#![warn(clippy::all)]`) and professional tooling (Criterion benchmarks, cargo-fuzz, proptest, insta snapshots).

### What Works

- HTML parsing (via custom parser; html5ever backend exists separately)
- CSS parsing (via LightningCSS wrapper, comprehensive)
- Selector parsing and specificity calculation
- Cascade resolution (specificity, !important, source order, inheritance)
- Computed style construction (50+ CSS properties)
- IR construction with semantic/layout/accessibility inference
- 6 optimization passes (text merging, empty removal, flattening, dedup)
- Code generation for Flutter, Compose, and SwiftUI (structural skeleton)
- JavaScript frontend (lexer, parser, semantic analysis — 6,972 LOC)
- Diagnostics with error codes (E0001-E0799), colored terminal output
- CLI with 4 commands (compile, watch/stub, init, check)
- LSP server (working diagnostics, stubs for other features)
- WASM bindings, C FFI bridge
- Configuration (JSON and TOML)
- Profiling infrastructure (phase timing, telemetry bus)
- Incremental compilation engine (not wired into pipeline)
- Artifact cache (not wired into pipeline)
- Comprehensive CI/CD (8 GitHub Actions workflows)

### What's Missing / Broken

| Issue | Severity |
|-------|----------|
| Dual HTML parsers (custom + html5ever) not integrated | Critical |
| CSS selector matching ignores combinators (descendant, child, sibling) | Critical |
| CSS variables (`var()`) not resolved | Critical |
| Media queries never evaluated (all rules unconditionally included) | Critical |
| Responsive IR is a complete stub | Critical |
| Generators have bugs producing invalid code (SwiftUI, Flutter table cell) | High |
| Most CSS properties not mapped to generated code | High |
| Incremental compilation and caching not wired into pipeline | High |
| No golden/snapshot tests for generated output | High |
| DAG parallel scheduler feature-gated (disabled by default) | Medium |
| Arena allocation not implemented (individual heap allocs for all nodes) | Medium |
| No string interning (no global SymbolId) | Medium |
| LSP completion/hover/definition/semantic-tokens are stubs | Medium |
| CLI watch mode is a stub | Medium |
| Source maps not implemented | Medium |

## Overall Scores

| Dimension | Score | Rationale |
|-----------|:-----:|-----------|
| **Architecture** | 7/10 | Well-conceived crate split, but dual-parser, dead code, and unused abstractions |
| **Code Quality** | 7/10 | Strong Rust idioms, safety culture, consistent linting. Inconsistent documentation. |
| **Performance** | 5/10 | Good foundations (SmolStr, SmallVec, rayon), missing arenas, string interning, lazy computation |
| **Maintainability** | 6/10 | Modular crates help; dead code, unused APIs, feature gate complexity hurt |
| **Scalability** | 4/10 | Single-threaded by default, no incremental compilation wired, no cache wired |
| **Production Readiness** | 3/10 | Missing error recovery quality, source maps, golden tests, CLI polish, LSP features |
| **Overall** | **5.5/10** | Early alpha with strong foundation. 10-16 months from 1.0. |

## Estimated Path to 1.0

| Phase | Time | Key Deliverable |
|-------|------|-----------------|
| Core Stabilization | 2-3 months | html5ever integration, combinator matching, incremental/cache wiring, generator bug fixes |
| CSS Engine Completion | 2-3 months | Variables, media queries, grid parsing, positioning, calc(), animations |
| IR Completion | 1-2 months | Responsive breakpoints, aria-labelledby resolution, TargetIr population |
| Generator Completion | 2-3 months | Full CSS property mapping, golden tests, event binding wiring |
| Performance | 1-2 months | Arena allocation, string interning, lazy styles, parallel default |
| CLI & Tooling | 1-2 months | Watch mode, multi-file, source maps, optimization levels |
| LSP Completion | 1 month | Completion, hover, definition, semantic tokens, code actions |
| Production Release | 1 month | Documentation refresh, publish, security audit |

**Total: 10-16 months to 1.0**
