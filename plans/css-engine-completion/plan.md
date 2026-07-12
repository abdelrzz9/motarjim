# CSS Engine Completion Engineering Plan

## Objective
Deliver a fully functional CSS engine that handles modern CSS features — variables, media queries, grid, positioning, `calc()`, animations — producing correct computed styles and structured data for code generation.

## Scope (6 tasks, ordered by recommended execution order)

| # | Task | Effort | Risk | Depends On |
|---|------|--------|------|------------|
| 1 | **CSS Variable Resolution** (`var()`) | 2-3 weeks | Medium | Nothing |
| 2 | **Positioning Offsets** (`top`/`right`/`bottom`/`left`) | 1 week | Low | Nothing |
| 3 | **Media Query Evaluation** (`@media`) | 2-3 weeks | Medium | Nothing |
| 4 | **CSS Grid Structured Parsing** | 2-3 weeks | Low | Nothing |
| 5 | **`calc()` Evaluation** | 2-3 weeks | Medium | Nothing |
| 6 | **Animations (`@keyframes` + animation properties)** | 3-4 weeks | Medium | Task 3 (responsive) |

## Excluded (defer to later milestones)
- `@supports` evaluation (low usage, simple stub)
- `@import` resolution (requires URL fetching + multi-file merging)
- Hex-alpha / named color refactoring (low impact; colors work as raw strings)
- `box-sizing`, `aspect-ratio`, `filter`, `text-shadow`, `outline` (new CSS properties, not engine features)
- Performance optimization (arena allocation, string interning)

---

## TASK 1: CSS Variable Resolution

### Context
- `var(--x)` is parsed by LightningCSS and stored in `Declaration.parsed: Option<CssValue::Variable("--x")>` but the `parsed` field is **never read** by the CSS engine.
- `properties.rs` receives the raw string `"var(--primary-color)"` and drops it via `_ => {}`.
- `ComputedStyle` has no `custom_properties` field for storing `--*` declarations.

### Sub-tasks

**1.1 Add `custom_properties` to `ComputedStyle`**
- File: `crates/motarjim-ast-html/src/style.rs`
- Add `custom_properties: HashMap<String, String>` to `ComputedStyle` struct
- Default to empty map

**1.2 Build custom property registry during cascade**
- File: `crates/motarjim-css/src/properties.rs`
- Add `apply_custom_property(decl: &Declaration, style: &mut ComputedStyle)`:
  - Detect declarations whose name starts with `--`
  - Store `name_without_prefix -> raw_value` in `style.custom_properties`
- Call from `apply_declarations()` or cascade loop

**1.3 Implement `var()` resolution in cascade**
- File: `crates/motarjim-css/src/properties.rs` — new function `resolve_var(value: &str, style: &ComputedStyle) -> Option<String>`
- Extract variable name and optional fallback from `var(--name)` / `var(--name, fallback)`
- Look up in `style.custom_properties`
- If found, substitute the value (handle nested `var()`)
- If not found, use fallback, or emit diagnostic for undefined variable
- Detect circular references → diagnostic

**1.4 Wire `parsed` field into the pipeline**
- Instead of regex-extracting `var()` from raw strings, use the already-parsed `Declaration.parsed`
- Modify `apply_property` to check `parsed` for `CssValue::Variable` before raw string matching

**1.5 Update tests** (`crates/motarjim-css/src/tests.rs`)
- `--primary: blue; color: var(--primary)` → `color = "blue"`
- `color: var(--undefined, red)` → `color = "red"`
- `color: var(--undefined)` → diagnostic
- Nested: `--a: var(--b); --b: 10px; width: var(--a)` → `width = "10px"`
- Circular: `--a: var(--b); --b: var(--a)` → diagnostic

### Files Changed
- `crates/motarjim-ast-html/src/style.rs`
- `crates/motarjim-css/src/properties.rs`
- `crates/motarjim-css/src/cascade.rs`

---

## TASK 2: Positioning Offsets

### Context
- `ComputedStyle` has `position: PositionType` but **no** `top`, `right`, `bottom`, `left` fields
- `properties.rs` silently drops these property names
- The `blog.css` example uses `position: sticky; top: 0;` — `top: 0` is ignored

### Sub-tasks

**2.1 Add offset fields to `ComputedStyle`**
- File: `crates/motarjim-ast-html/src/style.rs`
- Add:
  ```rust
  pub top: Option<String>,
  pub right: Option<String>,
  pub bottom: Option<String>,
  pub left: Option<String>,
  ```
