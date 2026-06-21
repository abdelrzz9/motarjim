import type { HtmlNode, SelectorSimple, SelectorCompound, SelectorRelation, SelectorAst, Combinator, Specificity } from '@html-native/shared';

// -- Parser --

function isIdentStart(c: string): boolean {
  return /[a-zA-Z_\-]/.test(c);
}

function isIdent(c: string): boolean {
  return /[a-zA-Z0-9_\-]/.test(c);
}

function skipWhitespace(s: string, i: number): number {
  while (i < s.length && (s[i] === ' ' || s[i] === '\t' || s[i] === '\n')) i++;
  return i;
}

function readIdent(s: string, i: number): number {
  while (i < s.length && isIdent(s[i])) i++;
  return i;
}

function readString(s: string, i: number): number {
  const quote = s[i];
  if (quote !== '"' && quote !== "'") return i;
  i++;
  while (i < s.length && s[i] !== quote) i++;
  if (i < s.length) i++;
  return i;
}

function parseAttribute(s: string, i: number): { attr: SelectorSimple; end: number } {
  i++;
  i = skipWhitespace(s, i);
  const nameStart = i;
  i = readIdent(s, i);
  const name = s.slice(nameStart, i);
  i = skipWhitespace(s, i);
  let operator: string | undefined;
  let compareValue: string | undefined;
  if (i < s.length && '~|^$*'.includes(s[i])) {
    const opStart = i;
    if (s[i + 1] === '=') {
      operator = s.slice(i, i + 2);
      i += 2;
    } else {
      i++;
    }
    operator = s.slice(opStart, i);
    i = skipWhitespace(s, i);
    if (s[i] === '"' || s[i] === "'") {
      const valStart = i;
      i = readString(s, i);
      compareValue = s.slice(valStart + 1, i - 1);
    } else {
      const valStart = i;
      while (i < s.length && s[i] !== ']' && !isIdent(s[i])) i++;
      if (i < s.length) {
        i = readIdent(s, i);
        compareValue = s.slice(valStart, i);
      }
    }
  }
  i = skipWhitespace(s, i);
  if (s[i] === ']') i++;
  return {
    attr: { type: 'attribute', value: name, operator, compareValue },
    end: i,
  };
}

function parsePseudo(s: string, i: number): { pseudo: SelectorSimple; end: number } {
  i++;
  if (s[i] === ':') i++;
  const nameStart = i;
  i = readIdent(s, i);
  const name = s.slice(nameStart, i);
  if (i < s.length && s[i] === '(') {
    let depth = 1;
    i++;
    while (i < s.length && depth > 0) {
      if (s[i] === '(') depth++;
      else if (s[i] === ')') depth--;
      i++;
    }
  }
  return {
    pseudo: { type: 'pseudo', value: name },
    end: i,
  };
}

function parseSimple(s: string, i: number): { simple: SelectorSimple; end: number } | null {
  if (i >= s.length) return null;
  const c = s[i];
  if (c === '*') {
    return { simple: { type: 'universal', value: '*' }, end: i + 1 };
  }
  if (c === '.') {
    const end = readIdent(s, i + 1);
    return { simple: { type: 'class', value: s.slice(i + 1, end) }, end };
  }
  if (c === '#') {
    const end = readIdent(s, i + 1);
    return { simple: { type: 'id', value: s.slice(i + 1, end) }, end };
  }
  if (c === '[') {
    return parseAttribute(s, i);
  }
  if (c === ':') {
    return parsePseudo(s, i);
  }
  if (isIdentStart(c)) {
    const end = readIdent(s, i);
    return { simple: { type: 'tag', value: s.slice(i, end) }, end };
  }
  return null;
}

function parseCompound(s: string, i: number): { compound: SelectorCompound; end: number } | null {
  const simples: SelectorSimple[] = [];
  let pos = i;
  while (pos < s.length) {
    const result = parseSimple(s, pos);
    if (!result) break;
    simples.push(result.simple);
    pos = result.end;
  }
  if (simples.length === 0) return null;
  return { compound: { simples }, end: pos };
}

function parseComplex(s: string, i: number): { ast: SelectorAst; end: number } | null {
  i = skipWhitespace(s, i);
  const leftResult = parseCompound(s, i);
  if (!leftResult) return null;
  let pos = leftResult.end;
  let current: SelectorAst = leftResult.compound;

  while (pos < s.length) {
    pos = skipWhitespace(s, pos);
    if (pos >= s.length) break;

    let combinator: Combinator = 'descendant';
    if (s[pos] === '>') {
      combinator = 'child';
      pos++;
    } else if (s[pos] === '+') {
      combinator = 'adjacent-sibling';
      pos++;
    } else if (s[pos] === '~') {
      combinator = 'general-sibling';
      pos++;
    } else if (s[pos] === ',') {
      break;
    }

    pos = skipWhitespace(s, pos);
    const rightResult = parseCompound(s, pos);
    if (!rightResult) break;

    current = { left: current, combinator, right: rightResult.compound };
    pos = rightResult.end;
  }

  return { ast: current, end: pos };
}

