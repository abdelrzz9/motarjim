import { describe, it, expect } from 'vitest';
import type { ASTNode, Diagnostic, CSSDeclaration } from '../services/types';
import { generateFlutter } from '../services/codeGenerators/flutterGenerator';
import { generateCompose } from '../services/codeGenerators/composeGenerator';
import { generateSwiftUI } from '../services/codeGenerators/swiftuiGenerator';
import { parseHtml } from '../services/htmlParser';
import { parseCss } from '../services/cssParser';

function compileHtml(html: string): ASTNode {
  const { ast } = parseHtml(html);
  return ast;
}

describe('Flutter Generator', () => {
  it('generates valid Flutter for simple HTML', () => {
    const html = '<div>Hello World</div>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain("import 'package:flutter/material.dart'");
    expect(code).toContain('class MotarjimApp');
    expect(code).toContain('class MotarjimHome');
    expect(code).toContain("const Text('Hello World')");
    expect(code).not.toContain('Container('); // Empty container should be removed
    expect(diagnostics.some(d => d.severity === 'error')).toBe(false);
  });

  it('generates flex layout as Row', () => {
    const html = '<div><div>Item 1</div><div>Item 2</div></div>';
    const cssMap = new Map<string, CSSDeclaration[]>();
    cssMap.set('div', [{ property: 'display', value: 'flex', important: false }]);
    const ast = compileHtml(html);
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssMap, diagnostics, false);

    expect(code).toContain('Row(');
    expect(code).toContain("const Text('Item 1')");
    expect(code).toContain("const Text('Item 2')");
  });

  it('generates heading elements', () => {
    const html = '<h1>Title</h1><h2>Subtitle</h2>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain("'Title'");
    expect(code).toContain("'Subtitle'");
    expect(code).toContain('fontSize: 32');
    expect(code).toContain('fontSize: 24');
    expect(code).toContain('FontWeight.bold');
  });

  it('generates buttons', () => {
    const html = '<button>Click Me</button>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('ElevatedButton(');
    expect(code).toContain("'Click Me'");
    expect(code).toContain('onPressed: () {}');
  });

  it('generates text fields', () => {
    const html = '<input placeholder="Enter name">';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('TextField(');
    expect(code).toContain("'Enter name'");
  });

  it('generates images', () => {
    const html = '<img src="https://example.com/img.png">';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('Image.network(');
    expect(code).toContain("'https://example.com/img.png'");
  });

  it('generates lists', () => {
    const html = '<ul><li>Item A</li><li>Item B</li></ul>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain("const Text('• ')");
    expect(code).toContain("'Item A'");
    expect(code).toContain("'Item B'");
  });

  it('removes empty containers', () => {
    const html = '<div></div>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssDeclarations, diagnostics, false);

    expect(code).not.toContain('Container(');
  });

  it('applies CSS text color on paragraph', () => {
    const html = '<p class="red">Red text</p>';
    const cssMap = new Map<string, CSSDeclaration[]>();
    cssMap.set('.red', [{
      property: 'color',
      value: '#FF0000',
      important: false,
    }]);
    const ast = compileHtml(html);
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssMap, diagnostics, false);

    expect(code).toContain('Color(0xFFFF0000)');
  });

  it('handles CSS background-color', () => {
    const html = '<div class="box">Box</div>';
    const ast = compileHtml(html);
    const cssMap = new Map<string, CSSDeclaration[]>();
    cssMap.set('.box', [{
      property: 'background-color',
      value: '#6366F1',
      important: false,
    }]);
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssMap, diagnostics, false);

    expect(code).toContain('decoration: BoxDecoration(');
    expect(code).toContain('Color(0xFF6366F1)');
  });

  it('handles CSS padding', () => {
    const html = '<div class="padded">Padded</div>';
    const ast = compileHtml(html);
    const cssMap = new Map<string, CSSDeclaration[]>();
    cssMap.set('.padded', [{
      property: 'padding',
      value: '16px',
      important: false,
    }]);
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssMap, diagnostics, false);

    expect(code).toContain('padding: const EdgeInsets.all(16)');
  });

  it('handles CSS border-radius', () => {
    const html = '<div class="rounded">Rounded</div>';
    const ast = compileHtml(html);
    const cssMap = new Map<string, CSSDeclaration[]>();
    cssMap.set('.rounded', [{
      property: 'border-radius',
      value: '8px',
      important: false,
    }]);
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssMap, diagnostics, false);

    expect(code).toContain('BorderRadius.circular(8)');
  });

  it('does not crash with no diagnostics', () => {
    const html = '<div><p>Hello</p><span>World</span></div>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssDeclarations, diagnostics, false);

    expect(code.length).toBeGreaterThan(0);
    expect(diagnostics.some(d => d.severity === 'error')).toBe(false);
  });

  it('generates const Text for plain text', () => {
    const html = '<span>Hello</span>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateFlutter(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain("const Text('Hello')");
  });
});

