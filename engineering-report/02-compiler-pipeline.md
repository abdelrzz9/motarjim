# Compiler Pipeline Analysis

## Overview

Motarjim follows a classic 6-stage compiler pipeline: Parse → Style → IR → Optimize → Generate. Each stage is a separate crate or set of crates.

```
Source HTML + CSS
     │
     ▼
┌─────────────────────────────────────────────────┐
│               Stage 1: Parsing                   │
│  ┌─────────────┐  ┌─────────────┐               │
│  │ HtmlLexer   │  │ CssLexer    │               │
│  │ (custom)    │  │ (custom)    │               │
│  └──────┬──────┘  └──────┬──────┘               │
│         │                │                       │
│         ▼                ▼                       │
│  ┌─────────────┐  ┌──────────────┐              │
│  │ HtmlParser  │  │ CssParser    │              │
│  │ (custom OR  │  │ (LightningCSS│              │
│  │  html5ever) │  │  wrapper)    │              │
│  └─────────────┘  └──────────────┘              │
└─────────────────────────────────────────────────┘
         │                │
         ▼                ▼
┌─────────────────────────────────────────────────┐
│               Stage 2: Style                     │
│  ┌──────────────┐  ┌────────────┐               │
│  │  Selectors   │  │  Cascade   │               │
│  │  (matching)  │──▶│  (resolve) │               │
│  └──────────────┘  └────────────┘               │
│                           │                      │
│                           ▼                      │
│                    ┌──────────────┐              │
│                    │  Computed    │              │
│                    │  Style       │              │
│                    └──────────────┘              │
└─────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────┐
│               Stage 3: IR                        │
│  ┌──────────────┐  ┌──────────┐  ┌───────────┐ │
│  │  Semantic    │  │  Layout  │  │Responsive │ │
│  │  Inference   │  │Inference │  │ (STUB)    │ │
│  └──────────────┘  └──────────┘  └───────────┘ │
│  ┌──────────────┐                               │
│  │Accessibility │                               │
│  │  Inference   │                               │
│  └──────────────┘                               │
│         │               │                        │
│         └───────┬───────┘                        │
│                 ▼                                │
│          ┌──────────────┐                        │
│          │  IrBuilder   │                        │
│          │  (IrTree)    │                        │
│          └──────────────┘                        │
└─────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────┐
│               Stage 4: Optimization              │
│  PassManager (6 passes in order)                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │  Remove  │ │ Collapse │ │  Merge   │        │
│  │  Empty   │ │Whitespace│ │Adjacent  │        │
│  │  Nodes   │ │          │ │  Text    │        │
│  └──────────┘ └──────────┘ └──────────┘        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ Remove   │ │  Flatten │ │  Inline  │        │
│  │ Unused   │ │ Nested   │ │Constant  │        │
│  │ Styles   │ │Containers│ │ Values   │        │
│  └──────────┘ └──────────┘ └──────────┘        │
└─────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────┐
│               Stage 5: Generation                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │  Flutter │ │  Compose │ │  SwiftUI │        │
│  │  (Dart)  │ │ (Kotlin) │ │  (Swift) │        │
│  └──────────┘ └──────────┘ └──────────┘        │
└─────────────────────────────────────────────────┘
```

---

## Stage 1: Parsing

### Input: Raw HTML/CSS source text
### Output: Typed ASTs (Document + CssStylesheet)

### HTML Parsing

| Aspect | Current State |
|--------|--------------|
| **Implementation** | Two parallel paths: custom parser (648 LOC) or html5ever wrapper (2,118 LOC) |
| **Custom Parser** | Recursive-descent over tokens from `motarjim-lexer` |
| **html5ever Parser** | Full HTML5 spec via Servo's html5ever 0.39 |
| **Error Recovery** | Custom: basic (collects diagnostics, continues). html5ever: full spec recovery |
| **Issues** | Custom lexer does NOT produce attribute tokens (forces parser to re-scan raw tag text via string operations) |

**Who:** `motarjim-parser` + `motarjim-lexer` (custom path) OR `motarjim-html` (html5ever path)
**Maturity:** 5/10 (custom), 8/10 (html5ever)

