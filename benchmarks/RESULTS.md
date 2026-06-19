# Performance Benchmark Results

## Setup

- **Date:** 2026-06-19
- **Fixture:** `benchmarks/fixtures/1000-nodes.html` + `1000-nodes.css`
- **Definition of "1000 nodes":** 1001 raw HTML elements (non-text `<tags>`) in the input.
  Post-optimization IR node count is 1209 (the optimizer adds wrappers for multiple children).
- **Method:** 10 iterations per phase, reporting min/max/mean/median via `performance.now()`.
  3 warm-up runs before measurement.
- **Pipeline measured:** parseHtml → parseCss → applyStyles → detectSemantics → styledNodeToIr → optimize
- **Machine:** Linux x64, Node.js v20.20.2

## Results (median of 10 iterations)

| Phase                | Median (ms) |
|----------------------|-------------|
| Parse HTML           | 23.31       |
| Parse CSS            | 6.97        |
| Apply styles         | 54.47       |
| Semantic analysis    | 4.21        |
| IR conversion        | 28.47       |
| Optimization         | 3.51        |
| **Pipeline total**   | **120.94**  |
| Flutter code gen     | 6.78        |
| Compose code gen     | 3.52        |
| SwiftUI code gen     | 3.88        |
| Generators combined  | 14.18       |
| **Total (all 3 gens)** | **135.12** |

## Verdict

**✓ PASS — 120.94ms pipeline median is well under the 500ms target (4.1× headroom).**

With all three generators combined: 135.12ms total — still under 500ms if measuring end-to-end.

## Phase Distribution

The pipeline time breaks down as:

1. **Apply styles: 54.47ms (45%)** — The dominant phase. For each of 1001 nodes, `resolveStyles()` iterates all CSS rules and runs selector matching. With ~50 CSS rules, this is ~50,000 selector match calls. This is expected O(n*m) behavior and acceptable at the current scale.
2. **IR conversion: 28.47ms (24%)** — Convert StyledNode tree to UiNode tree, including text propagation and semantic hint integration.
3. **Parse HTML: 23.31ms (19%)** — parse5 parsing of the full document.
4. **Parse CSS: 6.97ms (6%)** — PostCSS parsing.
5. **Semantic analysis: 4.21ms (3%)** — Rule-based detection over the styled tree.
6. **Optimization: 3.51ms (3%)** — Three passes (removeEmptyText, mergeTextNodes, flattenContainers).

## Notes

- The 500ms target has significant headroom. No optimization of the pipeline is warranted at this scale.
- If scaling to 10,000+ nodes, the `applyStyles` phase (O(n*m) selector matching) would be the first bottleneck.
  A precomputed rule index (e.g., bucket selectors by tag/class/id) would reduce that to O(n+k).
- Generator code gen time is negligible compared to the pipeline (median 14ms total for all 3 platforms).

## Re-running

```bash
# Generate fixtures (already committed; re-run if you modify the generator)
npm run gen-fixtures

# Run the benchmark
npm run bench
```
