import type {
  CompileRequest, CompileResult, Diagnostic,
  ASTNode, CSSRule, CSSDeclaration, PipelineStage,
} from './types';
import { parseHtml } from './htmlParser';
import { parseCss, findMatchingRules } from './cssParser';
import { generateFlutter } from './codeGenerators/flutterGenerator';
import { generateCompose } from './codeGenerators/composeGenerator';
import { generateSwiftUI } from './codeGenerators/swiftuiGenerator';
import { compilerCache } from './cache';
import { logger } from './logger';

export type StageCallback = (stage: PipelineStage) => void;

let abortController: AbortController | null = null;

export function cancelCurrentCompilation(): void {
  if (abortController) {
    abortController.abort();
    abortController = null;
    logger.info('Pipeline', 'Compilation cancelled');
  }
}

export function isCompilationActive(): boolean {
  return abortController !== null && !abortController.signal.aborted;
}

export async function compile(
  request: CompileRequest,
  onStage?: StageCallback,
): Promise<CompileResult> {
  cancelCurrentCompilation();
  abortController = new AbortController();
  const signal = abortController.signal;

  const startTime = performance.now();

  function checkAborted(): void {
    if (signal.aborted) {
      throw new Error('Compilation cancelled');
    }
  }

  function reportStage(stage: PipelineStage): void {
    if (onStage) onStage(stage);
  }

  try {
    const cached = compilerCache.get(request);
    if (cached) {
      logger.info('Pipeline', 'Cache hit, returning cached result');
      reportStage('complete');
      const elapsed = performance.now() - startTime;
      return {
        ...cached,
        stats: { ...cached.stats, timeMs: Math.round(elapsed) },
      };
    }

    const allDiagnostics: Diagnostic[] = [];
    const cssDeclMap = new Map<string, CSSDeclaration[]>();

    reportStage('parsing_html');
    checkAborted();
    const parseStart = performance.now();
    const { ast: htmlAst, diagnostics: htmlDiags } = parseHtml(request.html);
    const parseTime = performance.now() - parseStart;
    allDiagnostics.push(...htmlDiags);

    reportStage('parsing_css');
    checkAborted();
    let cssRules: CSSRule[] = [];
    if (request.css && request.css.trim()) {
      const cssResult = parseCss(request.css);
      cssRules = cssResult.rules;
      allDiagnostics.push(...cssResult.diagnostics);
    }

    const ruleCount = cssRules.length;

    reportStage('building_ast');
    checkAborted();
    buildCssMap(htmlAst, cssRules, cssDeclMap);

    let jsNodes = 0;

    reportStage('building_ir');
    checkAborted();
    const irNodes = buildIr(htmlAst);

    reportStage('optimizing');
    checkAborted();

    reportStage('generating_code');
    checkAborted();
    const genStart = performance.now();
    let code: string;

    switch (request.platform) {
      case 'flutter':
        code = generateFlutter(htmlAst, cssDeclMap, allDiagnostics, request.minify ?? false);
        break;
      case 'compose':
        code = generateCompose(htmlAst, cssDeclMap, allDiagnostics, request.minify ?? false);
        break;
      case 'swiftui':
        code = generateSwiftUI(htmlAst, cssDeclMap, allDiagnostics, request.minify ?? false);
        break;
      default:
        code = '// Unsupported platform';
    }

    const genTime = performance.now() - genStart;
    const totalTime = performance.now() - startTime;

    const hasErrors = allDiagnostics.some(d => d.severity === 'error');

    const result: CompileResult = {
      success: !hasErrors,
      code,
      diagnostics: allDiagnostics,
      stats: {
        nodesParsed: countNodes(htmlAst),
        cssRules: ruleCount,
        irNodes,
        jsNodes,
        timeMs: Math.round(totalTime),
        parseTimeMs: Math.round(parseTime),
        genTimeMs: Math.round(genTime),
      },
      ast: htmlAst,
      ir: { type: 'ir', nodes: irNodes },
    };

    if (!hasErrors) {
      compilerCache.set(request, result);
    }

    reportStage('complete');

    logger.info('Pipeline', 'Compilation completed', {
      platform: request.platform,
      timeMs: Math.round(totalTime),
      nodes: result.stats.nodesParsed,
      rules: result.stats.cssRules,
      diagnostics: allDiagnostics.length,
      errors: allDiagnostics.filter(d => d.severity === 'error').length,
    });

    return result;
  } catch (err) {
    if (signal.aborted) {
      const abortResult: CompileResult = {
        success: false,
        code: '',
        diagnostics: [{
          severity: 'info',
          code: 'I0001',
          title: 'Compilation Cancelled',
          explanation: 'The compilation was cancelled by the user.',
          suggestions: [],
          notes: [],
        }],
        stats: {
          nodesParsed: 0, cssRules: 0, irNodes: 0, jsNodes: 0,
          timeMs: Math.round(performance.now() - startTime),
          parseTimeMs: 0, genTimeMs: 0,
        },
      };
      reportStage('failed');
      return abortResult;
    }

    logger.error('Pipeline', 'Compilation failed', { error: String(err) });
    reportStage('failed');

    const errorResult: CompileResult = {
      success: false,
      code: '',
      diagnostics: [{
        severity: 'error',
        code: 'E0001',
        title: 'Compilation Failed',
        explanation: `An unexpected error occurred during compilation: ${err instanceof Error ? err.message : String(err)}`,
        suggestions: ['Check your HTML and CSS for syntax errors.', 'Try simplifying the input.'],
        notes: [],
      }],
      stats: {
        nodesParsed: 0, cssRules: 0, irNodes: 0, jsNodes: 0,
        timeMs: Math.round(performance.now() - startTime),
        parseTimeMs: 0, genTimeMs: 0,
      },
    };

    return errorResult;
  } finally {
    if (abortController && !abortController.signal.aborted) {
      abortController = null;
    }
  }
}

function buildCssMap(
  node: ASTNode,
  rules: CSSRule[],
  map: Map<string, CSSDeclaration[]>,
): void {
  if (node.type === 'element' && node.tagName) {
    const attrs = node.attributes || {};
    const classes = (attrs.class || '').split(/\s+/).filter(Boolean);
    const id = attrs.id;

    const matching = findMatchingRules(rules, node.tagName, classes, id);
    if (matching.length > 0) {
      const key = `${node.tagName}|${classes.join(',')}|${id || ''}`;
      map.set(key, matching);
    }

    if (classes.length > 0) {
      for (const cls of classes) {
        const key = `.${cls}`;
        if (!map.has(key)) {
          const clsRules = findMatchingRules(rules, '', [cls], id);
          if (clsRules.length > 0) map.set(key, clsRules);
        }
      }
    }

    if (id) {
      const key = `#${id}`;
      if (!map.has(key)) {
        const idRules = findMatchingRules(rules, '', [], id);
        if (idRules.length > 0) map.set(key, idRules);
      }
    }

    const tagKey = node.tagName;
    if (!map.has(tagKey)) {
      const tagRules = findMatchingRules(rules, node.tagName, [], undefined);
      if (tagRules.length > 0) map.set(tagKey, tagRules);
    }
  }

  if (node.children) {
    for (const child of node.children) {
      buildCssMap(child, rules, map);
    }
  }
}

function buildIr(node: ASTNode): number {
  if (!node.children) return 0;
  let count = 0;
  for (const _ of node.children) {
    count++;
  }
  for (const child of node.children) {
    count += buildIr(child);
  }
  return count;
}

function countNodes(node: ASTNode): number {
  let count = 1;
  if (node.children) {
    for (const child of node.children) {
      count += countNodes(child);
    }
  }
  return count;
}
