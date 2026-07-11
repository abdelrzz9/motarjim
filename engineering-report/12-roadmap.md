# 1.0 Roadmap

## Milestone 1: Core Stabilization
**Timeline: 2-3 months | Difficulty: Medium | Priority: Critical**

| Task | Effort | Dependencies | Risk |
|------|:------:|:------------:|:----:|
| Wire html5ever into main pipeline | 2 weeks | None | Low |
| Fix CSS combinator traversal | 1-2 weeks | None | Low |
| Wire IncrementalEngine into Compiler | 1-2 weeks | M1: html5ever | Low |
| Wire ArtifactCache into Compiler | 1 week | M1: html5ever | Low |
| Fix generator bugs (6 known bugs) | 1 week | None | Low |
| Enable DAG scheduler by default | 1 week | None | Low |
| Fuzz targets for optimizer, IR, generators | 1 week | None | Low |
| Fix `VirtualFileSystem.write()` mutation bug | 1 day | None | Low |

**Deliverable:** Core pipeline with correct HTML parsing, CSS matching, incremental rebuilds, artifact caching, parallel execution by default.

**Acceptance Criteria:**
- [ ] html5ever is the default HTML parser
- [ ] CSS `div .button` matches descendants correctly
- [ ] `motarjim check` shows 2× faster for second invocation
- [ ] All 6 generator bugs fixed
- [ ] Pipeline parallelizes via DAG by default
- [ ] 100% fuzz coverage for all pipeline stages

---

## Milestone 2: CSS Engine Completion
**Timeline: 2-3 months | Difficulty: Hard | Priority: Critical**