### CSS Parsing

| Aspect | Current State |
|--------|--------------|
| **Implementation** | LightningCSS wrapper (`motarjim-parser/src/css/`, 2,026 LOC) |
| **Strategy** | Parse with LightningCSS, then convert LightningCSS AST → Motarjim AST (converter.rs, 1,589 LOC) |
| **Selector Handling** | "Serialize then re-parse" — LightningCSS serializes selectors to string, then Motarjim re-parses them |
| **Error Recovery** | Full (delegated to LightningCSS) |
| **Validation** | Post-conversion validation pass exists but has `#[allow(dead_code)]` — not called |

**Who:** `motarjim-parser` → `motarjim-ast-css`
**Maturity:** 8/10

### JavaScript Parsing

| Aspect | Current State |
|--------|--------------|
| **Implementation** | Full custom ECMAScript frontend (6,972 LOC) |
| **Lexer** | 120+ token kinds, character-level tokenizer |
| **Parser** | Pratt parser with 11 precedence levels, full expression/statement parsing |
| **Semantic Analysis** | Scope tracking, const reassignment detection, function captures |
| **DOM Events** | `find_dom_event_bindings()` extracts event handlers |
| **Issues** | Only 2 unit tests despite 6,972 LOC; output not wired into IR/generators |

**Who:** `motarjim-js`
**Maturity:** 7/10 (crate quality), 2/10 (integration into pipeline)

---

## Stage 2: Style Resolution

### Input: Document + CssStylesheet
### Output: HashMap<NodeId, ComputedStyle>

### Sub-stage 2a: Selector Matching

| Aspect | Current State |
|--------|--------------|
| **Implementation** | `motarjim-css/src/matching.rs` + `resolver.rs` |
| **Combinators** | ❌ NOT implemented. Only simple selectors are checked against the element, ignoring descendant/child/sibling combinators |
| **Pseudo-classes** | ❌ All return `true` (always matches). No state awareness |
| **Attribute selectors** | ✅ Full support (=, ~=, \|=, ^=, $=, *=) |
| **Parallelism** | ✅ Via `rayon::par_iter()` in `StyleResolver::resolve_parallel()` |

### Sub-stage 2b: Cascade Resolution

| Aspect | Current State |
|--------|--------------|
| **Implementation** | `motarjim-css/src/cascade.rs` (158 LOC) |
| **Specificity** | ✅ Correct: (id, class, type) tuple sorting |
| **!important** | ✅ Important declarations override normal |
| **Source Order** | ✅ Later rules override earlier at equal specificity |
| **Inheritance** | ✅ Child starts as clone of parent, then resolved values override |
| **@media/@supports** | ❌ Never evaluated — all nested rules unconditionally included |

### Sub-stage 2c: Computed Style

| Aspect | Current State |
|--------|--------------|
| **Output** | `ComputedStyle` struct with 50+ fields |
| **Typed Fields** | Strong enums for display, position, flex, overflow, text-align, font-weight |
| **String Fields** | Many fields stored as `Option<String>` (width, height, font-size, flex-basis, gap, grid properties, box-shadow, transform) |
| **Missing** | `top`, `right`, `bottom`, `left` positioning offsets |

**Who:** `motarjim-css` + `motarjim-selectors`
**Maturity:** 6/10

---

## Stage 3: IR Construction

### Input: Document + HashMap<NodeId, ComputedStyle>
### Output: IrTree

### Four inference passes (per-node, independent):

| Pass | Implementation | Status |
|------|---------------|--------|
| **SemanticAnalyzer** | HTML tag + attributes + ARIA role → SemanticIr (41 variants) | ✅ 165 LOC, well-tested |
| **LayoutInferrer** | CSS computed style → LayoutIr (17 variants) | ✅ 65 LOC, functional but ZStack/LazyList never emitted |
| **ResponsiveInferrer** | Media queries → ResponsiveVariant | ❌ **Complete stub** — always returns empty vec |
| **AccessibilityInferrer** | ARIA attributes → metadata | ✅ 113 LOC, comprehensive |

