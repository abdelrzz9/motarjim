#!/usr/bin/env tsx
// Performance benchmark for html-native-engine.
//
// Measures parse-to-optimize pipeline time for ~1000 HTML nodes.
// Target: under 500ms (pipeline only; code generation reported separately).
//
// Usage: npx tsx benchmarks/run.ts
//   or:  npm run bench

import { readFileSync, existsSync } from 'fs';
import { join } from 'path';
import { parseHtml } from '../packages/parser/index.js';
import { parseCss, applyStyles } from '../packages/css-analyzer/index.js';
import { detectSemantics } from '../packages/semantic-analyzer/index.js';
import { styledNodeToIr } from '../packages/ir/index.js';
import { optimize } from '../packages/optimizer/index.js';
import { generate as generateFlutter } from '../packages/generators/flutter/index.js';
import { generate as generateCompose } from '../packages/generators/compose/index.js';
import { generate as generateSwiftUI } from '../packages/generators/swiftui/index.js';

// ---- Config ----
const ITERATIONS = 10;
const FIXTURE_DIR = join(import.meta.dirname, 'fixtures');
const HTML_FILE = join(FIXTURE_DIR, '1000-nodes.html');
const CSS_FILE = join(FIXTURE_DIR, '1000-nodes.css');

// ---- Helpers ----

function loadFixture(): { html: string; css: string } {
  if (!existsSync(HTML_FILE)) throw new Error(`Fixture not found: ${HTML_FILE}\nRun: npx tsx benchmarks/fixtures/generate.ts`);
  if (!existsSync(CSS_FILE)) throw new Error(`Fixture not found: ${CSS_FILE}\nRun: npx tsx benchmarks/fixtures/generate.ts`);
  return {
    html: readFileSync(HTML_FILE, 'utf-8'),
    css: readFileSync(CSS_FILE, 'utf-8'),
  };
}

function measure(fn: () => void): number {
  const start = performance.now();
  fn();
  return performance.now() - start;
}

function stats(values: number[]): { min: number; max: number; mean: number; median: number } {
  const sorted = [...values].sort((a, b) => a - b);
  const n = sorted.length;
  return {
    min: sorted[0],
    max: sorted[n - 1],
    mean: values.reduce((s, v) => s + v, 0) / n,
    median: n % 2 === 0 ? (sorted[n / 2 - 1] + sorted[n / 2]) / 2 : sorted[Math.floor(n / 2)],
  };
}

function formatRow(label: string, s: { min: number; max: number; mean: number; median: number }): string {
  const meanStr = s.mean.toFixed(2).padStart(8);
  const minStr = s.min.toFixed(2).padStart(8);
  const maxStr = s.max.toFixed(2).padStart(8);
  const medianStr = s.median.toFixed(2).padStart(8);
  return `  ${label.padEnd(24)} mean ${meanStr}  min ${minStr}  max ${maxStr}  median ${medianStr}`;
}

function countUiNodes(node: { children: { reduce: (fn: any, init: number) => number } }): number {
  return 1 + node.children.reduce((sum: number, c: any) => sum + countUiNodes(c), 0);
}

function countHtmlTags(html: string): number {
  return (html.match(/<(\w+)[\s>]/g) || []).length;
}

// ---- Main ----

