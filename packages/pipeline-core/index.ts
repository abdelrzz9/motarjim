import { parseHtml } from '@motarjim/parser';
import { parseCss, applyStyles, analyzeLayoutIntents, buildResponsiveMetadata } from '@motarjim/css-analyzer';
import { detectSemantics } from '@motarjim/semantic-analyzer';
import { analyzeAccessibility } from '@motarjim/accessibility-analyzer';
import { styledNodeToIr, enrichWithIntent, enrichWithIntentSync } from '@motarjim/ir';
import { styledNodeToIrV2 } from '@motarjim/ir-v2';
import { optimize } from '@motarjim/optimizer';
import { generate as generateFlutter } from '@motarjim/generator-flutter';
import { generateIr as generateFlutterIr } from '@motarjim/generator-flutter/ir-generate.js';
import { generate as generateCompose } from '@motarjim/generator-compose';
import { generate as generateSwiftUI } from '@motarjim/generator-swiftui';
import type { StyledNode, GenerateResult, Result, Diagnostic, ResponsiveMetadata } from '@motarjim/shared';
import type { IrNode } from '@motarjim/shared/ir-v2.js';
import { formatDiagnostics } from '@motarjim/shared/diagnostics.js';
import { countHtmlNodes, countNodes as countUiNodes, countComponentNodes } from '@motarjim/shared/count-nodes.js';

export type Target = 'flutter' | 'compose' | 'swiftui';

export interface PipelineInput {
  html: string;
  css: string;
  target: Target;
  aiEnhance?: boolean;
  aiModel?: string;
}

export interface PipelineStats {
  htmlNodes: number;
  styledNodes: number;
  componentsDetected: number;
  optimizationSavings: number;
  generatedLines: number;
  target: Target;
  duration: number;
}

export interface PipelineResult {
  code: string;
  stats: PipelineStats;
}

export class PipelineError extends Error {
  diagnostics: Diagnostic[];

  constructor(diagnostics: Diagnostic[]) {
    super(formatDiagnostics(diagnostics));
    this.name = 'PipelineError';
    this.diagnostics = diagnostics;
  }
}

function requireOk<T>(result: Result<T>, phase: string): T {
  if (!result.ok) {
    throw new PipelineError(result.diagnostics);
  }
  return result.value;
}

function attachResponsiveMetadata(ir: UiNode, metadata: ResponsiveMetadata): UiNode {
  function walk(node: UiNode): UiNode {
    return {
      ...node,
      responsiveMetadata: metadata,
      children: node.children.map(walk),
    };
  }
  return walk(ir);
}

/**
 * @deprecated Use runIrPipeline instead.
 * runPipeline uses the legacy UiNode-based pipeline. New code should use
 * runIrPipeline which produces IrNode trees with typed ComputedStyle,
 * SemanticIR, and LayoutIR.
 */
