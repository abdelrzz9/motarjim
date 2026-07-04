# Roadmap

This roadmap describes the **Rust engine** (`crates/`), which is the single
source of truth for parsing, analysis, and code generation. TypeScript is a
client of the Rust engine only (web playground, VS Code extension, website);
it does not contain compiler logic.

## Completed

### Rust workspace migration

The compiler was rewritten from a TypeScript/`parse5`/PostCSS pipeline into a
Rust `cargo` workspace of single-responsibility crates:

- [x] `motarjim-diag` — diagnostics, source spans, colored terminal output
- [x] `motarjim-ast` — shared AST/IR/style type definitions
- [x] `motarjim-lexer` / `motarjim-parser` — zero-copy HTML and CSS tokenizers/parsers
- [x] `motarjim-selectors` — CSS selector parsing, specificity, matching
- [x] `motarjim-css` — CSS cascade, computed styles, typed values
- [x] `motarjim-ir` — semantic/layout/target intermediate representation
- [x] `motarjim-optimizer` — IR optimization passes (merge text, flatten, prune, dedupe)
- [x] `motarjim-formatter` — target-language code writer
- [x] `motarjim-gen-flutter` / `motarjim-gen-compose` / `motarjim-gen-swiftui` — code generators
- [x] `motarjim-core` — compiler facade, event bus, query/cache system, compilation DAG
- [x] `motarjim-cli` — `compile` / `watch` / `init` / `check` commands
- [x] `motarjim-lsp` — language server scaffold (`tower-lsp`)
- [x] `motarjim-cache` / `motarjim-incremental` — compilation caching and incremental rebuilds
- [x] `motarjim-config` / `motarjim-fs` / `motarjim-profiling` / `motarjim-serialize` / `motarjim-ffi`
- [x] `motarjim-wasm` — WebAssembly bindings scaffold
- [x] `xtask` — workspace automation
- [x] Fuzz targets for the HTML/CSS lexers and parsers (`fuzz/`)
- [x] Criterion benchmarks per parser/crate
- [x] Workspace-wide `clippy::all`/`clippy::pedantic`/`clippy::nursery` lints,
      `#![deny(missing_docs)]`, `#![forbid(unsafe_code)]`

### JavaScript support (`motarjim-js`)

- [x] Zero-copy lexer/tokenizer (reuses `motarjim_lexer::Cursor`)
- [x] Recursive-descent parser with precedence climbing for expressions
- [x] AST covering variables, functions, arrow functions, template literals
      (including nested interpolations), `import`/`export`, control flow
- [x] `Visitor` trait for read-only AST traversal
- [x] Best-effort semantic analysis: duplicate `let`/`const` declarations,
      `const` reassignment, undeclared-variable warnings
- [x] DOM event binding extraction (`addEventListener`, `on*` assignment)
- [x] `Transform` trait + a first transform (template literals → string concatenation)
- [x] Wired into `motarjim check` for `.js`/`.mjs`/`.jsx` files

See [javascript.md](javascript.md) for details and current syntax coverage.

## In Progress / Next Up

### Rust engine

- [ ] Wire `motarjim-js` DOM event bindings into `motarjim-ir` so a `click`
      listener can drive a generated `onPressed`/`onClick`/`.onTapGesture`
      handler — the seam exists (`find_dom_event_bindings`) but nothing
      downstream consumes it yet
- [ ] CSS value mapping: colors, padding/margin shorthands, typography, per platform
- [ ] Responsive design generation from media query hints already captured in the IR
- [ ] Advanced CSS selectors: `:nth-child()`, `:not()`, `:has()`, pseudo-elements
- [ ] Flesh out `motarjim-lsp` handlers (hover, completion, rename, semantic tokens)
- [ ] Flesh out `motarjim-wasm`'s public API (`parse`/`compile`/`format`/`lint`/`ast`)
- [ ] `motarjim watch` is currently a stub (`cmd_watch` prints "not yet implemented")

### Documentation

- [ ] Most of `docs/*.md` (architecture, parser, cli, pipeline, css-analyzer,
      semantic-analyzer, ir, optimizer, generator-core, the per-platform
      generator docs, ai-enhancement, benchmarks) still describes the retired
      TypeScript/`parse5`/PostCSS pipeline and needs a full rewrite against
      the Rust crates. Treat anything under `docs/` other than this file and
      `javascript.md` as unverified until it's been re-audited.

### Tooling and CI

- [ ] `cargo deny`, `cargo machete`, `cargo nextest`, coverage reporting in CI
- [ ] Markdown lint / spellcheck / license check in CI
- [ ] Docker (`Dockerfile`, `docker-compose.yml`) for local development
- [ ] `motarjim.toml` as the primary config format (JSON config exists today)

### Web and editor clients

- [ ] `apps/playground` and `apps/website` are plain Vite + vanilla JS; the
      planned modern rewrite (component framework, typed API client, state
      management, Monaco editor, AST/diagnostics panels) has not started
- [ ] `packages/vscode-extension` is a scaffold; LSP wiring, diagnostics,
      and preview commands are not yet implemented

## Future Considerations

- [ ] React Native / UIKit / WinUI / Jetpack Views (raw XML) generator targets
- [ ] Plugin system for third-party generators (`Generator` trait already
      exists in `motarjim-core::plugin`; no external plugin loading yet)
- [ ] Property-based/fuzz testing for `motarjim-js`, mirroring the existing
      HTML/CSS fuzz targets

## How to Contribute

See [contributing.md](contributing.md) for development setup and guidelines.
Good first areas, roughly in order of leverage:

1. Wiring `motarjim-js` DOM events into the IR/generators
2. CSS value mapping (colors, spacing, typography)
3. Rewriting the stale TypeScript-era docs listed above
4. `motarjim watch` (file watching + incremental recompilation)