| Task | Effort | Dependencies | Risk |
|------|:------:|:------------:|:----:|
| CSS variable resolution (`var()`) | 2-3 weeks | None | Medium |
| Media query evaluation | 2-3 weeks | None | Medium |
| CSS Grid structured parsing | 2-3 weeks | None | Low |
| Positioning offsets (top/right/bottom/left) | 1 week | None | Low |
| `calc()` evaluation | 2-3 weeks | None | Medium |
| CSS `@keyframes` → platform hints | 3-4 weeks | None | Medium |
| CSS `@supports` evaluation | 1 week | None | Low |
| CSS `@import` resolution | 1-2 weeks | None | Medium |
| Hex-alpha colors (#rrggbbaa) | 1 day | None | Low |
| Named color resolution | 1 day | None | Low |

**Deliverable:** Full CSS engine matching real-world CSS usage.

**Acceptance Criteria:**
- [ ] `var(--primary, blue)` resolves correctly
- [ ] `@media (max-width: 768px)` generates responsive variants
- [ ] `grid-template-columns: 1fr 1fr 1fr` produces structured output
- [ ] `position: absolute; top: 10px; left: 20px` maps correctly
- [ ] `calc(100% - 40px)` evaluates
- [ ] `@keyframes slide { ... }` produces animation hints

---

## Milestone 3: IR Completion
**Timeline: 1-2 months | Difficulty: Medium | Priority: High**

| Task | Effort | Dependencies | Risk |
|------|:------:|:------------:|:----:|
| Responsive breakpoint detection | 1-2 weeks | M2: media queries | Low |
| Populate TargetIr (Flutter/Compose/SwiftUI) | 1-2 weeks | None | Low |
| Wire diagnostics into IR builder | 1 week | None | Low |
| `aria-labelledby` resolution | 1-2 weeks | None | Low |
| Event handler data from JS → IR | 2-3 weeks | None | Medium |
| Remove dead `LayoutStrategy` enum | 1 day | None | Low |
| Population of `ZStack`, `LazyList` layout variants | 1 week | None | Low |
| Text direction inference | 1 week | None | Low |

**Deliverable:** Complete IR with responsive support, proper TargetIr population, and diagnostics.

**Acceptance Criteria:**
- [ ] `@media (max-width: 768px)` → `ResponsiveVariant { breakpoint: Tablet }`
- [ ] `TargetIr::Flutter { widget: "Row", properties: [...] }` populated
- [ ] IR construction produces diagnostics for issues found
- [ ] `aria-labelledby="name"` resolves to referenced element's text
- [ ] JS `onclick="handleClick()"` → event handler data in IR

---

## Milestone 4: Generator Completion
**Timeline: 2-3 months | Difficulty: Medium | Priority: High**

| Task | Effort | Dependencies | Risk |
|------|:------:|:------------:|:----:|
| Map all CSS properties with computed values | 4-6 weeks | M2: CSS engine | Low |
| Wire image/icon/form data from IR | 2-3 weeks | M3: IR | Low |
| Wire event bindings from JS → generators | 1-2 weeks | M3: event IR | Low |
| Add accessibility attributes to generated code | 2-3 weeks | None | Low |
| Use formatter platform modules | 1 week | None | Low |
| Golden/snapshot tests for all examples | 2-3 weeks | All above | Low |
| Make generators use `TargetIr::Flutter`/`Compose`/`SwiftUI` | 1-2 weeks | None | Low |
| SwiftUI `justifyContent` mapping | 1 day | None | Low |
| Compose `Table` implementation | 1-2 weeks | None | Low |
| SwiftUI `Table` via `LazyVGrid` | 1-2 weeks | None | Low |

**Deliverable:** Production-quality generators producing correct, idiomatic platform code.

**Acceptance Criteria:**
- [ ] All 9 examples produce correct output on all 3 platforms
- [ ] Golden files match generated output
- [ ] CSS properties: width, height, background, border, border-radius, font-size, font-family, text-align, gap, flex-grow, opacity, overflow, position, box-shadow all mapped
- [ ] Event handlers from HTML `onclick` → platform `onPressed`/`onClick`/`.onTapGesture`
- [ ] Accessibility attributes (label, hint, traits) present in generated code
- [ ] No fake data in any generator output

---

## Milestone 5: Performance
**Timeline: 1-2 months | Difficulty: Medium | Priority: High**

| Task | Effort | Dependencies | Risk |
|------|:------:|:------------:|:----:|
| Arena allocation for AST/IR nodes | 4-6 weeks | None | Medium |
| String interning (SymbolId) | 2-3 weeks | None | Medium |
| Lazy computed style | 3-4 weeks | M2: CSS engine | Medium |
| Vec-indexed style map (no HashMap) | 1 week | None | Low |
| Parallel optimization passes | 1 week | None | Low |
| End-to-end pipeline benchmark | 1 week | None | Low |
| Memory allocation benchmarks | 1 week | None | Low |

**Deliverable:** Performance meeting or exceeding targets.

**Performance Targets:**

| Scenario | Current | Target | Milestone 5 Target |
|----------|:-------:|:------:|:------------------:|
| Small page (50 nodes) | ~2ms | ~1ms | <1ms |
| Medium page (500 nodes) | ~10ms | ~5ms | <3ms |
| Large page (5000 nodes) | ~98ms | ~30ms | <20ms |
| Batch (100 pages) | ~1s | ~500ms | <300ms |

---

## Milestone 6: CLI & Tooling
**Timeline: 1-2 months | Difficulty: Easy | Priority: Medium**

| Task | Effort | Dependencies |
|------|:------:|:------------:|
| CLI watch mode (file watcher + debounced recompilation) | 2-3 weeks | M1: incremental |
| Multi-file / directory compilation | 1 week | None |
| Optimization levels (`-O0`, `-O1`, `-O2`) | 1-2 weeks | None |
| Source maps | 3-4 weeks | M4: generators |
| Config file discovery (walk up directories) | 1 week | None |
| `--config` CLI flag | 1 day | None |
| Environment variable overrides | 1 week | None |
| JSON diagnostic output | 1 week | None |
| Shell completions (bash, zsh, fish) | 1 week | None |

---

## Milestone 7: LSP Completion
**Timeline: 1 month | Difficulty: Medium | Priority: Medium**

| Task | Effort | Dependencies |
|------|:------:|:------------:|
| Completion handler (CSS property/value autocomplete) | 1-2 weeks | M1 |
| Hover documentation (CSS property docs on hover) | 1 week | M1 |
| Go to definition (CSS class → style block) | 1 week | M1 |
| Semantic tokens (syntax highlighting via lexer) | 1-2 weeks | M1 |
| Code actions (suppress warning, auto-fix) | 2-3 weeks | M1 |
| Diagnostic push on file save | 1 week | M1 |

---

## Milestone 8: Production Release
**Timeline: 1 month | Difficulty: Easy | Priority: High**

| Task | Effort | Dependencies |
|------|:------:|:------------:|
| Documentation refresh (fix stubs, add missing) | 1-2 weeks | All above |
| Create CHANGELOG.md | 1 day | All above |
| Security audit (`cargo audit`, review unsafe FFI) | 1 week | All above |
| WASM npm package (TypeScript types, publish) | 1-2 weeks | M4 |
| VS Code extension publish | 1 week | M7: LSP |
| crates.io publish of all 23 crates | 1 week | All above |
| Version bump to 1.0.0 | 1 day | All above |
| GitHub Release with artifacts | 1 day | All above |

---

## Summary Timeline

```
Month 1  Month 2  Month 3  Month 4  Month 5  Month 6  Month 7  Month 8  Month 9  Month 10 Month 11 Month 12 Month 13 Month 14 Month 15 Month 16
├──────── M1: Core Stabilization ────────┤
          ├──────── M2: CSS Engine ────────────┤
                    ├── M3: IR ──┤
                              ├──────── M4: Generators ──────────┤
                                        ├── M5: Performance ────┤
                                                  ├── M6: CLI & Tooling ──┤
                                                            ├── M7: LSP ──┤
                                                                      ├── M8: Release ──┤
```

### Dependencies Graph

```
M1 (Core Stabilization)
├── M2 (CSS Engine)
│   ├── M3 (IR Completion)
│   │   ├── M4 (Generator Completion)
│   │   │   ├── M5 (Performance)
│   │   │   │   ├── M6 (CLI & Tooling)
│   │   │   │   │   ├── M7 (LSP)
│   │   │   │   │   │   ├── M8 (Release)
```

**Total estimated time to 1.0: 10-16 months with a team of 2-3 engineers.**