- Add `inset` shorthand handler

**2.2 Add property handlers**
- File: `crates/motarjim-css/src/properties.rs`
- Handle `"top"`, `"right"`, `"bottom"`, `"left"`, `"inset"`
- Parse length values using existing `parse_length` infrastructure

**2.3 Wire through IR and generators**
- File: `crates/motarjim-ir/src/layout.rs`
- Pass offset hints through `LayoutIr` or `ComputedStyle` to generators
- Generators map `position: absolute; top: 10px; left: 20px` to platform APIs

**2.4 Tests**
- `position: absolute; top: 10px; left: 20px` → computed offsets stored
- `position: static; top: 10px` → value exists, no semantic effect
- `inset: 10px` → all four offsets = `"10px"`
- `blog.css` topbar `position: sticky; top: 0;` now works

### Files Changed
- `crates/motarjim-ast-html/src/style.rs`
- `crates/motarjim-css/src/properties.rs`
- `crates/motarjim-ir/src/layout.rs`

---

## TASK 3: Media Query Evaluation

### Context
- `MediaRule` has full structured AST (MinWidth, MaxWidth, Screen, And, Or, Not)
- `StyleResolver` always recurses into ALL media rules regardless of conditions
- `ResponsiveInferrer` in `motarjim-ir/src/responsive.rs` is a stub
- No viewport info exists in config or session

### Sub-tasks

**3.1 Add viewport configuration**
- File: `crates/motarjim-config/src/lib.rs`
- Add `viewport_width: u32` (default 1920) and `viewport_height: u32` (default 1080) to `GlobalConfig`
- Add `prefers_color_scheme: String` (default `"light"`) for `prefers-color-scheme` media feature
- Add CLI flags `--viewport-width`, `--viewport-height`

**3.2 Implement media condition evaluator**
- New file: `crates/motarjim-css/src/media.rs`
- `fn evaluate_media_query(query: &MediaQuery, viewport: (u32, u32), color_scheme: &str) -> bool`
- Implement:
  - `MinWidth(px)` → `viewport.0 >= px`
  - `MaxWidth(px)` → `viewport.0 <= px`
  - `MinHeight(px)` → `viewport.1 >= px`
  - `MaxHeight(px)` → `viewport.1 <= px`
  - `Screen` → `true`
  - `Print` → `false`
  - `All` → `true`
  - `Not(cond)` → `!evaluate(cond)`
  - `And(conds)` → all true
  - `Or(conds)` → any true
  - `PrefersColorScheme(dark)` → `color_scheme == "dark"`
- Export from `motarjim-css/src/lib.rs`

**3.3 Wire evaluation into `StyleResolver`**
- File: `crates/motarjim-css/src/resolver.rs`
- Change `CssRule::Media` handler: call `evaluate_media_query()`, only recurse if `true`
- Thread `viewport: (u32, u32)` through `resolve_with_context()` signature

**3.4 Wire viewport from Session to Resolver**
- File: `crates/motarjim-core/src/lib.rs`
- Extract viewport + color scheme from `session.config()` and pass to resolver

**3.5 Implement `ResponsiveInferrer`**
- File: `crates/motarjim-ir/src/responsive.rs`
- Remove stub, add real logic:
  - Detect nodes with different styles at different breakpoints
  - Map common breakpoint widths to `ResponsiveVariant`: 320-480 → Mobile, 768-1024 → Tablet, 1280+ → Desktop
  - Populate `breakpoint` and `style_override`
- Wire into IR builder

**3.6 Tests**
- `@media (max-width: 768px) { .foo { color: red } }` with viewport 375×667 → applies
- Same query with viewport 1920×1080 → skipped
- `@media screen and (min-width: 1024px)` → evaluated correctly
- `@media not (max-width: 600px)` → negation correct
- `@media (prefers-color-scheme: dark)` with config `light` → skipped
- ResponsiveInferrer produces correct variants

### Files Changed
- `crates/motarjim-config/src/lib.rs`
- `crates/motarjim-css/src/lib.rs` (new `media.rs`)
- `crates/motarjim-css/src/resolver.rs`
- `crates/motarjim-core/src/lib.rs`
- `crates/motarjim-ir/src/responsive.rs`
- `crates/motarjim-ir/src/builder.rs`

---

## TASK 4: CSS Grid Structured Parsing

