import type { CSSRule, CSSDeclaration, Diagnostic, SourceSpan, SourcePosition } from './types';
import { cssParseError, unsupportedCssProperty } from './diagnostics';
import { logger } from './logger';

const SUPPORTED_CSS_PROPERTIES = new Set([
  'display', 'width', 'height', 'min-width', 'min-height', 'max-width', 'max-height',
  'margin', 'margin-top', 'margin-right', 'margin-bottom', 'margin-left',
  'padding', 'padding-top', 'padding-right', 'padding-bottom', 'padding-left',
  'color', 'background-color', 'background', 'background-image', 'background-size',
  'font-size', 'font-family', 'font-weight', 'font-style', 'line-height', 'text-align',
  'border', 'border-width', 'border-style', 'border-color',
  'border-top', 'border-right', 'border-bottom', 'border-left',
  'border-radius', 'border-top-left-radius', 'border-top-right-radius',
  'border-bottom-left-radius', 'border-bottom-right-radius',
  'box-shadow', 'opacity',
  'position', 'top', 'right', 'bottom', 'left',
  'flex', 'flex-direction', 'flex-wrap', 'flex-grow', 'flex-shrink', 'flex-basis',
  'justify-content', 'align-items', 'align-content', 'align-self', 'gap', 'row-gap', 'column-gap',
  'grid-template-columns', 'grid-template-rows', 'grid-column', 'grid-row', 'grid-area',
  'overflow', 'overflow-x', 'overflow-y',
  'z-index',
  'transform', 'transition', 'transition-property', 'transition-duration', 'transition-timing-function',
  'cursor', 'visibility', 'white-space', 'word-break',
  'list-style', 'text-decoration', 'text-transform', 'letter-spacing', 'word-spacing',
  'outline', 'outline-width', 'outline-style', 'outline-color', 'outline-offset',
  'box-sizing', 'border-collapse', 'border-spacing',
  'vertical-align', 'float', 'clear',
]);

