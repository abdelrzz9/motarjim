import { describe, it, expect } from 'vitest';
import { parseHtml, parseFragment } from '../packages/parser/index.js';

describe('Parser', () => {
  it('parses a simple div', () => {
    const ast = parseHtml('<div>Hello</div>');
    expect(ast.tagName).toBe('root');
    expect(ast.children.length).toBe(1);
    expect(ast.children[0].tagName).toBe('div');
  });

  it('parses nested elements', () => {
    const ast = parseHtml('<div><span>text</span></div>');
    const div = ast.children[0];
    expect(div.tagName).toBe('div');
    expect(div.children[0].tagName).toBe('span');
  });

  it('parses attributes', () => {
    const ast = parseHtml('<button class="btn" id="submit">Click</button>');
    const btn = ast.children[0];
    const classAttr = btn.attributes.find(a => a.name === 'class');
    const idAttr = btn.attributes.find(a => a.name === 'id');
    expect(classAttr?.value).toBe('btn');
    expect(idAttr?.value).toBe('submit');
  });

  it('parses multiple children', () => {
    const ast = parseHtml('<ul><li>A</li><li>B</li><li>C</li></ul>');
    const ul = ast.children[0];
    expect(ul.tagName).toBe('ul');
    expect(ul.children.length).toBe(3);
  });

  it('handles empty input', () => {
    const ast = parseHtml('');
    expect(ast.tagName).toBe('root');
    expect(ast.children.length).toBe(0);
  });

  it('parses all supported tags', () => {
    const tags = ['div', 'span', 'p', 'img', 'button', 'input', 'textarea', 'form',
                   'ul', 'ol', 'li', 'section', 'article', 'header', 'footer', 'nav', 'a', 'svg'];
    const html = tags.map(t => `<${t}></${t}>`).join('\n');
    const ast = parseHtml(`<div>${html}</div>`);
    const container = ast.children[0];
    for (const tag of tags) {
      expect(container.children.some(c => c.tagName === tag)).toBe(true);
    }
  });

  describe('text handling', () => {
    it('stores text content in node.value for #text nodes', () => {
      const ast = parseHtml('<p>Hello World</p>');
      const p = ast.children[0];
      const textNode = p.children[0];
      expect(textNode.tagName).toBe('#text');
      expect(textNode.value).toBe('Hello World');
    });

    it('does not create empty text nodes for whitespace', () => {
      const ast = parseHtml('<div><span>A</span><span>B</span></div>');
      const div = ast.children[0];
      expect(div.children.length).toBe(2);
      for (const child of div.children) {
        expect(child.tagName).not.toBe('#text');
      }
    });

    it('preserves text in deeply nested structures', () => {
      const ast = parseHtml('<section><article><h1>Title</h1><p>Body text</p></article></section>');
      const section = ast.children[0];
      const article = section.children[0];
      expect(article.children[0].tagName).toBe('h1');
      expect(article.children[0].children[0].value).toBe('Title');
      expect(article.children[1].tagName).toBe('p');
      expect(article.children[1].children[0].value).toBe('Body text');
    });

    it('handles mixed content (elements + text)', () => {
      const ast = parseHtml('<p>Hello <strong>World</strong></p>');
      const p = ast.children[0];
      expect(p.children.length).toBeGreaterThanOrEqual(2);
      expect(p.children[0].tagName).toBe('#text');
      expect(p.children[0].value).toBe('Hello');
    });
  });

  describe('parseFragment', () => {
    it('parses an HTML fragment', () => {
      const nodes = parseFragment('<div>hello</div><span>world</span>');
      expect(nodes.length).toBe(2);
      expect(nodes[0].tagName).toBe('div');
      expect(nodes[1].tagName).toBe('span');
    });

    it('handles text fragments', () => {
      const nodes = parseFragment('just text');
      expect(nodes.length).toBe(1);
      expect(nodes[0].tagName).toBe('#text');
      expect(nodes[0].value).toBe('just text');
    });
  });

  describe('self-closing tags', () => {
    it('parses img tag without children', () => {
      const ast = parseHtml('<img src="test.png" alt="Test" />');
      expect(ast.children.length).toBe(1);
      expect(ast.children[0].tagName).toBe('img');
      expect(ast.children[0].children.length).toBe(0);
    });

    it('parses input tag without children', () => {
      const ast = parseHtml('<input type="text" placeholder="Name" />');
      expect(ast.children[0].tagName).toBe('input');
      expect(ast.children[0].children.length).toBe(0);
    });
  });
});
