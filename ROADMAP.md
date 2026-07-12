# Roadmap

## Current Status (v0.2.0)

Motarjim is a functional static site generator with a complete CSS engine. The core pipeline (parse frontmatter → render templates → resolve styles → generate output) is implemented and tested.

### Current Capabilities

- **Multi-format content** — Markdown and HTML source files with YAML/TOML frontmatter
- **Template engine** — `{{ ... }}` variable expansion with iteration and conditionals
- **CSS engine**:
  - Selector matching (class, ID, tag, universal, attribute, pseudo-class, pseudo-element, combinators)
  - Cascade resolution with origin, specificity, source-order sorting
  - Property application with shorthand expansion and vendor prefixing
  - CSS custom properties (`--*`) with `var()` resolution and cycle detection
  - `calc()` arithmetic evaluation with unit conversion and percentage resolution
  - Media query evaluation (`min-width`, `max-width`, `prefers-color-scheme`, boolean combinators)
  - Structured grid layout parsing (`grid-template-*`, `grid-area`, `grid-column`, `grid-row`)
  - Positioning offsets (`top`, `right`, `bottom`, `left`, `inset`)
  - Animation properties (`animation-name`, `animation-duration`, `animation-timing-function`, `@keyframes` collection)
- **Multi-format output** — HTML, RSS/Atom feeds, JSON, plaintext
- **Asset pipeline** — Co-located asset copying preserving directory structure
- **Dev server** — Built-in HTTP server with live reload
- **CLI** — `init`, `build`, `serve`, `check` commands
- **Testing** — 500+ unit/integration tests across all crates

## Short-Term Goals (Next 1-3 Months)

### CSS Engine Enhancements

- [ ] `min()`, `max()`, `clamp()` math functions
- [ ] `env()` and `attr()` function support
- [ ] `@container` query support
- [ ] `:has()`, `:where()`, `:is()`, `:not()` with full selector lists
- [ ] CSS source maps for debugging
- [ ] Custom `@font-face` rule resolution

### Build Performance

- [ ] Incremental builds — Only rebuild changed files
- [ ] Parallel page processing across available cores
- [ ] Cached stylesheet parsing across pages

### Features

- [ ] Sass/SCSS preprocessing pipeline
- [ ] Syntax highlighting in generated output (built-in)
- [ ] Custom output formats via plugins
- [ ] `motarjim check` with enhanced diagnostics (broken links, missing assets, etc.)

## Medium-Term Goals (3-6 Months)

### Template Engine

- [ ] Template inheritance and blocks (`{% extends %}`, `{% block %}`)
- [ ] Custom template filters and functions
- [ ] Partials/include support

### Content Management

- [ ] Draft/preview workflow with `motarjim serve --drafts`
- [ ] Pagination for blog listings
- [ ] Tag/category index generation
- [ ] Sitemap.xml generation
- [ ] Image optimization pipeline (resize, format conversion, responsive srcset)

### Developer Experience

- [ ] File watcher with debounced rebuild (`motarjim watch`)
- [ ] Live reload with WebSocket (replace polling)
- [ ] Rich error output with suggestion hints
- [ ] `motarjim doctor` environment diagnostics

## Long-Term Vision (6-12 Months)

### Feature Maturity

- **Plugin ecosystem** — Custom generators, transformers, and output formats via a stable plugin API
- **Tailwind CSS integration** — First-class Tailwind utility class support
- **Internationalization (i18n)** — Multi-language site generation from a single source tree
- **Search** — Built-in full-text search index generation (JSON-based, no external deps)

### Performance Targets

| Scenario | Current | Target |
|----------|---------|--------|
| Small site (10 pages) | <10ms | <5ms |
| Medium site (100 pages) | <50ms | <20ms |
| Large site (1000 pages) | <500ms | <200ms |

### Community & Ecosystem

- [ ] GitHub Actions CI/CD template
- [ ] Starter templates and theme marketplace
- [ ] Documentation site built with motarjim itself
- [ ] Third-party plugin registry
- [ ] Regular release cadence

## How to Contribute

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

Priority areas for community contribution:

1. **Sass/SCSS pipeline** — Integrate a Rust Sass compiler
2. **Template engine enhancements** — Inheritance blocks, filters
3. **Plugin API** — Design and implement a stable plugin system
4. **Documentation** — Tutorials, guides, API docs
5. **Benchmarks** — Criterion benchmarks for the pipeline
6. **Tests** — Add fuzz testing for the CSS engine
7. **Themes** — Example site themes and starter templates