### Context
- Grid properties stored as raw strings: `grid_template_columns: Option<String>`
- `CssUnit::Fr` exists in `motarjim-ast-css` but is unused in `motarjim-css/src/value.rs`
- No `grid-template-areas`, `grid-auto-flow`, `grid-auto-rows/columns` fields
- `gap`/`row_gap`/`column_gap` are `Option<String>` (raw strings)

### Sub-tasks

**4.1 Create structured grid types**
- New file: `crates/motarjim-ast-html/src/grid.rs`
- Define:
  ```rust
  pub struct GridTemplate {
      pub tracks: Vec<GridTrack>,
  }

  pub enum GridTrack {
      Fixed(f64),
      Fr(f64),
      MinMax(Box<GridTrack>, Box<GridTrack>),
      Auto,
      MinContent,
      MaxContent,
      FitContent(f64),
      Repeat(u32, Vec<GridTrack>),
  }

  pub struct GridPlacement {
      pub line: GridLine,
      pub span: Option<u32>,
  }

  pub enum GridLine {
      Auto,
      Named(String),
      Number(u32),
  }
  ```

**4.2 Add structured grid fields to `ComputedStyle`**
- File: `crates/motarjim-ast-html/src/style.rs`
- Replace raw string grid fields:
  ```rust
  pub grid_template_columns: Option<GridTemplate>,
  pub grid_template_rows: Option<GridTemplate>,
  pub grid_column_start: Option<GridPlacement>,
  pub grid_column_end: Option<GridPlacement>,
  pub grid_row_start: Option<GridPlacement>,
  pub grid_row_end: Option<GridPlacement>,
  pub grid_template_areas: Option<Vec<String>>,
  pub grid_auto_flow: Option<String>,
  pub grid_auto_columns: Option<GridTemplate>,
  pub grid_auto_rows: Option<GridTemplate>,
  ```

**4.3 Implement grid value parsers**
- File: `crates/motarjim-css/src/properties.rs`
- Parse `grid-template-columns` / `grid-template-rows`:
  - `1fr 1fr 1fr` → `[Fr(1), Fr(1), Fr(1)]`
  - `repeat(3, 1fr)` → `Repeat(3, [Fr(1)])`
  - `minmax(100px, 1fr)` → `MinMax(Fixed(100), Fr(1))`
  - `auto` → `Auto`
  - `fit-content(200px)` → `FitContent(200)`
- Parse `grid-column` / `grid-row`:
  - `1 / 3` → `GridPlacement { line: Number(1) }` + `GridPlacement { line: Number(3) }`
  - `span 2` → `GridPlacement { line: Auto, span: Some(2) }`
  - `auto` → `GridPlacement { line: Auto, span: None }`
- Parse `grid-template-areas`: `"a b" "c d"` → `["a", "b", "c", "d"]`

**4.4 Update `apply_property`**
- File: `crates/motarjim-css/src/properties.rs`
- Replace raw string storage with structured parsers for grid properties
- Keep raw string fallback for unknown syntax (store as raw + emit diagnostic)

**4.5 Tests**
- `grid-template-columns: 1fr 1fr 1fr` → structured correctly
- `grid-template-columns: repeat(3, 1fr)` → repeat expanded
- `grid-template-columns: 200px auto 1fr` → mixed tracks
- `grid-column: 1 / 3` → correct placement pair
- `grid-column: span 2` → span placement

### Files Changed
- `crates/motarjim-ast-html/src/style.rs`
- `crates/motarjim-ast-html/src/grid.rs` (new)
- `crates/motarjim-ast-html/src/lib.rs` (export grid module)
- `crates/motarjim-css/src/properties.rs`

---

## TASK 5: `calc()` Evaluation

### Context
- `calc()` is parsed by LightningCSS and stored as `CssValue::Function("calc", ...)` in `parsed` — but `parsed` is never used
- Raw string `"calc(100% - 40px)"` reaches `properties.rs`'s `parse_length` which fails and returns `None`
- For the compiler use case, we need partial evaluation: resolve same-unit arithmetic, emit diagnostic for incompatible/missing-context

### Sub-tasks

**5.1 Create calc evaluation engine**
- New file: `crates/motarjim-css/src/calc.rs`
- Define:
  ```rust
  pub enum CalcValue {
      Length(f64, CssUnit),
      Percentage(f64),
      Number(f64),
      Raw(String), // fallback for unparseable
  }

  pub struct CalcContext {
      pub viewport_width: f64,
      pub viewport_height: f64,
      pub parent_width: Option<f64>,
      pub font_size: Option<f64>,
  }
  ```
