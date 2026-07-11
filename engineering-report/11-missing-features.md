# Missing Features for 1.0

## Critical (Blocking 1.0 Release)

| # | Feature | Current Status | Effort | Risk |
|--:|---------|:--------------:|:------:|:----:|
| 1 | **Wire html5ever into main pipeline** | Custom parser is default; html5ever is separate, incompatible | 2 weeks | Low |
| 2 | **CSS combinator traversal** | Combinators (descendant/child/sibling) ignored by engine | 1-2 weeks | Low |
| 3 | **CSS variable resolution** | `var(--name)` passes through unresolvable | 2-3 weeks | Medium |
| 4 | **Media query evaluation** | `@media` rules always unconditionally included | 2-3 weeks | Medium |
| 5 | **Responsive IR** | `ResponsiveInferrer` returns empty vec | 1-2 weeks | Low |
| 6 | **CSS property mapping in generators** | 2-6/50+ CSS properties mapped to platform code | 4-6 weeks | Low |
| 7 | **Fix generator bugs** | 3 SwiftUI bugs, 1 Flutter bug, 2 Compose issues | 1 week | Low |
| 8 | **Incremental compilation** | `IncrementalEngine` exists but not wired | 1-2 weeks | Low |
| 9 | **Artifact caching** | `ArtifactCache` exists but not wired | 1 week | Low |

## High Priority

| # | Feature | Current Status | Effort | Risk |
|--:|---------|:--------------:|:------:|:----:|
| 10 | **CLI watch mode** | Stub ("not yet implemented") | 2-3 weeks | Low |
| 11 | **Multi-file/directory compilation** | Single file input only | 1 week | Low |
| 12 | **Source maps** | Not generated | 3-4 weeks | Medium |
| 13 | **Snapshot/golden tests** | No expected output for any platform | 2-3 weeks | Low |
| 14 | **Optimization levels** | No `-O0`/`-O1`/`-O2` | 1-2 weeks | Low |
| 15 | **DAG scheduler by default** | Feature-gated behind `dag` flag | 1 week | Low |
| 16 | **Arena allocation** | Individual heap allocs for all nodes | 4-6 weeks | Medium |
| 17 | **LSP real handlers** | Completion/hover/definition/semantic-tokens are stubs | 4-6 weeks | Medium |
| 18 | **CSS top/right/bottom/left** | Not in ComputedStyle | 1 week | Low |
| 19 | **Wire JS event bindings** | `find_dom_event_bindings()` output not consumed | 2-3 weeks | Low |
| 20 | **CSS Grid structured parsing** | Grid properties stored as raw strings | 2-3 weeks | Low |

## Medium Priority

| # | Feature | Current Status | Effort |
|--:|---------|:--------------:|:------:|
| 21 | `calc()` evaluation | Parsed, not evaluated | 2-3 weeks |
| 22 | CSS `@keyframes` → platform animations | Parsed, extraction needed | 3-4 weeks |
| 23 | CLI config file discovery (walk up dirs) | CWD only | 1 week |
| 24 | WASM TypeScript types + npm package | No `.d.ts`, no npm publish | 1-2 weeks |
| 25 | `--config` CLI flag | Not implemented | 1 day |
| 26 | Environment variable overrides | Not implemented | 1 week |
| 27 | String interning (SymbolId) | SmolStr, no global interning | 2-3 weeks |
| 28 | Lazy computed style | All properties computed for all nodes | 3-4 weeks |
| 29 | JSON diagnostic output | `json` feature exists, emitter only supports terminal | 1 week |
| 30 | Accessibility in generated code | `AccessibilityInfo` extracted but not used by generators | 2-3 weeks |
| 31 | HTML character reference decoding | `&amp;` etc. not decoded in custom parser | 1-2 weeks |
| 32 | `aria-labelledby` resolution | Stored as string, not resolved | 1-2 weeks |
| 33 | Compose `Table` widget | Stub (comment only) | 1-2 weeks |
| 34 | SwiftUI `justifyContent` mapping | Missing (Flutter/Compose have it) | 1 day |
| 35 | Image `src` from IR → generators | Hardcoded URL in all three | 1 week |
| 36 | Icon name from IR → generators | Hardcoded star icon in all three | 1 week |
| 37 | Form field labels/placeholders from IR | Hardcoded | 1-2 weeks |
| 38 | CSS `gap` → platform spacing | Parsed, not used by generators | 1 week |
| 39 | CSS `flex-grow`/`flex-shrink` → platform | Parsed, not used by generators | 1-2 weeks |
| 40 | CSS `opacity` → platform | Parsed, not used by generators | 1 day |
| 41 | CSS `overflow` → platform scroll/clip | Parsed, not used by generators | 1 week |
| 42 | CSS `border-radius` → platform | Parsed, not used by generators | 1-2 weeks |
| 43 | CSS `box-shadow` → platform | Stored as raw string, not parsed → platform | 2-3 weeks |
| 44 | CSS `font-family` → platform | Parsed, not used by generators | 1 week |
| 45 | CSS `text-align` → platform | Parsed, not used by generators | 1 day |

## Low Priority (Post-1.0)

| # | Feature | Notes |
|--:|---------|-------|
| 46 | CSS `@container` queries | Requires container query support |
| 47 | CSS `@scope` support | New CSS feature |
| 48 | CSS `:has()` selector evaluation | Complex, requires full selector rewrite |
| 49 | CSS nesting engine evaluation | Already parsed via LightningCSS |
| 50 | Template literals → string concat in JS | Only transform, but not wired |
| 51 | SVG path compilation → platform vector graphics | Complex |
| 52 | Canvas 2D API compilation | Very complex |
| 53 | Web Components (custom elements, slots) | Complex |
| 54 | CSS `filter` and `backdrop-filter` | Requires platform mapping |
| 55 | CSS `clip-path` → platform | Complex shape mapping |
| 56 | CSS `scroll-snap` support | Medium complexity |
| 57 | CSS `aspect-ratio` | ComputedStyle missing, already in LayoutConstraints |
| 58 | CSS `contain` / `content-visibility` | Performance optimization |
| 59 | CSS `color-mix()` | New CSS feature |
| 60 | CSS `@property` registration | Requires Houdini-like model |

## Features That Need RFC First

| Feature | Reason |
|---------|--------|
| Plugin API v2 | Current `Generator` trait is functional but may need changes for 1.0 API stability |
| IR HIR/MIR/LIR split | Significant architectural change, needs RFC-0001 |
| Incremental compilation API | Query system design needs formalization via RFC-0007 |
| Layout engine | Flexbox/grid lowering strategy needs design via RFC-0006 |
| Arena allocation | Needs design for lifetime management and API ergonomics |
