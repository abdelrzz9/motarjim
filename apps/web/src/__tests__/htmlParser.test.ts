import { describe, it, expect } from 'vitest';
import { parseHtml } from '../services/htmlParser';

describe('HTML Parser', () => {
  it('parses a simple element', () => {
    const { ast, diagnostics } = parseHtml('<div>Hello</div>');
    expect(diagnostics).toHaveLength(0);
    expect(ast.type).toBe('document');
    expect(ast.children).toHaveLength(1);
    expect(ast.children![0].tagName).toBe('div');
    expect(ast.children![0].children![0].value).toBe('Hello');
  });

  it('parses nested elements', () => {
    const { ast, diagnostics } = parseHtml('<div><span>text</span></div>');
    expect(diagnostics).toHaveLength(0);
    expect(ast.children![0].tagName).toBe('div');
    expect(ast.children![0].children![0].tagName).toBe('span');
  });

  it('parses attributes', () => {
    const { ast, diagnostics } = parseHtml('<div class="container" id="main">text</div>');
    expect(diagnostics).toHaveLength(0);
    expect(ast.children![0].attributes).toEqual({ class: 'container', id: 'main' });
  });

  it('parses self-closing elements', () => {
    const { ast, diagnostics } = parseHtml('<div><br><hr><img src="test.png"></div>');
    expect(diagnostics).toHaveLength(0);
    const div = ast.children![0];
    const children = div.children!.filter(c => c.type === 'element');
    expect(children.map(c => c.tagName)).toEqual(['br', 'hr', 'img']);
  });

  it('detects missing closing tags', () => {
    const { diagnostics } = parseHtml('<div><span>text</div>');
    expect(diagnostics.length).toBeGreaterThan(0);
    expect(diagnostics[0].severity).toBe('error');
    const hasMissing = diagnostics.some(d => d.code === 'E0102');
    const hasMismatch = diagnostics.some(d => d.code === 'E0101');
    expect(hasMissing || hasMismatch).toBe(true);
  });

  it('detects mismatched closing tags', () => {
    const { diagnostics } = parseHtml('<div><span>text</span></span></div>');
    expect(diagnostics.length).toBeGreaterThan(0);
  });

  it('detects duplicate IDs', () => {
    const { diagnostics } = parseHtml('<div id="x"></div><span id="x"></span>');
    const dups = diagnostics.filter(d => d.code === 'W0101');
    expect(dups.length).toBe(1);
  });

  it('handles empty input', () => {
    const { ast, diagnostics } = parseHtml('');
    expect(diagnostics).toHaveLength(0);
    expect(ast.type).toBe('document');
  });

  it('handles comments', () => {
    const { ast, diagnostics } = parseHtml('<!-- comment --><div>text</div>');
    expect(diagnostics).toHaveLength(0);
    expect(ast.children).toHaveLength(1);
  });

  it('handles unclosed comment', () => {
    const { diagnostics } = parseHtml('<!-- unclosed');
    expect(diagnostics.length).toBeGreaterThan(0);
  });

  it('extracts text nodes', () => {
    const { ast } = parseHtml('<p>Hello <b>world</b>!</p>');
    const texts = ast.children![0].children!.filter(c => c.type === 'text');
    expect(texts.length).toBeGreaterThan(0);
    expect(texts[0].value).toBe('Hello ');
  });

  it('handles void elements correctly', () => {
    const { ast, diagnostics } = parseHtml('<input type="text"><br><hr>');
    expect(diagnostics).toHaveLength(0);
    const elements = ast.children!.filter(c => c.type === 'element');
    expect(elements.map(e => e.tagName)).toEqual(['input', 'br', 'hr']);
  });

  it('handles raw text elements (script, style)', () => {
    const { ast } = parseHtml('<script>const x = 1 < 2;</script><style>.a { color: red; }</style>');
    expect(ast.children).toHaveLength(2);
    const texts = ast.children!.map(c => c.children?.[0]?.value || '');
    expect(texts[0]).toBe('const x = 1 < 2;');
    expect(texts[1]).toBe('.a { color: red; }');
  });

  it('handles unicode and special characters', () => {
    const { ast, diagnostics } = parseHtml('<p>Hello 世界 ñ ñ</p>');
    expect(diagnostics).toHaveLength(0);
    expect(ast.children![0].children![0].value).toBe('Hello 世界 ñ ñ');
  });
});