export function parseSelector(s: string): SelectorAst | null {
  const trimmed = s.trim();
  if (!trimmed) return null;
  const result = parseComplex(trimmed, 0);
  if (!result) return null;
  return result.ast;
}

export function parseSelectorList(s: string): SelectorAst[] {
  const parts = s.split(',').map(p => p.trim()).filter(Boolean);
  const results: SelectorAst[] = [];
  for (const part of parts) {
    const ast = parseSelector(part);
    if (ast) results.push(ast);
  }
  return results;
}

// -- Specificity --

export function calculateSpecificity(ast: SelectorAst): Specificity {
  const spec: Specificity = { id: 0, class: 0, tag: 0 };

  function walk(a: SelectorAst): void {
    if ('simples' in a) {
      for (const s of a.simples) {
        if (s.type === 'id') spec.id++;
        else if (s.type === 'class' || s.type === 'attribute' || s.type === 'pseudo') spec.class++;
        else if (s.type === 'tag') spec.tag++;
      }
    } else {
      walk((a as SelectorRelation).right);
      walk((a as SelectorRelation).left);
    }
  }

  walk(ast);
  return spec;
}

export function specificityToString(spec: Specificity): string {
  return `(${spec.id},${spec.class},${spec.tag})`;
}

// -- Matching --

export type ParentResolver = (nodeId: string) => HtmlNode | null;

function matchSimple(simple: SelectorSimple, node: HtmlNode): boolean {
  switch (simple.type) {
    case 'universal':
      return true;
    case 'tag':
      return simple.value === node.tagName;
    case 'class': {
      const classAttr = node.attributes.find(a => a.name === 'class');
      return classAttr?.value.split(/\s+/).includes(simple.value) || false;
    }
    case 'id': {
      const idAttr = node.attributes.find(a => a.name === 'id');
      return idAttr?.value === simple.value || false;
    }
    case 'attribute': {
      const attr = node.attributes.find(a => a.name === simple.value);
      if (!attr) return false;
      if (!simple.operator) return true;
      switch (simple.operator) {
        case '=': return attr.value === simple.compareValue;
        case '~=': return attr.value.split(/\s+/).includes(simple.compareValue!);
        case '|=': return attr.value === simple.compareValue || attr.value.startsWith(simple.compareValue! + '-');
        case '^=': return attr.value.startsWith(simple.compareValue!);
        case '$=': return attr.value.endsWith(simple.compareValue!);
        case '*=': return attr.value.includes(simple.compareValue!);
        default: return false;
      }
    }
    case 'pseudo':
      return true;
    default:
      return false;
  }
}

export function matchCompound(compound: SelectorCompound, node: HtmlNode): boolean {
  for (const simple of compound.simples) {
    if (!matchSimple(simple, node)) return false;
  }
  return true;
}

function findAncestor(node: HtmlNode, condition: (n: HtmlNode) => boolean, getParent: ParentResolver): boolean {
  let current = getParent(node.nodeId);
  while (current) {
    if (condition(current)) return true;
    current = getParent(current.nodeId);
  }
  return false;
}

export function matchAst(
  ast: SelectorAst,
  node: HtmlNode,
  getParent: ParentResolver,
): boolean {
  if ('simples' in ast) {
    return matchCompound(ast, node);
  }

  const relation = ast as SelectorRelation;

  if (!matchCompound(relation.right, node)) return false;

  switch (relation.combinator) {
    case 'descendant':
      return findAncestor(node, (n) => matchAst(relation.left, n, getParent), getParent);
    case 'child': {
      const parent = getParent(node.nodeId);
      return parent !== null && matchAst(relation.left, parent, getParent);
    }
    case 'adjacent-sibling': {
      const parent = getParent(node.nodeId);
      if (!parent) return false;
      const idx = parent.children.findIndex(c => c.nodeId === node.nodeId);
      if (idx <= 0) return false;
      return matchAst(relation.left, parent.children[idx - 1], getParent);
    }
    case 'general-sibling': {
      const parent = getParent(node.nodeId);
      if (!parent) return false;
      const idx = parent.children.findIndex(c => c.nodeId === node.nodeId);
      for (let i = idx - 1; i >= 0; i--) {
        if (matchAst(relation.left, parent.children[i], getParent)) return true;
      }
      return false;
    }
    default:
      return false;
  }
}

// -- Legacy string-based matcher (for backward compat) --

export function matchSelectorString(selector: string, node: HtmlNode, getParent: ParentResolver = () => null): boolean {
  const ast = parseSelector(selector);
  if (!ast) return false;
  return matchAst(ast, node, getParent);
}