export async function runPipeline(input: PipelineInput): Promise<PipelineResult> {
  const { html, css, target, aiEnhance, aiModel } = input;
  const startTime = Date.now();

  const ast = requireOk(parseHtml(html), 'parser');
  const htmlNodeCount = countHtmlNodes(ast);

  const stylesheet = requireOk(parseCss(css || ''), 'css');

  let styledNodes: StyledNode[] = requireOk(applyStyles(ast.children, stylesheet), 'css');
  const styledCount = styledNodes.reduce((acc, n) => acc + countHtmlNodes(n.node), 0);
  styledNodes = analyzeLayoutIntents(styledNodes);

  const responsiveMetadata = buildResponsiveMetadata(stylesheet);

  let hints;
  if (aiEnhance) {
    const { createAiDetector } = await import('@motarjim/semantic-analyzer/ai');
    const aiDetector = createAiDetector(aiModel ? { model: aiModel } : undefined);
    hints = await aiDetector(styledNodes);
  } else {
    const semanticResult = detectSemantics(styledNodes);
    hints = requireOk(semanticResult, 'semantic');
  }

  const a11yResult = analyzeAccessibility(styledNodes);
  const a11y = requireOk(a11yResult, 'accessibility');

  const rootStyled: StyledNode = {
    node: ast,
    styles: {},
    children: styledNodes,
    layoutIntent: { type: 'Stack', properties: {}, confidence: 1 },
  };

  let ir = requireOk(styledNodeToIr(rootStyled, hints, a11y.perNodeInfo), 'ir');

  if (responsiveMetadata.breakpoints.length > 0) {
    ir = attachResponsiveMetadata(ir, responsiveMetadata);
  }

  if (aiEnhance) {
    ir = await enrichWithIntent(ir, { enabled: true, aiConfig: aiModel ? { model: aiModel } : undefined });
  } else {
    ir = enrichWithIntentSync(ir);
  }

  const componentsDetected = countComponentNodes(ir);

  const irBefore = structuredClone(ir);
  const optimized = requireOk(optimize(ir), 'optimizer');
  const originalCount = countUiNodes(irBefore);
  const optimizedCount = countUiNodes(optimized);
  const savings = originalCount > 0 ? Math.round(((originalCount - optimizedCount) / originalCount) * 100) : 0;

  let result: GenerateResult;
  switch (target) {
    case 'flutter':
      result = requireOk(generateFlutter(optimized), 'generator');
      break;
    case 'compose':
      result = requireOk(generateCompose(optimized), 'generator');
      break;
    case 'swiftui':
      result = requireOk(generateSwiftUI(optimized), 'generator');
      break;
    default:
      throw new Error(`Unknown target "${target}"`);
  }

  const duration = (Date.now() - startTime) / 1000;

  return {
    code: result.code,
    stats: {
      htmlNodes: htmlNodeCount,
      styledNodes: styledCount,
      componentsDetected,
      optimizationSavings: savings,
      generatedLines: result.code.split('\n').length,
      target,
      duration,
    },
  };
}

// ============================================================
// IR v2 Pipeline — uses the three-layer IR architecture
// ============================================================

export async function runIrPipeline(input: PipelineInput): Promise<PipelineResult> {
  const { html, css, target, aiEnhance, aiModel } = input;
  const startTime = Date.now();

  const ast = requireOk(parseHtml(html), 'parser');
  const htmlNodeCount = countHtmlNodes(ast);

  const stylesheet = requireOk(parseCss(css || ''), 'css');

  let styledNodes: StyledNode[] = requireOk(applyStyles(ast.children, stylesheet), 'css');
  const styledCount = styledNodes.reduce((acc, n) => acc + countHtmlNodes(n.node), 0);
  styledNodes = analyzeLayoutIntents(styledNodes);

  const responsiveMetadata = buildResponsiveMetadata(stylesheet);

  // IR v2: Build IrNode tree directly from styled nodes
  const rootStyled: StyledNode = {
    node: ast,
    styles: {},
    children: styledNodes,
    layoutIntent: { type: 'Stack', properties: {}, confidence: 1 },
  };

  const irNode = styledNodeToIrV2(rootStyled);

  // Attach responsive metadata
  function attachResponsiveMetadataIr(node: IrNode, metadata: ResponsiveMetadata): IrNode {
    return {
      ...node,
      responsiveVariants: metadata.breakpoints.map(bp => ({
        breakpoint: { condition: 'min-width' as const, value: bp.value },
        layoutOverrides: {},
        styleOverrides: {},
      })),
      children: node.children.map(c => attachResponsiveMetadataIr(c, metadata)),
    };
  }

  let ir: IrNode = irNode;
  if (responsiveMetadata.breakpoints.length > 0) {
    ir = attachResponsiveMetadataIr(ir, responsiveMetadata);
  }

  const componentsDetected = 0;

  let result: GenerateResult;
  switch (target) {
    case 'flutter':
      result = requireOk(generateFlutterIr(ir), 'generator');
      break;
    case 'compose':
    case 'swiftui':
      throw new Error(`IR v2 pipeline does not yet support target "${target}". Use runPipeline (legacy) for non-Flutter targets.`);
    default:
      throw new Error(`Unknown target "${target}"`);
  }

  const duration = (Date.now() - startTime) / 1000;

  return {
    code: result.code,
    stats: {
      htmlNodes: htmlNodeCount,
      styledNodes: styledCount,
      componentsDetected,
      optimizationSavings: 0,
      generatedLines: result.code.split('\n').length,
      target,
      duration,
    },
  };
}
