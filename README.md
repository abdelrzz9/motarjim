# Motarjim

A modular static site generator written in Rust, built with cleanliness, performance, and extensibility in mind.

## Key features

- **Multi-format input** — Write content in Markdown or HTML source files with YAML/TOML frontmatter
- **Multi-format output** — Generate HTML, XML (RSS/Atom), JSON, and plaintext from a single build
- **Powerful template engine** — Built-in template-driven rendering with context injection
- **CSS engine** — Full CSS processing pipeline including cascade resolution, selector specificity matching, vendor prefix handling, CSS custom properties (variables) with `var()` resolution, `calc()` evaluation, structured grid parsing, positioning offset support, and media query evaluation
- **Theme system** — Layered theme inheritance with built-in `base` and `motarjim` themes; user themes override selectively
- **Asset pipeline** — Co-located asset copying with content directory mirroring
- **Dev server** — Built-in HTTP server with live reload for rapid iteration
- **RSS/Atom feeds** — Automatic feed generation with configurable metadata and filtering
- **Configurable pipelines** — Choose which output formats to generate; configure URLs, author info, and rendering options
- **CLI-first** — Full command-line interface with `init`, `build`, `serve`, and `check` commands

## Installation

**System requirements:** Rust 1.70 or later.

```bash
cargo install motarjim
```

Or build from source:

```bash
git clone https://github.com/your-org/motarjim
cd motarjim
cargo build --release
```

## Quick start

```bash
# Create a new site
motarjim init my-site

# Build the site
cd my-site
motarjim build

# Serve with live reload
motarjim serve --port 8080
```

## Architecture

Motarjim is organized as a **Cargo workspace** with these crates:

| Crate | Purpose |
|---|---|
| `motarjim-core` | Orchestration orchestrator — config loading, pipeline wiring, subcommand dispatch |
| `motarjim-config` | Configuration loading, validation, and defaults |
| `motarjim-cli` | CLI argument parsing and user-facing commands |
| `motarjim-ast-html` | HTML/CSS AST types (node tree, computed style, animation, grid) |
| `motarjim-frontmatter` | Frontmatter parsing (YAML/TOML/JSON) |
| `motarjim-output` | Output generation — HTML, XML, JSON, plaintext |
| `motarjim-css` | CSS selector matching, cascade resolution, property application, vendor prefixing, variable resolution, `calc()` evaluation, media queries, and `@keyframes` collection |
| `motarjim-assets` | Asset copying and content directory management |

### Pipeline stages

```
[Content Files] → [Frontmatter Parse] → [Template Render] → [Style Resolve] → [Output Generate]
```

The **Style Resolve** stage handles:
1. **Selector matching** — specificity calculation across element, class, ID, attribute, and pseudo-class selectors
2. **Cascade resolution** — inline styles, external stylesheets, author vs. user-agent precedence
3. **Property application** — shorthand expansion, vendor prefix insertion, type coercion
4. **CSS variable resolution** — `var(--name)` substitution with custom property registry and cycle detection
5. **`calc()` evaluation** — recursive-descent arithmetic with unit conversion and percentage-to-px resolution
6. **Media query evaluation** — viewport- and preference-based condition matching (min-width, max-width, prefers-color-scheme, boolean combinators)
7. **Grid layout parsing** — structured representation of `grid-template-columns`, `grid-template-rows`, `grid-area`, `grid-column`, `grid-row` values
8. **Positioning offset handling** — `top`, `right`, `bottom`, `left`, `inset` property resolution
9. **Animation property support** — `animation-name`, `animation-duration`, `animation-timing-function`, and shorthand; `@keyframes` collection
10. **Vendor prefix generation** — `-webkit-`, `-moz-`, `-ms-` prefix insertion for modern CSS features

## Configuration

Motarjim uses a `motarjim.toml` file for site configuration:

```toml
[site]
title = "My Site"
base_url = "https://example.com"
author = "Your Name"

[build]
output_dir = "public"
input_dir = "content"

[build.viewport]
width = 1024
height = 768
prefers_color_scheme = "light"

[feeds.rss]
enabled = true
title = "My Site RSS Feed"

[feeds.atom]
enabled = true
```

## Development

```bash
# Run tests
cargo test --workspace

# Build
cargo build

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all
```

## License

MIT