### IrTree Structure

```rust
pub struct IrTree {
    pub nodes: Vec<IrNode>,
    pub root_id: NodeId,
    pub target_hints: Vec<TargetHint>,
}

pub struct IrNode {
    pub id: NodeId,
    pub semantic: SemanticIr,
    pub layout: LayoutIr,
    pub target: TargetIr,
    pub computed_style: ComputedStyle,
    pub children: SmallVec<[NodeId; 4]>,
    pub parent: Option<NodeId>,
    pub text: Option<String>,
}
```

### Issues

1. **Responsive inferrer is a complete no-op** — No media query breakpoints extracted
2. **TargetIr::Flutter/Compose/SwiftUI never constructed** — Only `TargetIr::Generic` is emitted
3. **`_diagnostics` parameter ignored** — `IrBuilder::build()` accepts `DiagnosticBag` but never writes to it
4. **`aria-labelledby` stored as string but not resolved** — No cross-reference to actual element content
5. **`LayoutIr::ZStack` and `LayoutIr::LazyList` never emitted** — Dead variants

**Who:** `motarjim-ir` + `motarjim-ast-ir`
**Maturity:** 6/10

---

## Stage 4: Optimization

### Input: IrTree | Output: Optimized IrTree

### PassManager

- `Pass` trait with `name()`, `description()`, `prerequisites()`, `invalidated_by()`, `estimated_cost()`, `run()`
- `PassManager::run_all()` runs all passes in registration order with a `PassContext`

### Six optimization passes (in order):

| Pass | Cost | Description |
|------|------|-------------|
| RemoveEmptyNodes | O(n) | Removes empty/whitespace-only text, empty containers, display:none nodes |
| CollapseWhitespace | O(n) | Collapses runs of whitespace into single spaces, trims ends |
| MergeAdjacentText | O(n) | Merges adjacent sibling text nodes into one |
| RemoveUnusedStyles | O(n) | Clears flex/grid/text style props on inappropriate node types |
| FlattenNestedContainers | O(n) | Removes single-child containers promoting grandchildren |
| InlineConstantValues | O(n) | Removes default-styled single-child containers |

### Issues

1. **Pass dependency system unused** — `prerequisites()` and `invalidated_by()` are declared but never checked by PassManager
2. **No parallel pass execution** — Passes run sequentially despite thread-safe `PassStatistics`
3. **`memory_freed` always 0** — `PassStatistics` field never populated
4. **No optimization levels** — No O0/O1/O2, no pass selection/filtering
5. **No dead code elimination** — Beyond empty nodes, no unused style or layout optimization

**Who:** `motarjim-optimizer`
**Maturity:** 7/10 (implementation), 4/10 (completeness)

---

## Stage 5: Code Generation

### Input: Optimized IrTree | Output: Platform source code

### Three generators, same architecture:

| Generator | LOC | Widgets | CSS Mapped | Tests | Bugs |
|-----------|-----|---------|------------|-------|------|
| Flutter | 1,033 | 34/41 semantic variants | padding, margin, color, justify-content, align-items | 14 | emit_table_cell writes TableRow |
| Compose | 904 | 34/41 | padding, margin, color, width, height, justify-content, align-items | 12 | Lists use fake data; coil dependency |
| SwiftUI | 783 | 34/41 | padding, color | 11 | hstack alignment, dialog modifier, navbar chain broken |

### Common Issues Across All Generators

1. **Most CSS properties not mapped** — No background, border, border-radius, box-shadow, font-family, font-size (beyond headings), text-align, gap, flex-grow, opacity, overflow, position, z-index
2. **Hardcoded data** — Image URLs hardcoded as `"https://example.com/image.png"`, icons hardcoded as `star`/`star.fill`, chip labels hardcoded
3. **No data from IR for form fields** — No placeholder, label, validation wired
4. **`#[allow(clippy::unused_self)]`** — Many methods could be free functions
5. **No accessibility attributes in generated code**

**Who:** `motarjim-gen-flutter`, `motarjim-gen-compose`, `motarjim-gen-swiftui`
**Maturity:** 4-5/10
