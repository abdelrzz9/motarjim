import * as parse5 from 'parse5';
import type { HtmlNode, HtmlAttribute, Result } from '@html-native/shared';
import { DiagnosticBag } from '@html-native/shared/diagnostics.js';

let nodeCounter = 0;

function nextId(): string {
  return `node_${++nodeCounter}`;
}

function convertAttrs(attrs: any[] | undefined): HtmlAttribute[] {
  if (!attrs || !Array.isArray(attrs)) return [];
  return attrs.map((a: any) => ({ name: a.name, value: a.value ?? '' }));
}

function walkTree(node: any, bag: DiagnosticBag): HtmlNode | null {
  const tagName = node.tagName?.toLowerCase() || '';

  if (!tagName) return null;

  const htmlNode: HtmlNode = {
    nodeId: nextId(),
    tagName,
    attributes: convertAttrs(node.attrs),
    children: [],
    sourceLocation: node.sourceCodeLocation
      ? {
          line: node.sourceCodeLocation.startLine,
          col: node.sourceCodeLocation.startCol,
        }
      : undefined,
  };

  const childNodes = (node as any).childNodes || [];
  for (const child of childNodes) {
    if (child.nodeName === '#text') {
      const text = (child.value || '').trim();
      if (text) {
        htmlNode.children.push({
          nodeId: nextId(),
          tagName: '#text',
          attributes: [],
          children: [],
          value: text,
        });
      }
    } else if ((child as any).tagName) {
      const childNode = walkTree(child, bag);
      if (childNode) {
        htmlNode.children.push(childNode);
      }
    }
  }

  return htmlNode;
}

function findBody(doc: any): any {
  const docChildren = doc.childNodes || [];
  const html = docChildren.find(
    (n: any) => n.tagName?.toLowerCase() === 'html'
  );
  if (!html) return null;
  const htmlChildren = html.childNodes || [];
  return htmlChildren.find(
    (n: any) => n.tagName?.toLowerCase() === 'body'
  );
}

export function parseHtml(html: string, file: string = 'input.html'): Result<HtmlNode> {
  nodeCounter = 0;
  const bag = new DiagnosticBag();

  let document: any;
  try {
    document = parse5.parse(html);
  } catch (err) {
    bag.addError('PARSER_001', `Failed to parse HTML: ${(err as Error).message}`, 'parser');
    return bag.asResult();
  }

  const body = findBody(document);

  const children: HtmlNode[] = [];
  if (body) {
    const bodyChildren = (body as any).childNodes || [];
    for (const child of bodyChildren) {
      if (child.tagName) {
        const node = walkTree(child, bag);
        if (node) children.push(node);
      }
    }
  } else {
    bag.addWarning('PARSER_002', 'No <body> element found in HTML document', 'parser');
  }

  const root: HtmlNode = {
    nodeId: 'root',
    tagName: 'root',
    attributes: [],
    children,
  };

  return bag.toResult(root);
}

export function parseFragment(html: string, file: string = 'fragment.html'): Result<HtmlNode[]> {
  nodeCounter = 0;
  const bag = new DiagnosticBag();

  let fragment: any;
  try {
    fragment = parse5.parseFragment(html);
  } catch (err) {
    bag.addError('PARSER_003', `Failed to parse HTML fragment: ${(err as Error).message}`, 'parser');
    return bag.asResult();
  }

  const nodes: HtmlNode[] = [];

  const fragChildren = (fragment as any).childNodes || [];
  for (const child of fragChildren) {
    if (child.nodeName === '#text') {
      const text = (child.value || '').trim();
      if (text) {
        nodes.push({
          nodeId: nextId(),
          tagName: '#text',
          attributes: [],
          children: [],
          value: text,
        });
      }
    } else if ((child as any).tagName) {
      const node = walkTree(child, bag);
      if (node) nodes.push(node);
    }
  }

  return bag.toResult(nodes);
}