- `fn evaluate_calc(expr: &str, context: &CalcContext) -> CalcValue`
  - Recursive descent parser for `+`, `-`, `*`, `/`, parens
  - Same-unit arithmetic: `calc(100% - 20px)` with known parent → resolve
  - `calc(50% + 10%)` → `Percentage(60.0)`
  - `calc(10px * 2)` → `Length(20.0, Px)`
  - `calc(100vw - 40px)` → use viewport from CalcContext
  - Incompatible units → diagnostic + `Raw` fallback

**5.2 Integrate into properties pipeline**
- File: `crates/motarjim-css/src/properties.rs`
- In `parse_length` and property handlers, before failing on unknown values, check for `calc(`
- If found, call `evaluate_calc()` and use the result
- For unresolvable calc (e.g., `calc(100% - 20px)` without parent context), store raw string + diagnostic

**5.3 Wire through to generators as hints**
- When calc can't be fully resolved, generators emit platform-specific calc (e.g., Flutter `MediaQuery.of(context).size.width * 1.0 - 40`)

**5.4 Tests**
- `calc(10px + 20px)` → `Length(30, Px)`
- `calc(100% - 40px)` with parent=500 → `Length(460, Px)`
- `calc(50% + 10%)` → `Percentage(60)`
- `calc(10px * 2)` → `Length(20, Px)`
- `calc(10px + 10em)` → diagnostic (incompatible)
- `calc(100vw - 20px)` with viewport=1920 → `Length(1900, Px)`

### Files Changed
- `crates/motarjim-css/src/calc.rs` (new)
- `crates/motarjim-css/src/lib.rs` (export calc)
- `crates/motarjim-css/src/properties.rs`
- `crates/motarjim-css/src/value.rs` (export `CssUnit`)

---

## TASK 6: Animations (`@keyframes` + Animation Properties)

### Context
- `@keyframes` is fully parsed into `KeyframesRule { name, keyframes: Vec<Keyframe> }` but **skipped** by the CSS engine
- `ComputedStyle` has only `transform: Option<String>` and `transition: Option<String>` — no animation fields
- No `animation-name`, `animation-duration`, `animation-*` property handlers exist
- Generators have no animation output

### Sub-tasks

**6.1 Add animation fields to `ComputedStyle`**
- File: `crates/motarjim-ast-html/src/style.rs`
- Add:
  ```rust
  pub animation_name: Option<String>,
  pub animation_duration: Option<String>,
  pub animation_timing_function: Option<String>,
  pub animation_delay: Option<String>,
  pub animation_iteration_count: Option<String>,
  pub animation_direction: Option<String>,
  pub animation_fill_mode: Option<String>,
  pub animation_play_state: Option<String>,
  ```
- Add `animation` shorthand handler that expands into individual sub-properties

**6.2 Pass `@keyframes` through the cascade**
- File: `crates/motarjim-css/src/resolver.rs`
- Change `CssRule::Keyframes(_)` from `// skip` to collecting into a `Vec<KeyframesRule>`
- File: `crates/motarjim-css/src/lib.rs`
- Add `KeyframesCollection: HashMap<String, KeyframesRule>` type
- Thread through resolution output so generators can access keyframe data

**6.3 Add animation property handlers**
- File: `crates/motarjim-css/src/properties.rs`
- Add handlers for `animation-name`, `animation-duration`, `animation-timing-function`, `animation-delay`, `animation-iteration-count`, `animation-direction`, `animation-fill-mode`, `animation-play-state`
- Add `animation` shorthand: space-separated values → expanded to individual fields

**6.4 Wire through IR to generators**
- File: `crates/motarjim-ir/src/builder.rs`
- When a node has animation properties, emit `SemanticIr` hints with animation data
- Generators map to platform APIs:
  - Flutter: `AnimationController` + `AnimatedBuilder`
  - Compose: `animate*AsState`
  - SwiftUI: `withAnimation` + `.animation()` modifier

**6.5 Tests**
- `@keyframes slide { from { opacity: 0 } to { opacity: 1 } }` → stored in collection
- `animation-name: slide` → `ComputedStyle.animation_name = "slide"`
- `animation: slide 1s ease-in-out` → shorthand parsed correctly
- Full animation shorthand with all values
- Generator produces animation code structure

