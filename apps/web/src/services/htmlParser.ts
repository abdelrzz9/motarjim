import type { ASTNode, Diagnostic, SourceSpan, SourcePosition } from './types';
import { htmlParseError, missingClosingTag, duplicateId } from './diagnostics';
import { logger } from './logger';

const VOID_ELEMENTS = new Set([
  'area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input',
  'link', 'meta', 'param', 'source', 'track', 'wbr',
]);

const RAW_TEXT_ELEMENTS = new Set(['script', 'style', 'textarea', 'title']);

export function parseHtml(input: string): { ast: ASTNode; diagnostics: Diagnostic[] } {
  const diagnostics: Diagnostic[] = [];
  const seenIds = new Map<string, SourceSpan>();

  const root: ASTNode = {
    type: 'document',
    children: [],
  };

  if (!input.trim()) {
    return { ast: root, diagnostics };
  }

  const stack: { node: ASTNode; tagName: string; openSpan: SourceSpan }[] = [];
  let current = root;
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

  function parseTagName(): string | null {
    const start = pos;
    while (pos < input.length && /[a-zA-Z0-9_-]/.test(input[pos])) {
      pos++;
    }
    if (pos === start) return null;
    return input.slice(start, pos);
  }

  function parseAttribute(): { name: string; value: string } | null {
    skipWhitespace();
    if (pos >= input.length || /[>/]/.test(input[pos])) return null;

    const nameStart = pos;
    while (pos < input.length && /[a-zA-Z0-9_:.-]/.test(input[pos])) {
      pos++;
    }
    if (pos === nameStart) return null;
    const name = input.slice(nameStart, pos);

    skipWhitespace();
    if (pos >= input.length || input[pos] !== '=') {
      return { name, value: '' };
    }
    pos++;

    skipWhitespace();
    if (pos >= input.length) return { name, value: '' };

    let value = '';
    const quote = input[pos];
    if (quote === '"' || quote === "'") {
      pos++;
      const valStart = pos;
      while (pos < input.length && input[pos] !== quote) {
        if (input[pos] === '\\' && pos + 1 < input.length) {
          pos += 2;
        } else {
          pos++;
        }
      }
      value = input.slice(valStart, pos);
      if (pos < input.length) pos++;
    } else {
      const valStart = pos;
      while (pos < input.length && !/\s|>/.test(input[pos])) {
        pos++;
      }
      value = input.slice(valStart, pos);
    }

    return { name, value };
  }

  function parseAttributes(): Record<string, string> {
    const attrs: Record<string, string> = {};
    while (pos < input.length) {
      skipWhitespace();
      if (pos >= input.length || input[pos] === '>' || (input[pos] === '/' && pos + 1 < input.length && input[pos + 1] === '>')) {
        break;
      }
      const attr = parseAttribute();
      if (attr) {
        attrs[attr.name] = attr.value;
        if (attr.name === 'id' && attr.value) {
          const span = createSpan(pos, pos);
          if (seenIds.has(attr.value)) {
            diagnostics.push(duplicateId(attr.value, seenIds.get(attr.value)!, span));
          } else {
            seenIds.set(attr.value, span);
          }
        }
      } else {
        break;
      }
    }
    return attrs;
  }

  function parseComment(): boolean {
    if (input.startsWith('<!--', pos)) {
      const end = input.indexOf('-->', pos + 4);
      if (end >= 0) {
        pos = end + 3;
        return true;
      }
      diagnostics.push(htmlParseError('Unclosed HTML comment', createSpan(pos, pos + 4), 'Add --> to close the comment.'));
      pos = input.length;
      return true;
    }
    return false;
  }

  function parseDoctype(): boolean {
    if (input.startsWith('<!', pos) && !input.startsWith('<!--', pos)) {
      const end = input.indexOf('>', pos);
      if (end >= 0) {
        pos = end + 1;
        return true;
      }
      pos = input.length;
      return true;
    }
    return false;
  }

  function parseText(): string {
    const start = pos;
    while (pos < input.length && input[pos] !== '<') {
      pos++;
    }
    return input.slice(start, pos);
  }

  function parseRawText(tagName: string): string {
    const endTag = `</${tagName}`;
    const endIdx = input.toLowerCase().indexOf(endTag.toLowerCase(), pos);
    if (endIdx >= 0) {
      const content = input.slice(pos, endIdx);
      pos = endIdx;
      return content;
    }
    const content = input.slice(pos);
    pos = input.length;
    diagnostics.push(missingClosingTag(tagName, createSpan(pos - content.length, pos)));
    return content;
  }

  function createTextNode(text: string, startPos: number): ASTNode | null {
    if (!text.trim() && text.length === 0) return null;
    return {
      type: 'text',
      value: text,
      position: createSpan(startPos, pos),
    };
  }

  function parseElement(): boolean {
    if (pos >= input.length || input[pos] !== '<') return false;

    if (parseComment() || parseDoctype()) return true;

    const isClosing = input[pos + 1] === '/';

    if (isClosing) {
      pos += 2;
      const tagName = parseTagName();
      if (!tagName) {
        diagnostics.push(htmlParseError('Invalid closing tag', createSpan(pos - 2, pos)));
        pos++;
        return true;
      }
      skipWhitespace();
      if (pos < input.length && input[pos] === '>') pos++;

      if (stack.length > 0 && stack[stack.length - 1].tagName.toLowerCase() === tagName.toLowerCase()) {
        stack.pop();
        current = stack.length > 0 ? stack[stack.length - 1].node : root;
      } else {
        diagnostics.push(htmlParseError(
          `Unexpected closing tag </${tagName}>. Expected ${stack.length > 0 ? `</${stack[stack.length - 1].tagName}>` : 'end of document'}.`,
          createSpan(pos - tagName.length - 3, pos),
          stack.length > 0
            ? `Replace </${tagName}> with </${stack[stack.length - 1].tagName}>.`
            : `Remove extraneous </${tagName}>.`,
        ));
        if (stack.length > 0) {
          while (stack.length > 0 && stack[stack.length - 1].tagName.toLowerCase() !== tagName.toLowerCase()) {
            const el = stack.pop()!;
            diagnostics.push(missingClosingTag(el.tagName, el.openSpan));
          }
          if (stack.length > 0 && stack[stack.length - 1].tagName.toLowerCase() === tagName.toLowerCase()) {
            stack.pop();
          }
          current = stack.length > 0 ? stack[stack.length - 1].node : root;
        }
      }
      return true;
    }

    const openStart = pos;
    pos++;

    const tagName = parseTagName();
    if (!tagName) {
      pos = openStart + 1;
      return false;
    }

    const attrs = parseAttributes();
    const isSelfClosing = VOID_ELEMENTS.has(tagName.toLowerCase());

    let selfClose = false;
    if (pos < input.length && input[pos] === '/') {
      if (pos + 1 < input.length && input[pos + 1] === '>') {
        selfClose = true;
        pos += 2;
      } else {
        pos++;
      }
    } else if (pos < input.length && input[pos] === '>') {
      pos++;
    } else {
      diagnostics.push(htmlParseError('Unclosed tag', createSpan(openStart, pos)));
      return true;
    }

    const element: ASTNode = {
      type: 'element',
      tagName: tagName.toLowerCase(),
      attributes: attrs,
      children: [],
      position: createSpan(openStart, 0),
    };

    if (current) {
      if (!current.children) current.children = [];
      current.children.push(element);
    } else {
      root.children ??= [];
      root.children.push(element);
    }

    if (!isSelfClosing && !selfClose) {
      element.position = createSpan(openStart, pos);
      stack.push({ node: element, tagName: tagName.toLowerCase(), openSpan: createSpan(openStart, pos) });
      current = element;

      if (RAW_TEXT_ELEMENTS.has(tagName.toLowerCase())) {
        const textContent = parseRawText(tagName.toLowerCase());
        if (textContent) {
          const textNode: ASTNode = {
            type: 'text',
            value: textContent,
            position: createSpan(pos - textContent.length, pos),
          };
          element.children ??= [];
          element.children.push(textNode);
        }
        if (pos < input.length && input.startsWith('</', pos)) {
          pos += 2;
          parseTagName();
          skipWhitespace();
          if (pos < input.length && input[pos] === '>') pos++;
          if (stack.length > 0) {
            stack.pop();
            current = stack.length > 0 ? stack[stack.length - 1].node : root;
          }
        }
      }
    } else {
      element.position = createSpan(openStart, pos);
    }

    return true;
  }

  while (pos < input.length) {
    if (input[pos] === '<') {
      if (!parseElement()) {
        diagnostics.push(htmlParseError(
          `Invalid character sequence at position ${pos}`,
          createSpan(pos, pos + 1),
        ));
        pos++;
      }
    } else {
      const textStart = pos;
      const text = parseText();
      if (text) {
        const textNode = createTextNode(text, textStart);
        if (textNode && current) {
          if (!current.children) current.children = [];
          current.children.push(textNode);
        }
      }
    }
  }

  while (stack.length > 0) {
    const unclosed = stack.pop()!;
    diagnostics.push(missingClosingTag(unclosed.tagName, unclosed.openSpan));
  }

  logger.info('HtmlParser', `Parsed ${input.length} bytes`, {
    nodes: countNodes(root),
    diagnostics: diagnostics.length,
  });

  return { ast: root, diagnostics };
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

export function serializeAst(node: ASTNode, indent = ''): string {
  const lines: string[] = [];
  switch (node.type) {
    case 'document':
      lines.push(indent + '#document');
      if (node.children) {
        for (const child of node.children) {
          lines.push(serializeAst(child, indent + '  '));
        }
      }
      break;
    case 'element':
      lines.push(`${indent}<${node.tagName}>${node.attributes && Object.keys(node.attributes).length > 0 ? ` ${JSON.stringify(node.attributes)}` : ''}`);
      if (node.children) {
        for (const child of node.children) {
          lines.push(serializeAst(child, indent + '  '));
        }
      }
      break;
    case 'text':
      if (node.value && node.value.trim()) {
        lines.push(`${indent}"${node.value.trim().slice(0, 80)}"`);
      }
      break;
  }
  return lines.join('\n');
}
