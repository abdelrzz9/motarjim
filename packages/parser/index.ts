import * as parse5 from 'parse5';
import { HtmlNode, HtmlAttribute } from '@html-native/shared';

let nodeCounter = 0;

const SUPPORTED_TAGS = new Set([
  'div', 'span', 'p', 'img', 'button', 'input', 'textarea', 'form',
  'ul', 'ol', 'li', 'section', 'article', 'header', 'footer', 'nav', 'a', 'svg',
]);

function nextId(): string {
  return `node_${++nodeCounter}`;
}

function convertAttrs(attrs: any[] | undefined): HtmlAttribute[] {
  if (!attrs || !Array.isArray(attrs)) return [];
  return attrs.map((a: any) => ({ name: a.name, value: a.value ?? '' }));
}

function walkTree(node: any): HtmlNode | null {
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
      const childNode = walkTree(child);
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

export function parseHtml(html: string): HtmlNode {
  nodeCounter = 0;
  const document = parse5.parse(html);
  const body = findBody(document);

  const children: HtmlNode[] = [];
  if (body) {
    const bodyChildren = (body as any).childNodes || [];
    for (const child of bodyChildren) {
      if (child.tagName) {
        const node = walkTree(child);
        if (node) children.push(node);
      }
    }
  }

  return {
    nodeId: 'root',
    tagName: 'root',
    attributes: [],
    children,
  };
}

export function parseFragment(html: string): HtmlNode[] {
  nodeCounter = 0;
  const fragment = parse5.parseFragment(html);
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
      const node = walkTree(child);
      if (node) nodes.push(node);
    }
  }

  return nodes;
}
