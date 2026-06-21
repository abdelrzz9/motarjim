import type { HtmlNode, CssStylesheet, CssStyleDeclaration, ResolvedStyles, Specificity } from '@html-native/shared';
import { parseSelector, calculateSpecificity, matchAst, type ParentResolver } from './selector.js';

// -- Inheritable properties --

export const INHERITED_PROPERTIES: ReadonlySet<string> = new Set([
  'color',
  'font',
  'font-family',
  'font-size',
  'font-style',
  'font-variant',
  'font-weight',
  'letter-spacing',
  'line-height',
  'text-align',
  'text-indent',
  'text-transform',
  'text-decoration',
  'visibility',
  'white-space',
  'word-break',
  'word-spacing',
  'word-wrap',
  'direction',
  'list-style',
  'list-style-type',
  'list-style-position',
  'list-style-image',
]);

// -- Inline style parser --

export function parseInlineStyles(style: string): CssStyleDeclaration[] {
  const decls: CssStyleDeclaration[] = [];
  const parts = style.split(';');
  for (const part of parts) {
    const trimmed = part.trim();
    if (!trimmed) continue;
    const colonIdx = trimmed.indexOf(':');
    if (colonIdx === -1) continue;
    const prop = trimmed.slice(0, colonIdx).trim();
    if (!prop) continue;
    const rawValue = trimmed.slice(colonIdx + 1).trim();
    const important = rawValue.toLowerCase().includes('!important');
    const value = important
      ? rawValue.replace(/!important\s*$/i, '').trim()
      : rawValue;
    decls.push({ property: prop, value, important });
  }
  return decls;
}

// -- Cascade entry --

interface CascadeEntry {
  property: string;
  value: string;
  specificity: Specificity;
  sourceOrder: number;
  important: boolean;
  inline: boolean;
}

function cascadeCompare(a: CascadeEntry, b: CascadeEntry): number {
  if (a.important !== b.important) return a.important ? 1 : -1;
  if (a.inline !== b.inline) return a.inline ? 1 : -1;

  const diff =
    (b.specificity.id - a.specificity.id) ||
    (b.specificity.class - a.specificity.class) ||
    (b.specificity.tag - a.specificity.tag);
  if (diff !== 0) return diff;

  return a.sourceOrder - b.sourceOrder;
}

// -- Inheritance --

export function inheritMissingProperties(styles: ResolvedStyles, parentStyles: ResolvedStyles): void {
  for (const prop of INHERITED_PROPERTIES) {
    if (!(prop in styles) && prop in parentStyles) {
      styles[prop] = parentStyles[prop];
    }
  }
}

// -- Build parent map --

export function buildParentMap(nodes: HtmlNode[]): Map<string, string> {
  const map = new Map<string, string>();

  function walk(children: HtmlNode[], parentId: string | null): void {
    for (const child of children) {
      if (parentId !== null) {
        map.set(child.nodeId, parentId);
      }
      walk(child.children, child.nodeId);
    }
  }

  walk(nodes, null);
  return map;
}

export function createParentResolver(nodes: HtmlNode[]): ParentResolver {
  const parentMap = buildParentMap(nodes);
  const nodeMap = new Map<string, HtmlNode>();

  function index(n: HtmlNode[]): void {
    for (const child of n) {
      nodeMap.set(child.nodeId, child);
      index(child.children);
    }
  }
  index(nodes);

  return (nodeId: string): HtmlNode | null => {
    const parentId = parentMap.get(nodeId);
    if (!parentId) return null;
    return nodeMap.get(parentId) ?? null;
  };
}

// -- Main cascade function --

export function cascadeStyles(
  node: HtmlNode,
  stylesheet: CssStylesheet,
  getParent: ParentResolver,
  parentStyles: ResolvedStyles | null,
): ResolvedStyles {
  const entries: CascadeEntry[] = [];

  // 1. Collect matching rules from stylesheet
  stylesheet.rules.forEach((rule, ruleIndex) => {
    for (const selText of rule.selectors) {
      const ast = parseSelector(selText);
      if (!ast) continue;
      if (!matchAst(ast, node, getParent)) continue;
      const spec = calculateSpecificity(ast);
      for (const decl of rule.declarations) {
        entries.push({
          property: decl.property,
          value: decl.value,
          specificity: spec,
          sourceOrder: ruleIndex,
          important: decl.important,
          inline: false,
        });
      }
    }
  });

  // 2. Parse and add inline styles
  const styleAttr = node.attributes.find(a => a.name === 'style');
  if (styleAttr) {
    const inlineDecls = parseInlineStyles(styleAttr.value);
    for (const decl of inlineDecls) {
      entries.push({
        property: decl.property,
        value: decl.value,
        specificity: { id: 0, class: 0, tag: 0 },
        sourceOrder: -1,
        important: decl.important,
        inline: true,
      });
    }
  }

  // 3. Sort by cascade priority
  entries.sort(cascadeCompare);

  // 4. Apply declarations in sorted order (last wins)
  const styles: ResolvedStyles = {};
  for (const entry of entries) {
    styles[entry.property] = entry.value;
  }

  // 5. Inherit unset inheritable properties from parent
  if (parentStyles) {
    inheritMissingProperties(styles, parentStyles);
  }

  return styles;
}
