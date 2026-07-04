# Changelog

## [Unreleased]

### Added

- **`motarjim-js` crate**: a new JavaScript front end — lexer, AST, recursive-descent
  parser, `Visitor` trait, best-effort semantic analysis (duplicate declarations,
  `const` reassignment, undeclared-variable warnings), DOM event binding extraction
  (`addEventListener`/`on*`), and a `Transform` trait with a first concrete transform
  (template literals → string concatenation). See [docs/javascript.md](docs/javascript.md).
- New JavaScript diagnostic code range `E0700`-`E0799` in `motarjim-diag`.
- `motarjim check` now routes `.js`/`.mjs`/`.jsx` files to `motarjim-js` instead of
  the HTML/CSS compiler pipeline.

### Fixed

- Workspace-wide `cargo clippy --workspace --all-features -- -D warnings` is clean
  again (missing docs on feature-gated modules/fields in `motarjim-fs`, `motarjim-ir`,
  `motarjim-optimizer`, `motarjim-css`, the three generator crates, and `motarjim-core`;
  a `clippy::large_enum_variant` in the new `motarjim-js` AST; a few `clippy::all`
  nits in `motarjim-core`).
- `docs/roadmap.md` rewritten — it described the retired TypeScript/`parse5`/PostCSS
  pipeline instead of the current Rust workspace. Most other files under `docs/`
  still need the same treatment; see the note at the top of `docs/roadmap.md`.

- **Double optimization removed**: Generators no longer call `optimize()` internally.
  Previously, the CLI optimized the IR and then each generator re-optimized it
  silently. Optimization now happens exactly once, at the pipeline level (CLI and
  test helper). If adding a new generate() call site, ensure the IR is pre-optimized.

- **`getValue()` dead code removed**: The fallback `node.properties.value` was never
  set by the IR (it's always deleted in `styledNodeToIr` before `createIrNode`).
  Simplified to `node.value ?? ''`.

### Changed

- **Extracted `@motarjim/generator-core`**: Shared traversal (`walkTree`),
  `NodeEmitter` interface, `countNodes`, `escapeString`/`escapeStringExtra`,
  `findTextLabel`, `getNonTextChildren`. Each platform generator implements
  `NodeEmitter` instead of duplicating the tree-walk switch. 130+ lines of
  duplication removed across the three generators.
