# Architecture

## High-Level Overview

Motarjim is a **static site generator** that transforms Markdown and HTML source files into static web content (HTML, RSS/Atom feeds, JSON, plaintext). It follows a sequential pipeline architecture with a dedicated CSS engine for style resolution.

The compiler is built as a **Rust workspace** of single-responsibility crates. Each crate is independently publishable, testable, and benchmarkable.

```
Content Files (Markdown/HTML + Frontmatter)
    │
    ▼
┌──────────────────────────────────────────┐
│            Pipeline Stages               │
│                                          │
│  ┌──────────┐  ┌──────────┐             │
│  │  Parse   │  │  Render  │             │
│  │ Front-   │──▶ Template │             │
│  │ matter   │  │ + HTML   │             │
│  └──────────┘  └────┬─────┘             │
│                     │                    │
│                     ▼                    │
│  ┌────────────────────────────────────┐  │
│  │         Style Resolve              │  │
│  │  ┌──────────┐  ┌──────────┐       │  │
│  │  │ Selector │  │ Cascade  │       │  │
│  │  │ Match    │──▶ Resolve  │       │  │
│  │  └──────────┘  └────┬─────┘       │  │
│  │                     │             │  │
│  │                     ▼             │  │
│  │  ┌──────────┐  ┌──────────┐      │  │
│  │  │ Variable │  │ calc()   │      │  │
│  │  │ Resolve  │  │ Eval     │      │  │
│  │  └──────────┘  └──────────┘      │  │
│  │  ┌──────────┐  ┌──────────┐      │  │
│  │  │ Media    │  │ Vendor   │      │  │
│  │  │ Query    │  │ Prefix   │      │  │
│  │  └──────────┘  └──────────┘      │  │
│  └────────────────────────────────────┘  │
│                     │                    │
│                     ▼                    │
│  ┌──────────────────────────────────┐   │
│  │         Output Generation        │   │
│  │  ┌──────┐ ┌──────┐ ┌──────┐     │   │
│  │  │ HTML │ │ XML  │ │ JSON │     │   │
│  │  │      │ │/Feed │ │      │     │   │
│  │  └──────┘ └──────┘ └──────┘     │   │
│  └──────────────────────────────────┘   │
└──────────────────────────────────────────┘
    │
    ▼
Static Site (public/)
```

## Crate Dependency Graph

```
motarjim-ast-html         (standalone - HTML/CSS AST types)
motarjim-config           (standalone - config loading and validation)

motarjim-frontmatter      → motarjim-ast-html
motarjim-css              → motarjim-ast-html

motarjim-templates        → motarjim-ast-html
motarjim-output           → motarjim-ast-html, motarjim-templates
motarjim-assets           (standalone)

motarjim-core             → ALL crates above (orchestration)
motarjim-cli              → motarjim-core, motarjim-config
```

## Pipeline Stages

### Stage 1: Parsing

**Input:** Source files (`.md`, `.html`) with YAML/TOML frontmatter
**Output:** Parsed HTML document tree + frontmatter metadata
**Crates:** `motarjim-frontmatter`, `motarjim-ast-html`

Source files are split into two parts:
- **Frontmatter** — YAML or TOML metadata block between `---` delimiters
- **Content** — Markdown rendered to HTML, or raw HTML

The frontmatter parser produces typed metadata (title, date, tags, layout, draft status, custom fields) available as template variables.

### Stage 2: Template Rendering

**Input:** HTML document + frontmatter metadata
**Output:** Fully rendered HTML document (template-expanded)
**Crates:** `motarjim-templates`

Template tags (`{{ ... }}`) in the source content are expanded using context variables derived from frontmatter and site configuration. Templates support variable substitution, iteration, and conditional blocks.

### Stage 3: Style Resolution

**Input:** HTML document + CSS stylesheets
**Output:** `HashMap<NodeId, ComputedStyle>`
**Crates:** `motarjim-css`, `motarjim-ast-html`