### Files Changed
- `crates/motarjim-ast-html/src/style.rs`
- `crates/motarjim-css/src/resolver.rs`
- `crates/motarjim-css/src/lib.rs`
- `crates/motarjim-css/src/properties.rs`
- `crates/motarjim-ir/src/ir.rs` (if new variants needed)
- `crates/motarjim-ir/src/builder.rs`
- `crates/motarjim-gen-flutter/src/generator.rs`
- `crates/motarjim-gen-compose/src/generator.rs`
- `crates/motarjim-gen-swiftui/src/generator.rs`

---

## Execution Order

```
Week 1-2:   TASK 2 (Positioning) — easy win, no deps
            + TASK 1 (Variables) — start (most impactful)
Week 3-4:   TASK 1 (Variables) — finish
            + TASK 4 (Grid parsing) — start
Week 5-6:   TASK 4 (Grid) — finish
            + TASK 3 (Media queries) — start
Week 7-8:   TASK 3 (Media queries) — finish
            + TASK 5 (calc) — start
Week 9-10:  TASK 5 (calc) — finish
            + TASK 6 (Animations) — start
Week 11-12: TASK 6 (Animations) — finish
```

## Files Changed Summary

| File | Task | Change |
|------|------|--------|
| `motarjim-ast-html/src/style.rs` | 1, 2, 4, 6 | Add `custom_properties`, `top/right/bottom/left`, structured grid, animation fields |
| `motarjim-ast-html/src/grid.rs` | 4 | New: `GridTemplate`, `GridTrack`, `GridPlacement` types |
| `motarjim-ast-html/src/lib.rs` | 4 | Export `grid` module |
| `motarjim-css/src/properties.rs` | 1, 2, 4, 5, 6 | `resolve_var()`, position handlers, grid parsers, calc integration, animation handlers |
| `motarjim-css/src/cascade.rs` | 1 | Thread `custom_properties` accumulation |
| `motarjim-css/src/resolver.rs` | 3, 6 | Media evaluation, keyframe collection |
| `motarjim-css/src/media.rs` | 3 | New: `evaluate_media_query()` |
| `motarjim-css/src/calc.rs` | 5 | New: `evaluate_calc()` + `CalcValue`/`CalcContext` |
| `motarjim-css/src/value.rs` | 5 | Export `CssUnit` |
| `motarjim-css/src/lib.rs` | 3, 5, 6 | Export new modules; `KeyframesCollection` type |
| `motarjim-config/src/lib.rs` | 3 | Add `viewport_width`, `viewport_height`, `prefers_color_scheme` |
| `motarjim-core/src/lib.rs` | 3 | Pass viewport + color scheme to resolver |
| `motarjim-ir/src/responsive.rs` | 3 | Real `ResponsiveInferrer` implementation |
| `motarjim-ir/src/builder.rs` | 3, 6 | Wire responsive variants, animation data |
| `motarjim-ir/src/ir.rs` | 6 | Animation-related IR variants (if needed) |
| `motarjim-ir/src/layout.rs` | 2 | Position offset hints |
| All 3 generators | 2, 6 | Positioning + animation scaffolding |

## Test Plan

After each task:
```bash
cargo test -p <affected-crate>
cargo build --workspace
```

After all tasks:
```bash
cargo test --workspace
# Run examples with various CSS features
cargo run -- compile examples/blog.html --format dart
cargo run -- compile examples/dashboard.html --format kotlin
cargo run -- compile examples/ecommerce.html --format swift
```

## Acceptance Criteria

- [ ] `var(--primary, blue)` resolves correctly, including fallbacks and nested vars
- [ ] `position: absolute; top: 10px; left: 20px` → offsets stored and available to generators
- [ ] `@media (max-width: 768px)` only applies rules when viewport matches
- [ ] `@media` rules produce `ResponsiveVariant` entries in IR
- [ ] `grid-template-columns: 1fr 1fr 1fr` → structured `GridTemplate` with `Fr` tracks
- [ ] `calc(100% - 40px)` evaluates partially (resolve same-unit, diagnostic for context-dependent)
- [ ] `@keyframes slide { ... }` collected and accessible by generators
- [ ] `animation-name: slide` stored in `ComputedStyle`
- [ ] All existing tests still pass
- [ ] No regressions in Core Stabilization features (combinators, diagnostics, generators)
