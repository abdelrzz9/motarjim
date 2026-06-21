import postcss from 'postcss';
import type { CssStylesheet, CssRule, CssMediaQuery, HtmlNode, ResolvedStyles, StyledNode, ResponsiveHint, Result, Specificity } from '@html-native/shared';
import { DiagnosticBag } from '@html-native/shared/diagnostics.js';
import { parseSelector, calculateSpecificity, matchAst, matchSelectorString, type ParentResolver } from './selector.js';
import { cascadeStyles, createParentResolver, inheritMissingProperties, parseInlineStyles } from './cascade.js';

// -- Re-export selector and cascade types/functions --

export type { ParentResolver } from './selector.js';
export {
  parseSelector,
  calculateSpecificity,
  matchAst,
  matchSelectorString,
  parseSelectorList,
} from './selector.js';
export {
  cascadeStyles,
  createParentResolver,
  inheritMissingProperties,
  parseInlineStyles,
  INHERITED_PROPERTIES,
} from './cascade.js';

export { detectLayoutIntent, analyzeLayoutIntents, describeLayout, LAYOUT_PATTERNS } from './intent.js';
export {
  extractBreakpoints,
  detectMobileFirst,
  buildResponsiveMetadata,
  detectResponsivePatterns,
  classifyBreakpoint,
  BREAKPOINT_LABELS,
} from './responsive.js';
export type { ResponsivePattern } from './responsive.js';

function parseDeclarations(declNodes: postcss.ChildNode[]): CssRule['declarations'] {
  const declarations: CssRule['declarations'] = [];
  for (const decl of declNodes) {
    if (decl.type === 'decl') {
      declarations.push({
        property: decl.prop,
        value: decl.value.replace(/!important\s*$/, '').trim(),
        important: decl.important,
      });
    }
  }
  return declarations;
}

function parseSelectors(selector: string): string[] {
  return selector.split(',').map(s => s.trim()).filter(Boolean);
}

export function parseCss(css: string, file: string = 'styles.css'): Result<CssStylesheet> {
  const bag = new DiagnosticBag();
  const result: CssStylesheet = { rules: [], mediaQueries: [] };

  if (!css.trim()) {
    bag.addInfo('CSS_001', 'Empty CSS input, returning empty stylesheet', 'css');
    return bag.toResult(result);
  }

  let root: postcss.Root;
  try {
    root = postcss.parse(css);
  } catch (err) {
    bag.addError('CSS_002', `Failed to parse CSS: ${(err as Error).message}`, 'css', {
      file,
      start: { line: 0, column: 0 },
      end: { line: 0, column: 0 },
    });
    return bag.asResult();
  }

  for (const node of root.nodes) {
    if (node.type === 'rule' && node.nodes) {
      for (const sel of parseSelectors(node.selector)) {
        result.rules.push({
          selectors: [sel],
          declarations: parseDeclarations(node.nodes),
        });
      }
    } else if (node.type === 'atrule' && node.name === 'media' && node.nodes) {
      const mediaRules: CssRule[] = [];
      for (const child of node.nodes) {
        if (child.type === 'rule' && child.nodes) {
          for (const sel of parseSelectors(child.selector)) {
            mediaRules.push({
              selectors: [sel],
              declarations: parseDeclarations(child.nodes),
            });
          }
        }
      }
      if (mediaRules.length > 0) {
        result.mediaQueries.push({
          condition: node.params,
          rules: mediaRules,
        });
      }
    }
  }

  return bag.toResult(result);
}

export function extractResponsiveHints(stylesheet: CssStylesheet): ResponsiveHint[] {
  const hints: ResponsiveHint[] = [];

  for (const mq of stylesheet.mediaQueries) {
    const condition = mq.condition;
    const minMatch = condition.match(/min-width\s*:\s*([^)\s]+)/);
    const maxMatch = condition.match(/max-width\s*:\s*([^)\s]+)/);
    const minHMatch = condition.match(/min-height\s*:\s*([^)\s]+)/);
    const maxHMatch = condition.match(/max-height\s*:\s*([^)\s]+)/);

    let conditionType: ResponsiveHint['condition'] | null = null;
    let value = '';

    if (minMatch) { conditionType = 'min-width'; value = minMatch[1]; }
    else if (maxMatch) { conditionType = 'max-width'; value = maxMatch[1]; }
    else if (minHMatch) { conditionType = 'min-height'; value = minHMatch[1]; }
    else if (maxHMatch) { conditionType = 'max-height'; value = maxHMatch[1]; }

    if (conditionType) {
      for (const rule of mq.rules) {
        const styles: ResolvedStyles = {};
        for (const decl of rule.declarations) {
          styles[decl.property] = decl.value;
        }
        hints.push({
          breakpoint: `${conditionType}: ${value}`,
          condition: conditionType,
          value,
          styles,
        });
      }
    }
  }

  return hints;
}

// -- Backward-compatible wrapper using new selector engine --

export function matchSelector(selector: string, node: HtmlNode): boolean {
  return matchSelectorString(selector, node);
}

// -- Cascade-based style resolution --

export function resolveStyles(
  node: HtmlNode,
  stylesheet: CssStylesheet,
  getParent?: ParentResolver,
  parentStyles?: ResolvedStyles | null,
): ResolvedStyles {
  const resolver = getParent ?? (() => null);
  return cascadeStyles(node, stylesheet, resolver, parentStyles ?? null);
}

// -- Apply styles using cascade with inheritance --

export function applyStyles(nodes: HtmlNode[], stylesheet: CssStylesheet, file: string = 'input.html'): Result<StyledNode[]> {
  const bag = new DiagnosticBag();

  const getParent = createParentResolver(nodes);

  function apply(nodes: HtmlNode[], parentStyles: ResolvedStyles | null): StyledNode[] {
    return nodes.map(node => {
      const styles = cascadeStyles(node, stylesheet, getParent, parentStyles);
      return {
        node,
        styles,
        children: apply(node.children, styles),
      };
    });
  }

  if (stylesheet.rules.length === 0 && stylesheet.mediaQueries.length === 0) {
    bag.addInfo('CSS_003', 'No CSS rules to apply, nodes will be unstyled', 'css');
  }

  const styled = apply(nodes, null);
  return bag.toResult(styled);
}