export function parseCss(input: string): { rules: CSSRule[]; diagnostics: Diagnostic[] } {
  const diagnostics: Diagnostic[] = [];

  if (!input.trim()) {
    return { rules: [], diagnostics };
  }

  const rules: CSSRule[] = [];
  let pos = 0;

  function createPosition(offset: number): SourcePosition {
    const line = input.slice(0, offset).split('\n').length;
    const lastNewline = input.lastIndexOf('\n', offset - 1);
    const column = lastNewline >= 0 ? offset - lastNewline : offset + 1;
    return { line, column, offset };
  }

  function createSpan(start: number, end: number): SourceSpan {
    return { start: createPosition(start), end: createPosition(end) };
  }

  function skipWhitespace() {
    while (pos < input.length && /\s/.test(input[pos])) {
      pos++;
    }
  }

  function skipComment(): boolean {
    if (input.startsWith('/*', pos)) {
      const end = input.indexOf('*/', pos + 2);
      if (end >= 0) {
        const comment = input.slice(pos + 2, end);
        pos = end + 2;
        rules.push({
          type: 'comment',
          value: comment.trim(),
          position: createSpan(pos, pos),
        });
        return true;
      }
      diagnostics.push(cssParseError('Unclosed CSS comment', createSpan(pos, pos + 2)));
      pos = input.length;
      return true;
    }
    return false;
  }

  function parseIdent(): string | null {
    const start = pos;
    while (pos < input.length && /[a-zA-Z0-9_\-\\.*#:[\]>~+$()|='"]/.test(input[pos])) {
      pos++;
    }
    if (pos === start) return null;
    return input.slice(start, pos);
  }

  function parseSelectors(): string[] {
    const selectors: string[] = [];
    let current = '';
    let inParens = 0;
    let inString = false;
    let stringChar = '';

    while (pos < input.length) {
      const ch = input[pos];

      if (inString) {
        current += ch;
        if (ch === stringChar && input[pos - 1] !== '\\') {
          inString = false;
        }
        pos++;
        continue;
      }

      if (ch === '"' || ch === "'") {
        inString = true;
        stringChar = ch;
        current += ch;
        pos++;
        continue;
      }

      if (ch === '(') {
        inParens++;
        current += ch;
        pos++;
        continue;
      }

      if (ch === ')') {
        inParens--;
        current += ch;
        pos++;
        continue;
      }

      if (ch === ',' && inParens === 0) {
        const trimmed = current.trim();
        if (trimmed) selectors.push(trimmed);
        current = '';
        pos++;
        continue;
      }

      if (ch === '{' && inParens === 0) {
        break;
      }

      current += ch;
      pos++;
    }

    const trimmed = current.trim();
    if (trimmed) selectors.push(trimmed);
    if (pos < input.length && input[pos] === '{') pos++;
    return selectors;
  }

  function parseDeclaration(): CSSDeclaration | null {
    skipWhitespace();
    if (pos >= input.length || input[pos] === '}' || input[pos] === ';') {
      if (pos < input.length && input[pos] === ';') pos++;
      return null;
    }

    const propStart = pos;
    while (pos < input.length && input[pos] !== ':') {
      pos++;
    }

    if (pos >= input.length) return null;
    const property = input.slice(propStart, pos).trim();
    if (!property) {
      pos++;
      return null;
    }
    pos++;

    skipWhitespace();
    const valStart = pos;
    let inParens = 0;
    let inSingleString = false;
    let inDoubleString = false;

    while (pos < input.length) {
      const ch = input[pos];
      if (inSingleString) {
        if (ch === "'" && input[pos - 1] !== '\\') inSingleString = false;
      } else if (inDoubleString) {
        if (ch === '"' && input[pos - 1] !== '\\') inDoubleString = false;
      } else {
        if (ch === "'") inSingleString = true;
        else if (ch === '"') inDoubleString = true;
        else if (ch === '(') inParens++;
        else if (ch === ')') inParens--;
        else if (ch === ';' && inParens === 0) break;
        else if (ch === '}' && inParens === 0) break;
      }
      pos++;
    }

    let value = input.slice(valStart, pos).trim();
    let important = false;

    if (value.endsWith('!important')) {
      important = true;
      value = value.slice(0, -10).trim();
    }

    if (pos < input.length && input[pos] === ';') pos++;

    const start = createPosition(propStart);
    const end = createPosition(pos);
    const decl: CSSDeclaration = {
      property,
      value,
      important,
      position: { start, end },
    };

    if (!SUPPORTED_CSS_PROPERTIES.has(property)) {
      diagnostics.push(unsupportedCssProperty(property, value, decl.position));
    }

    return decl;
  }

  function parseBlock(): CSSRule[] {
    const innerRules: CSSRule[] = [];
    const declarations: CSSDeclaration[] = [];

    while (pos < input.length) {
      skipWhitespace();
      if (pos >= input.length) break;

      if (input[pos] === '}') {
        pos++;
        break;
      }

      if (skipComment()) continue;

      if (input[pos] === '@') {
        const atRule = parseAtRule();
        if (atRule) innerRules.push(atRule);
        continue;
      }

      const decl = parseDeclaration();
      if (decl) {
        declarations.push(decl);
      } else {
        pos++;
      }
    }

    return declarations.length > 0
      ? [{ type: 'rule', declarations }]
      : innerRules;
  }

  function parseAtRule(): CSSRule | null {
    const start = pos;
    pos++;
    const name = parseIdent();
    if (!name) {
      pos = start + 1;
      return null;
    }

    if (name === 'media') {
      const mediaStart = pos;
      while (pos < input.length && input[pos] !== '{') pos++;
      if (pos >= input.length) {
        diagnostics.push(cssParseError('Unclosed @media rule', createSpan(start, pos)));
        return null;
      }
      pos++;
      const mediaQuery = input.slice(mediaStart, pos - 1).trim();

      const rule: CSSRule = {
        type: 'media',
        name: mediaQuery,
        rules: [],
      };

      while (pos < input.length) {
        skipWhitespace();
        if (pos >= input.length) break;
        if (input[pos] === '}') { pos++; break; }
        if (skipComment()) continue;

        if (input[pos] === '@') {
          const nested = parseAtRule();
          if (nested && rule.rules) rule.rules.push(nested);
          continue;
        }

        const selectors = parseSelectors();
        const blockRules = parseBlock();
        for (const br of blockRules) {
          if (br.type === 'rule') {
            rule.rules?.push({
              type: 'rule',
              selectors,
              declarations: br.declarations,
            });
          }
        }
      }

      return rule;
    }

    if (name === 'keyframes' || name === '-webkit-keyframes') {
      skipWhitespace();
      const animNameStart = pos;
      while (pos < input.length && input[pos] !== '{') pos++;
      const animName = input.slice(animNameStart, pos).trim();
      if (pos < input.length && input[pos] === '{') pos++;

      const rule: CSSRule = {
        type: 'keyframes',
        name: animName,
        rules: [],
      };

      while (pos < input.length) {
        skipWhitespace();
        if (pos >= input.length) break;
        if (input[pos] === '}') { pos++; break; }
        if (skipComment()) continue;

        const keyframeStart = pos;
        while (pos < input.length && input[pos] !== '{') pos++;
        const keyframeSel = input.slice(keyframeStart, pos).trim();
        if (pos < input.length && input[pos] === '{') pos++;

        const keyDecls: CSSDeclaration[] = [];
        while (pos < input.length) {
          skipWhitespace();
          if (input[pos] === '}') { pos++; break; }
          if (skipComment()) continue;
          const decl = parseDeclaration();
          if (decl) keyDecls.push(decl);
        }

        if (keyDecls.length > 0) {
          rule.rules?.push({
            type: 'rule',
            selectors: [keyframeSel],
            declarations: keyDecls,
          });
        }
      }

      return rule;
    }

    if (name === 'font-face') {
      const rule: CSSRule = { type: 'font-face', declarations: [] };
      if (pos < input.length && input[pos] === '{') pos++;
      while (pos < input.length) {
        skipWhitespace();
        if (input[pos] === '}') { pos++; break; }
        if (skipComment()) continue;
        const decl = parseDeclaration();
        if (decl && rule.declarations) rule.declarations.push(decl);
      }
      return rule;
    }

    while (pos < input.length && input[pos] !== '{' && input[pos] !== ';') pos++;
    if (pos < input.length && input[pos] === '{') {
      pos++;
      let depth = 1;
      while (pos < input.length && depth > 0) {
        if (input[pos] === '{') depth++;
        if (input[pos] === '}') depth--;
        pos++;
      }
    } else if (pos < input.length && input[pos] === ';') {
      pos++;
    }

    return null;
  }

  while (pos < input.length) {
    skipWhitespace();
    if (pos >= input.length) break;

    if (skipComment()) continue;

    if (input[pos] === '@') {
      const atRule = parseAtRule();
      if (atRule) rules.push(atRule);
      continue;
    }

    const selectors = parseSelectors();
    const blockRules = parseBlock();

    for (const br of blockRules) {
      if (br.type === 'rule') {
        rules.push({
          type: 'rule',
          selectors,
          declarations: br.declarations,
        });
      }
    }
  }

  logger.info('CssParser', `Parsed CSS with ${rules.length} rules`, {
    rules: rules.length,
    diagnostics: diagnostics.length,
  });

  return { rules, diagnostics };
}

export function findMatchingRules(
  rules: CSSRule[],
  tagName: string,
  classNames: string[],
  id?: string,
): CSSDeclaration[] {
  const declarations: CSSDeclaration[] = [];
  const seenProperties = new Set<string>();

  function processRule(rule: CSSRule) {
    if (rule.type !== 'rule' || !rule.selectors || !rule.declarations) return;

    for (const selector of rule.selectors) {
      if (matchesSelector(selector, tagName, classNames, id)) {
        for (const decl of rule.declarations) {
          if (!seenProperties.has(decl.property)) {
            seenProperties.add(decl.property);
            declarations.push(decl);
          }
        }
        break;
      }
    }
  }

  for (const rule of rules) {
    if (rule.type === 'media') {
      if (rule.rules) {
        for (const sub of rule.rules) processRule(sub);
      }
    } else {
      processRule(rule);
    }
  }

  return declarations;
}

function matchesSelector(
  selector: string,
  tagName: string,
  classNames: string[],
  id?: string,
): boolean {
  const parts = selector.split(/\s+/).filter(Boolean);
  if (parts.length === 0) return false;

  const last = parts[parts.length - 1];

  if (last === '*') return true;

  const tagMatch = last.startsWith(tagName) || !/^[a-zA-Z]/.test(last);
  const classMatch = classNames.length === 0 || last.includes('.')
    ? classNames.some(c => last.includes(`.${c}`)) || last.includes('.')
    : true;
  const idMatch = !id || !last.includes('#') || last.includes(`#${id}`);

  if (tagMatch && classMatch && idMatch) return true;

  for (const cls of classNames) {
    const simpleSel = last.includes('.') ? last.split('.')[1] : last;
    if (simpleSel === cls) return true;
  }

  if (id && last === `#${id}`) return true;
  if (tagName && last === tagName) return true;

  return false;
}
