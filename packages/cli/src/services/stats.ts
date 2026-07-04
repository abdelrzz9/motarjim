import type { UiNode, PlatformTarget } from '@motarjim/shared';
import type { ConversionStats } from '../types.js';
import {
  countNodes as sharedCountNodes,
  countComponentNodes as sharedCountComponentNodes,
} from '@motarjim/shared/count-nodes.js';

export function countNodes(node: UiNode): number {
  return sharedCountNodes(node);
}

export function countComponentNodes(node: UiNode): number {
  return sharedCountComponentNodes(node);
}

export function countLines(code: string): number {
  return code.split('\n').length;
}

export function computeOptimizationSavings(original: UiNode, optimized: UiNode): number {
  const originalCount = sharedCountNodes(original);
  const optimizedCount = sharedCountNodes(optimized);
  if (originalCount === 0) return 0;
  return Math.round(((originalCount - optimizedCount) / originalCount) * 100);
}

export function generateStatsTable(stats: ConversionStats): string {
  const lines = [
    '─'.repeat(24),
    `  HTML Nodes:              ${stats.htmlNodes}`,
    `  Styled Nodes:            ${stats.styledNodes}`,
    `  Components Detected:     ${stats.componentsDetected}`,
    `  Optimization Savings:    ${stats.optimizationSavings}%`,
    `  Generated Lines:         ${stats.generatedLines}`,
    `  Target:                  ${stats.target.charAt(0).toUpperCase() + stats.target.slice(1)}`,
    `  Duration:                ${stats.duration.toFixed(2)}s`,
    '─'.repeat(24),
  ];
  return `\n${lines.join('\n')}\n`;
}