The CSS engine performs eight sub-passes:

#### 3a. Selector Matching

For each HTML node, find all matching CSS rules. Selectors supported:
- **Simple:** element, class (`.`), ID (`#`), universal (`*`)
- **Compound:** attribute (`[attr]`, `[attr=val]`), pseudo-class (`:hover`, `:first-child`), pseudo-element (`::before`)
- **Combinators:** descendant (space), child (`>`), adjacent sibling (`+`), general sibling (`~`)
- **Lists:** comma-separated selector groups

Specificity is calculated per the W3C spec (inline > ID > class/attribute/pseudo-class > element/pseudo-element).

#### 3b. Cascade Resolution

Sort matching rules by origin (author vs. user-agent), specificity (high to low), and source order (last declaration wins). Handles `!important`, `inherit`, `initial`, and `unset` values.

#### 3c. Property Application

Shorthand expansion (e.g., `margin: 10px 20px` → `margin-top`, `margin-right`, etc.), type coercion (strings to typed CSS values), and validation against property-specific value grammars.

#### 3d. CSS Variable Resolution

`var(--name)` function calls are resolved from the `custom_properties` map accumulated from the cascade. Features:
- Custom property registry built from `--*` declarations
- Cycle detection via a visited set (`HashSet<String>`) during resolution
- Fallback values: `var(--missing, fallback-value)`
- Inherited variable propagation through the DOM tree

#### 3e. `calc()` Evaluation

Recursive-descent expression evaluator (`crates/motarjim-css/src/calc.rs`) that parses and computes `calc()` expressions:
- **Operators:** `+`, `-`, `*`, `/` with standard precedence (PEMDAS)
- **Units:** mixed-unit arithmetic with automatic conversion (px, em, %, etc.)
- **Percentage resolution:** percentage values resolved relative to a parent context dimension
- **Parenthesized sub-expressions** evaluated depth-first
- Error handling for division by zero and type mismatches

#### 3f. Media Query Evaluation

Runtime filtering of `@media` at-rules (`crates/motarjim-css/src/media.rs`) based on configurable viewport and user preferences:
- **Width/height conditions:** `min-width`, `max-width`, `min-height`, `max-height`
- **User preference:** `prefers-color-scheme` (light/dark)
- **Boolean combinators:** `not`, `and`, `or` (comma-separated list is an implicit `or`)
- Viewport and color scheme are configured via `GlobalConfig.build.viewport`

#### 3g. Grid Layout Parsing

Structured representation of CSS Grid properties (`crates/motarjim-ast-html/src/grid.rs`):
- `grid-template-columns`, `grid-template-rows` → `GridTemplate` with track lists
- `grid-template-areas` → named area strings
- `grid-column`, `grid-row`, `grid-area` → `GridPlacement` with start/end lines
- Track types: fixed length, percentage, flexible (`fr`), `min-content`, `max-content`, `auto`, `minmax()`, `repeat()`
- Line types: auto, numeric index, named line, named line with span

#### 3h. Vendor Prefix Generation

Automatic `-webkit-`, `-moz-`, and `-ms-` prefix insertion for CSS properties that require vendor prefixes for cross-browser compatibility.

### Stage 4: Output Generation

**Input:** Styled HTML documents
**Output:** Static files on disk
**Crates:** `motarjim-output`

Multiple output formats are generated from a single build:
- **HTML** — Complete rendered pages with resolved styles
- **RSS/Atom feeds** — XML feed files with configurable title, description, filtering, and item count
- **JSON** — Structured page data for programmatic consumption
- **Plaintext** — Stripped content for search indexing or email

### Stage 5: Asset Copying

**Input:** Source asset directory
**Output:** Mirrored asset directory in output
**Crates:** `motarjim-assets`

Co-located assets (images, fonts, scripts) are copied from the content directory tree into the output directory, preserving directory structure. Only files not already processed by the pipeline are copied.

### Configuration Loading