describe('Compose Generator', () => {
  it('generates valid Compose for simple HTML', () => {
    const html = '<div>Hello World</div>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateCompose(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('package com.motarjim.generated');
    expect(code).toContain('class MainActivity');
    expect(code).toContain('fun MainScreen()');
    expect(code).toContain('"Hello World"');
    expect(diagnostics.some(d => d.severity === 'error')).toBe(false);
  });

  it('generates heading elements', () => {
    const html = '<h1>Title</h1><h2>Subtitle</h2>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateCompose(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('"Title"');
    expect(code).toContain('"Subtitle"');
    expect(code).toContain('fontSize = 32.sp');
    expect(code).toContain('fontSize = 24.sp');
  });

  it('generates buttons', () => {
    const html = '<button>Click Me</button>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateCompose(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('Button(');
    expect(code).toContain('onClick = { /* Button: Click Me */ }');
    expect(code).toContain('"Click Me"');
  });

  it('generates lists', () => {
    const html = '<ul><li>Item A</li><li>Item B</li></ul>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateCompose(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('"• "');
    expect(code).toContain('"Item A"');
    expect(code).toContain('"Item B"');
  });

  it('applies CSS text color on paragraph', () => {
    const html = '<p class="red">Red text</p>';
    const cssMap = new Map<string, CSSDeclaration[]>();
    cssMap.set('.red', [{
      property: 'color',
      value: '#FF0000',
      important: false,
    }]);
    const ast = compileHtml(html);
    const diagnostics: Diagnostic[] = [];
    const code = generateCompose(ast, cssMap, diagnostics, false);

    expect(code).toContain('Color(0xFFFF0000)');
  });

  it('does not crash with no diagnostics', () => {
    const html = '<div><p>Hello</p><span>World</span></div>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateCompose(ast, cssDeclarations, diagnostics, false);

    expect(code.length).toBeGreaterThan(0);
    expect(diagnostics.some(d => d.severity === 'error')).toBe(false);
  });

  it('handles flex display as Row', () => {
    const html = '<div><span>A</span><span>B</span></div>';
    const ast = compileHtml(html);
    const cssMap = new Map<string, CSSDeclaration[]>();
    cssMap.set('div', [{
      property: 'display',
      value: 'flex',
      important: false,
    }]);
    const diagnostics: Diagnostic[] = [];
    const code = generateCompose(ast, cssMap, diagnostics, false);

    expect(code).toContain('Row(');
  });
});

describe('SwiftUI Generator', () => {
  it('generates valid SwiftUI for simple HTML', () => {
    const html = '<div>Hello World</div>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateSwiftUI(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('import SwiftUI');
    expect(code).toContain('struct MotarjimApp: App');
    expect(code).toContain('struct ContentView: View');
    expect(code).toContain('"Hello World"');
    expect(diagnostics.some(d => d.severity === 'error')).toBe(false);
  });

  it('generates heading elements', () => {
    const html = '<h1>Title</h1><h2>Subtitle</h2>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateSwiftUI(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('"Title"');
    expect(code).toContain('"Subtitle"');
    expect(code).toContain('.font(.largeTitle)');
    expect(code).toContain('.font(.title)');
  });

  it('generates buttons', () => {
    const html = '<button>Click Me</button>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateSwiftUI(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('Button(action:');
    expect(code).toContain('"Click Me"');
  });

  it('generates text fields', () => {
    const html = '<input placeholder="Enter name">';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateSwiftUI(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('TextField(');
    expect(code).toContain('"Enter name"');
  });

  it('generates lists', () => {
    const html = '<ul><li>Item A</li><li>Item B</li></ul>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateSwiftUI(ast, cssDeclarations, diagnostics, false);

    expect(code).toContain('"•"');
    expect(code).toContain('"Item A"');
    expect(code).toContain('"Item B"');
  });

  it('does not crash with no diagnostics', () => {
    const html = '<div><p>Hello</p><span>World</span></div>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];
    const code = generateSwiftUI(ast, cssDeclarations, diagnostics, false);

    expect(code.length).toBeGreaterThan(0);
    expect(diagnostics.some(d => d.severity === 'error')).toBe(false);
  });

  it('applies CSS text color on paragraph', () => {
    const html = '<p class="red">Red</p>';
    const cssMap = new Map<string, CSSDeclaration[]>();
    cssMap.set('.red', [{
      property: 'color',
      value: '#FF0000',
      important: false,
    }]);
    const ast = compileHtml(html);
    const diagnostics: Diagnostic[] = [];
    const code = generateSwiftUI(ast, cssMap, diagnostics, false);

    expect(code).toContain('Color(red:');
  });
});

describe('Compiler Pipeline Integration', () => {
  it('handles empty HTML without crashing', () => {
    const html = '';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];

    expect(() => generateFlutter(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateCompose(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateSwiftUI(ast, cssDeclarations, diagnostics, false)).not.toThrow();
  });

  it('handles nested layouts without crashing', () => {
    const html = '<div><div><div><p>Deeply nested</p></div></div></div>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];

    expect(() => generateFlutter(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateCompose(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateSwiftUI(ast, cssDeclarations, diagnostics, false)).not.toThrow();
  });

  it('handles CSS with flex properties', () => {
    const html = '<div class="flex"><div>A</div><div>B</div></div>';
    const css = '.flex { display: flex; justify-content: center; align-items: center; gap: 16px; }';
    const { rules } = parseCss(css);
    const cssMap = new Map<string, CSSDeclaration[]>();
    for (const rule of rules) {
      if (rule.type === 'rule' && rule.selectors && rule.declarations) {
        for (const sel of rule.selectors) {
          cssMap.set(sel, rule.declarations);
        }
      }
    }
    const ast = compileHtml(html);
    const diagnostics: Diagnostic[] = [];

    expect(() => generateFlutter(ast, cssMap, diagnostics, false)).not.toThrow();
    expect(() => generateCompose(ast, cssMap, diagnostics, false)).not.toThrow();
    expect(() => generateSwiftUI(ast, cssMap, diagnostics, false)).not.toThrow();
  });

  it('handles forms with multiple input types', () => {
    const html = '<form><input placeholder="Name"><textarea placeholder="Message"></textarea><button>Submit</button></form>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];

    expect(() => generateFlutter(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateCompose(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateSwiftUI(ast, cssDeclarations, diagnostics, false)).not.toThrow();
  });

  it('handles tables', () => {
    const html = '<table><tr><td>A</td><td>B</td></tr></table>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];

    expect(() => generateFlutter(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateCompose(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateSwiftUI(ast, cssDeclarations, diagnostics, false)).not.toThrow();
  });

  it('handles inline elements (strong, em, code)', () => {
    const html = '<p>This is <strong>bold</strong> and <em>italic</em> and <code>code</code></p>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];

    expect(() => generateFlutter(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateCompose(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateSwiftUI(ast, cssDeclarations, diagnostics, false)).not.toThrow();
  });

  it('handles br and hr elements', () => {
    const html = '<p>Line1<br>Line2</p><hr>';
    const ast = compileHtml(html);
    const cssDeclarations = new Map<string, CSSDeclaration[]>();
    const diagnostics: Diagnostic[] = [];

    expect(() => generateFlutter(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateCompose(ast, cssDeclarations, diagnostics, false)).not.toThrow();
    expect(() => generateSwiftUI(ast, cssDeclarations, diagnostics, false)).not.toThrow();
  });

  it('full pipeline with realistic HTML+CSS generates without errors', () => {
    const html = `<div class="container">
  <h1>Welcome</h1>
  <p class="intro">This is a <strong>test</strong> page.</p>
  <div class="flex-row">
    <button>Click</button>
    <button>Cancel</button>
  </div>
  <input placeholder="Enter your name">
  <ul>
    <li>Item 1</li>
    <li>Item 2</li>
  </ul>
</div>`;
    const css = `.container { padding: 20px; background-color: #f0f0f0; }
.intro { color: #333; font-size: 16px; }
.flex-row { display: flex; gap: 12px; }`;

    const { ast } = parseHtml(html);
    const { rules } = parseCss(css);
    const cssMap = new Map<string, CSSDeclaration[]>();
    for (const rule of rules) {
      if (rule.type === 'rule' && rule.selectors && rule.declarations) {
        for (const sel of rule.selectors) {
          cssMap.set(sel, rule.declarations);
        }
      }
    }
    const diagnostics: Diagnostic[] = [];

    const flutterCode = generateFlutter(ast, cssMap, diagnostics, false);
    expect(flutterCode).toContain("'Welcome'");
    expect(flutterCode).toContain('ElevatedButton(');
    expect(flutterCode).toContain('TextField(');
    expect(diagnostics.some(d => d.severity === 'error')).toBe(false);

    const composeCode = generateCompose(ast, cssMap, [], false);
    expect(composeCode).toContain('"Welcome"');
    expect(composeCode).toContain('Button(');
    expect(composeCode).toContain('OutlinedTextField(');

    const swiftCode = generateSwiftUI(ast, cssMap, [], false);
    expect(swiftCode).toContain('"Welcome"');
    expect(swiftCode).toContain('Button(action:');
    expect(swiftCode).toContain('TextField(');
  });
});
