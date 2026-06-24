import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'fs';
import { resolve, dirname } from 'path';
import { PipelineError, runPipeline as coreRunPipeline } from '@motarjim/pipeline-core';
import type { PlatformTarget } from '@motarjim/shared';
import type { ResolvedOptions, ConversionStats } from '../types.js';
import { countLines, computeOptimizationSavings, generateStatsTable } from './stats.js';
import { createPipelineSpinners } from '../ui/progress.js';

export { PipelineError };

export interface PipelineResult {
  code: string;
  stats: ConversionStats;
}

export async function runPipeline(options: ResolvedOptions): Promise<PipelineResult> {
  const spinners = createPipelineSpinners();
  const startTime = Date.now();

  try {
    spinners.start('Parsing HTML');
    const html = readFileSync(options.input, 'utf-8');
    spinners.succeed('Parsing HTML');

    spinners.start('Parsing CSS');
    let css = '';
    if (options.css) {
      css = readFileSync(options.css, 'utf-8');
    }
    spinners.succeed('Parsing CSS');

    spinners.start('Compiling');
    const result = await coreRunPipeline({
      html,
      css,
      target: options.target as 'flutter' | 'compose' | 'swiftui',
      aiEnhance: options.aiEnhance,
      aiModel: options.aiModel,
    });
    spinners.succeed('Compiling');

    const duration = (Date.now() - startTime) / 1000;

    const stats: ConversionStats = {
      htmlNodes: result.stats.htmlNodes,
      styledNodes: result.stats.styledNodes,
      componentsDetected: result.stats.componentsDetected,
      optimizationSavings: result.stats.optimizationSavings,
      generatedLines: result.stats.generatedLines,
      target: options.target,
      duration,
    };

    return { code: result.code, stats };
  } catch (err) {
    spinners.stopAll();
    throw err;
  }
}

export function writeOutput(code: string, outputPath: string): void {
  const resolved = resolve(outputPath);
  const dir = dirname(resolved);
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  writeFileSync(resolved, code, 'utf-8');
}

export function logStats(stats: ConversionStats): void {
  console.log(generateStatsTable(stats));
}

export function logDryRun(options: ResolvedOptions): void {
  const header = '─'.repeat(40);
  console.log(`\n${header}`);
  console.log('  Dry Run — No files will be generated');
  console.log(`${header}`);
  console.log(`  Input:       ${options.input}`);
  if (options.css) console.log(`  CSS:         ${options.css}`);
  console.log(`  Target:      ${options.target}`);
  if (options.output) console.log(`  Output:      ${options.output}`);
  console.log(`  AI Enhance:  ${options.aiEnhance ? 'Yes' : 'No'}`);
  console.log(`${header}\n`);
}