**Crate:** `motarjim-config`

Loads `motarjim.toml` with the following top-level sections:

```toml
[site]          # title, base_url, author, language
[build]         # output_dir, input_dir, viewport (width, height, prefers_color_scheme)
[feeds]         # RSS and Atom feed configuration (enabled, title, description, count, filter)
[server]        # dev server port, host, live_reload
```

The `[build.viewport]` section controls media query evaluation:
- `width` — viewport width in pixels (default: 1024)
- `height` — viewport height in pixels (default: 768)
- `prefers_color_scheme` — `"light"` or `"dark"` (default: `"light"`)

## Design Decisions

### Why Rust?

- **Performance** — Blazingly fast builds; CSS engine handles thousands of selectors in microseconds
- **Correctness** — Strong type system, ownership model, exhaustive pattern matching on CSS value types
- **Reliability** — No runtime crashes; graceful error handling for malformed CSS and missing values

### Why Separate Crates?

Each crate is independently publishable on crates.io. This enables:
- **Reusability** — `motarjim-css` can be embedded in other Rust tooling that needs CSS cascade resolution
- **Parallel compilation** — Cargo compiles crates in parallel
- **Focused testing** — Each crate has its own test suite

### CSS Engine Design

The CSS engine is designed for **predictability** rather than completeness:
- Follows the W3C cascade specification for property resolution order
- Recursive-descent parsers are simple to debug and maintain
- Custom property resolution with cycle detection prevents infinite loops
- `calc()` evaluation supports the full arithmetic subset with proper unit handling
- Media queries are evaluated at build time against a configured viewport (not a real browser)

## Key Data Structures

### `ComputedStyle` (`motarjim-ast-html::style`)

```rust
pub struct ComputedStyle {
    // Basic styling
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub font_size: Option<String>,
    pub font_family: Option<String>,
    pub font_weight: Option<String>,
    pub text_align: Option<String>,

    // Box model
    pub width: Option<String>,
    pub height: Option<String>,
    pub margin: Option<String>,
    pub margin_top: Option<String>,
    pub margin_right: Option<String>,
    pub margin_bottom: Option<String>,
    pub margin_left: Option<String>,
    pub padding: Option<String>,
    pub padding_top: Option<String>,
    pub padding_right: Option<String>,
    pub padding_bottom: Option<String>,
    pub padding_left: Option<String>,

    // Border
    pub border: Option<String>,
    pub border_top: Option<String>,
    pub border_right: Option<String>,
    pub border_bottom: Option<String>,
    pub border_left: Option<String>,
    pub border_radius: Option<String>,

    // Layout
    pub display: Option<String>,
    pub position: Option<String>,
    pub flex_direction: Option<String>,
    pub flex_wrap: Option<String>,
    pub justify_content: Option<String>,
    pub align_items: Option<String>,
    pub gap: Option<String>,

    // Positioning offsets
    pub top: Option<String>,
    pub right: Option<String>,
    pub bottom: Option<String>,
    pub left: Option<String>,

    // Animation
    pub animation_name: Option<String>,
    pub animation_duration: Option<String>,
    pub animation_timing_function: Option<String>,

    // Grid
    pub grid_template_columns: Option<GridTemplate>,
    pub grid_template_rows: Option<GridTemplate>,
    pub grid_template_areas: Option<Vec<String>>,
    pub grid_column: Option<GridPlacement>,
    pub grid_row: Option<GridPlacement>,

    // CSS custom properties
    pub custom_properties: HashMap<String, String>,
}
```

## Future Plans

1. **Incremental builds** — Only re-process files whose source or dependencies changed
2. **CSS source maps** — Line-number mapping from output styles back to source CSS
3. **Sass/SCSS compilation** — Pre-processor support before cascade resolution
4. **More selectors** — `:has()`, `:where()`, `:is()`, `:not()` with full selector lists
5. **Benchmarking suite** — Criterion benchmarks for pipeline throughput