function main(): void {
  console.log('=== html-native-engine Performance Benchmark ===\n');

  // Load fixtures
  console.log('Loading fixture...');
  const { html, css } = loadFixture();
  const tagCount = countHtmlTags(html);
  console.log(`  HTML tags in fixture: ${tagCount}`);
  console.log(`  CSS length: ${css.length} chars`);
  console.log(`  Iterations: ${ITERATIONS}\n`);

  // Warm-up runs
  console.log('Warm-up...');
  for (let w = 0; w < 3; w++) {
    const ast = parseHtml(html);
    const sheet = parseCss(css);
    const styled = applyStyles(ast.children, sheet);
    const hints = detectSemantics(styled);
    const root = { node: ast, styles: {}, children: styled };
    const ir = styledNodeToIr(root, hints);
    optimize(ir);
    generateFlutter(ir);
    generateCompose(ir);
    generateSwiftUI(ir);
  }
  console.log('  Done.\n');

  // Collect measurements per phase across iterations
  type Phase = 'parseHtml' | 'parseCss' | 'applyStyles' | 'semanticAnalysis' | 'irConversion' | 'optimization' | 'genFlutter' | 'genCompose' | 'genSwiftUI';
  const allMeasurements: Record<Phase, number[]> = {
    parseHtml: [],
    parseCss: [],
    applyStyles: [],
    semanticAnalysis: [],
    irConversion: [],
    optimization: [],
    genFlutter: [],
    genCompose: [],
    genSwiftUI: [],
  };

  let irNodeCount = 0;

  for (let iter = 0; iter < ITERATIONS; iter++) {
    // Phase 1: Parse HTML
    let ast: ReturnType<typeof parseHtml>;
    allMeasurements.parseHtml.push(measure(() => { ast = parseHtml(html); }) as number);

    // Phase 2: Parse CSS
    let sheet: ReturnType<typeof parseCss>;
    allMeasurements.parseCss.push(measure(() => { sheet = parseCss(css); }) as number);

    // Phase 3: Apply styles
    let styled: ReturnType<typeof applyStyles>;
    allMeasurements.applyStyles.push(measure(() => { styled = applyStyles((ast!).children, sheet!); }) as number);

    // Phase 4: Semantic analysis
    let hints: ReturnType<typeof detectSemantics>;
    allMeasurements.semanticAnalysis.push(measure(() => { hints = detectSemantics(styled!); }) as number);

    // Phase 5: IR conversion
    let ir: ReturnType<typeof styledNodeToIr>;
    allMeasurements.irConversion.push(measure(() => {
      const root = { node: ast!, styles: {}, children: styled! };
      ir = styledNodeToIr(root, hints!);
    }) as number);

    // Phase 6: Optimization
    let optimized: ReturnType<typeof optimize>;
    allMeasurements.optimization.push(measure(() => { optimized = optimize(ir!); }) as number);

    // Count IR nodes on first iteration
    if (iter === 0) {
      irNodeCount = countUiNodes(optimized!);
    }

    // Phase 7: Code generation (reported separately from pipeline target)
    allMeasurements.genFlutter.push(measure(() => generateFlutter(optimized!)));
    allMeasurements.genCompose.push(measure(() => generateCompose(optimized!)));
    allMeasurements.genSwiftUI.push(measure(() => generateSwiftUI(optimized!)));
  }

  // ---- Results ----
  console.log('--- Per-phase timing (ms) ---\n');

  const pipelinePhases: Phase[] = ['parseHtml', 'parseCss', 'applyStyles', 'semanticAnalysis', 'irConversion', 'optimization'];
  const genPhases: Phase[] = ['genFlutter', 'genCompose', 'genSwiftUI'];

  const phaseLabels: Record<Phase, string> = {
    parseHtml: 'Parse HTML',
    parseCss: 'Parse CSS',
    applyStyles: 'Apply styles',
    semanticAnalysis: 'Semantic analysis',
    irConversion: 'IR conversion',
    optimization: 'Optimization',
    genFlutter: 'Flutter code gen',
    genCompose: 'Compose code gen',
    genSwiftUI: 'SwiftUI code gen',
  };

  for (const phase of [...pipelinePhases, ...genPhases]) {
    console.log(formatRow(phaseLabels[phase], stats(allMeasurements[phase])));
  }

  // Compute pipeline total (excl generators)
  const pipelineMedians = pipelinePhases.map(p => stats(allMeasurements[p]).median);
  const pipelineTotalMedian = pipelineMedians.reduce((s, v) => s + v, 0);
  const pipelineTotalMean = pipelinePhases.reduce((s, p) => s + stats(allMeasurements[p]).mean, 0);

  // Generator totals
  const genMedians = genPhases.map(p => stats(allMeasurements[p]).median);
  const genTotalMedian = genMedians.reduce((s, v) => s + v, 0);

  const allPhases = [...pipelinePhases, ...genPhases];
  const allTotalMedian = allPhases.reduce((s, p) => s + stats(allMeasurements[p]).median, 0);

  console.log('');
  console.log('--- Summary ---\n');
  console.log(`  Pipeline (parse → optimize) median: ${pipelineTotalMedian.toFixed(2).padStart(8)}ms  (target: < 500ms)`);
  console.log(`  Pipeline (parse → optimize) mean:   ${pipelineTotalMean.toFixed(2).padStart(8)}ms`);
  console.log(`  Generators combined median:         ${genTotalMedian.toFixed(2).padStart(8)}ms`);
  console.log(`  Total (pipeline + all 3 gens):      ${allTotalMedian.toFixed(2).padStart(8)}ms`);
  console.log('');
  console.log(`  IR nodes after optimization:         ${irNodeCount}`);
  console.log(`  HTML tags in fixture:                ${tagCount}`);
  console.log(`  Target met (< 500ms):                ${pipelineTotalMedian < 500 ? '✓ PASS' : '✗ FAIL'}`);
  console.log('');
}

main();
