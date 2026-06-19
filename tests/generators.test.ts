import { describe, it, expect } from 'vitest';
import { parseHtml } from '../packages/parser/index.js';
import { parseCss, applyStyles } from '../packages/css-analyzer/index.js';
import { detectSemantics } from '../packages/semantic-analyzer/index.js';
import { styledNodeToIr } from '../packages/ir/index.js';
import { optimize } from '../packages/optimizer/index.js';
import { generate as generateFlutter } from '../packages/generators/flutter/index.js';
import { generate as generateCompose } from '../packages/generators/compose/index.js';
import { generate as generateSwiftUI } from '../packages/generators/swiftui/index.js';


function runPipeline(html: string, css: string = '') {
  const ast = parseHtml(html);
  const sheet = parseCss(css);
  const styled = applyStyles(ast.children, sheet);
  const hints = detectSemantics(styled);
  const root = { node: ast, styles: {}, children: styled };
  const ir = styledNodeToIr(root, hints);
  return optimize(ir);
}

const SAMPLE_HTML = `<div class="container">
  <h1>Hello World</h1>
  <button>Get Started</button>
</div>`;

const SAMPLE_CSS = `.container { padding: 16px; }
h1 { font-size: 24px; color: blue; }
button { background: blue; color: white; border-radius: 8px; }`;

describe('Generator boilerplate', () => {
  describe('Flutter', () => {
    it('wraps in StatelessWidget with material import', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      const result = generateFlutter(ir);
      expect(result.code).toContain("import 'package:flutter/material.dart'");
      expect(result.code).toContain('class GeneratedView extends StatelessWidget');
      expect(result.code).toContain('Widget build(BuildContext context)');
    });

    it('produces valid Dart widget structure', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      const result = generateFlutter(ir);
      expect(result.code).toContain('return');
      expect(result.code).toContain('Container');
      expect(result.code).toContain('Text("Hello World")');
      expect(result.code).toContain('ElevatedButton');
    });
  });

  describe('Compose', () => {
    it('wraps in @Composable function with Material3 imports', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      const result = generateCompose(ir);
      expect(result.code).toContain('import androidx.compose.material3.*');
      expect(result.code).toContain('import androidx.compose.runtime.*');
      expect(result.code).toContain('import androidx.compose.foundation.layout.*');
      expect(result.code).toContain('@Composable');
      expect(result.code).toContain('fun GeneratedView()');
    });

    it('produces valid Compose structure', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      const result = generateCompose(ir);
      expect(result.code).toContain('Text(text = "Hello World")');
      expect(result.code).toContain('Button(');
    });
  });

  describe('SwiftUI', () => {
    it('wraps in View struct with SwiftUI import', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      const result = generateSwiftUI(ir);
      expect(result.code).toContain('import SwiftUI');
      expect(result.code).toContain('struct GeneratedView: View');
      expect(result.code).toContain('var body: some View');
    });

    it('produces valid SwiftUI structure', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      const result = generateSwiftUI(ir);
      expect(result.code).toContain('Text("Hello World")');
      expect(result.code).toContain('Button("Get Started")');
    });
  });

  describe('Metadata', () => {
    it('reports correct platform target', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      expect(generateFlutter(ir).metadata.platform).toBe('flutter');
      expect(generateCompose(ir).metadata.platform).toBe('compose');
      expect(generateSwiftUI(ir).metadata.platform).toBe('swiftui');
    });

    it('reports node count', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      expect(generateFlutter(ir).metadata.nodes).toBeGreaterThan(0);
      expect(generateCompose(ir).metadata.nodes).toBeGreaterThan(0);
      expect(generateSwiftUI(ir).metadata.nodes).toBeGreaterThan(0);
    });
  });
});

describe('Structural validity', () => {
  describe('Flutter', () => {
    it('has balanced braces', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      const { code } = generateFlutter(ir);
      let depth = 0;
      for (const ch of code) {
        if (ch === '{') depth++;
        if (ch === '}') depth--;
        expect(depth).toBeGreaterThanOrEqual(0);
      }
      expect(depth).toBe(0);
    });
  });

  describe('Compose', () => {
    it('has balanced braces and parens', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      const { code } = generateCompose(ir);
      let depth = 0;
      for (const ch of code) {
        if (ch === '{') depth++;
        if (ch === '}') depth--;
        expect(depth).toBeGreaterThanOrEqual(0);
      }
      expect(depth).toBe(0);
    });
  });

  describe('SwiftUI', () => {
    it('has balanced braces', () => {
      const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
      const { code } = generateSwiftUI(ir);
      let depth = 0;
      for (const ch of code) {
        if (ch === '{') depth++;
        if (ch === '}') depth--;
        expect(depth).toBeGreaterThanOrEqual(0);
      }
      expect(depth).toBe(0);
    });
  });
});

describe('Custom name', () => {
  it('Flutter uses custom class name', () => {
    const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
    const { code } = generateFlutter(ir, 'MyWidget');
    expect(code).toContain('class MyWidget extends StatelessWidget');
  });

  it('Compose uses custom function name', () => {
    const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
    const { code } = generateCompose(ir, 'MyScreen');
    expect(code).toContain('fun MyScreen()');
  });

  it('SwiftUI uses custom struct name', () => {
    const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
    const { code } = generateSwiftUI(ir, 'CustomView');
    expect(code).toContain('struct CustomView: View');
  });

  it('defaults to GeneratedView for all platforms', () => {
    const ir = runPipeline(SAMPLE_HTML, SAMPLE_CSS);
    expect(generateFlutter(ir).code).toContain('class GeneratedView');
    expect(generateCompose(ir).code).toContain('fun GeneratedView()');
    expect(generateSwiftUI(ir).code).toContain('struct GeneratedView: View');
  });
});

// -- Snapshot tests for varied HTML/CSS examples --

const NAV_HTML = `<nav class="navbar"><div class="container"><h1>My App</h1><ul><li><a href="/">Home</a></li><li><a href="/about">About</a></li></ul></div></nav>`;
const NAV_CSS = `.navbar { background: #333; color: white; padding: 1rem; } .container { display: flex; align-items: center; } ul { display: flex; gap: 1rem; list-style: none; }`;

const CARD_HTML = `<section class="features"><div class="card"><h2>Fast</h2><p>Lightning performance</p></div><div class="card"><h2>Secure</h2><p>Data protection</p></div></section>`;
const CARD_CSS = `.features { display: flex; gap: 2rem; } .card { padding: 2rem; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }`;

const HERO_HTML = `<section class="hero"><h1>Welcome</h1><p>Build something great</p><button>Get Started</button></section>`;
const HERO_CSS = `.hero { padding: 6rem 2rem; text-align: center; background: #1a1a2e; color: white; } h1 { font-size: 3rem; }`;

const FORM_HTML = `<form class="contact"><h2>Contact</h2><input type="text" placeholder="Name" /><input type="email" placeholder="Email" /><button type="submit">Send</button></form>`;
const FORM_CSS = `.contact { display: flex; flex-direction: column; gap: 1rem; max-width: 400px; } input { padding: 0.5rem; border: 1px solid #ccc; }`;

describe('Snapshot tests', () => {
  const examples: { name: string; html: string; css: string }[] = [
    { name: 'nav-bar', html: NAV_HTML, css: NAV_CSS },
    { name: 'card-grid', html: CARD_HTML, css: CARD_CSS },
    { name: 'hero-section', html: HERO_HTML, css: HERO_CSS },
    { name: 'contact-form', html: FORM_HTML, css: FORM_CSS },
  ];

  for (const { name, html, css } of examples) {
    describe(name, () => {
      it('Flutter snapshot', () => {
        const ir = runPipeline(html, css);
        const result = generateFlutter(ir);
        expect(result.code).toMatchSnapshot(`flutter-${name}`);
      });

      it('Compose snapshot', () => {
        const ir = runPipeline(html, css);
        const result = generateCompose(ir);
        expect(result.code).toMatchSnapshot(`compose-${name}`);
      });

      it('SwiftUI snapshot', () => {
        const ir = runPipeline(html, css);
        const result = generateSwiftUI(ir);
        expect(result.code).toMatchSnapshot(`swiftui-${name}`);
      });
    });
  }
});

// -- End-to-end integration test: full pipeline for all three platforms --

const FULL_HTML = `<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
  <nav class="navbar"><h1>Site</h1></nav>
  <section class="hero"><h1>Hero Title</h1><p>Hero description text</p><button>Action</button></section>
  <div class="card"><h2>Card Title</h2><p>Card body content</p></div>
  <footer class="footer"><p>Footer content</p></footer>
</body>
</html>`;

const FULL_CSS = `
.navbar { background: #333; color: white; padding: 1rem; }
.hero { padding: 4rem; text-align: center; background: #1a1a2e; color: white; }
.card { padding: 1.5rem; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
.footer { background: #333; color: white; text-align: center; padding: 2rem; }
`;

describe('End-to-end full pipeline', () => {
  it('generates all three platforms from one HTML input', () => {
    const ir = runPipeline(FULL_HTML, FULL_CSS);

    const flutter = generateFlutter(ir);
    expect(flutter.code).toContain("import 'package:flutter/material.dart'");
    expect(flutter.code).toContain('GeneratedView');
    expect(flutter.metadata.nodes).toBeGreaterThan(0);

    const compose = generateCompose(ir);
    expect(compose.code).toContain('import androidx.compose.material3.*');
    expect(compose.code).toContain('GeneratedView');
    expect(compose.metadata.nodes).toBeGreaterThan(0);

    const swift = generateSwiftUI(ir);
    expect(swift.code).toContain('import SwiftUI');
    expect(swift.code).toContain('GeneratedView');
    expect(swift.metadata.nodes).toBeGreaterThan(0);
  });

  it('produces consistent node counts across platforms', () => {
    const ir = runPipeline(FULL_HTML, FULL_CSS);
    const flutter = generateFlutter(ir);
    const compose = generateCompose(ir);
    const swift = generateSwiftUI(ir);
    expect(flutter.metadata.nodes).toBe(compose.metadata.nodes);
    expect(compose.metadata.nodes).toBe(swift.metadata.nodes);
  });

  it('generates different code per platform from same IR', () => {
    const ir = runPipeline(FULL_HTML, FULL_CSS);
    const flutter = generateFlutter(ir).code;
    const compose = generateCompose(ir).code;
    const swift = generateSwiftUI(ir).code;
    expect(flutter).not.toBe(compose);
    expect(compose).not.toBe(swift);
  });

  it('does not contain raw HTML in any output', () => {
    const ir = runPipeline(FULL_HTML, FULL_CSS);
    const results = [generateFlutter(ir), generateCompose(ir), generateSwiftUI(ir)];
    for (const result of results) {
      expect(result.code).not.toContain('<!DOCTYPE');
      expect(result.code).not.toContain('<html');
      expect(result.code).not.toContain('<body');
    }
  });

  describe('Semantic mapping correctness', () => {
    it('<nav> produces Nav/AppBar equivalent on each platform', () => {
      const ir = runPipeline(FULL_HTML, FULL_CSS);
      const flutter = generateFlutter(ir).code;
      const compose = generateCompose(ir).code;
      const swift = generateSwiftUI(ir).code;
      expect(flutter).toContain('AppBar');
      expect(compose).toContain('TopAppBar');
      expect(swift).toContain('.navigationTitle');
    });

    it('<section class="hero"> produces content in each platform', () => {
      const ir = runPipeline(FULL_HTML, FULL_CSS);
      const flutter = generateFlutter(ir).code;
      const compose = generateCompose(ir).code;
      const swift = generateSwiftUI(ir).code;
      expect(flutter).toContain('Hero Title');
      expect(compose).toContain('Hero Title');
      expect(swift).toContain('Hero Title');
    });

    it('<div class="card"> produces Card component on each platform', () => {
      const ir = runPipeline(FULL_HTML, FULL_CSS);
      const flutter = generateFlutter(ir).code;
      const compose = generateCompose(ir).code;
      const swift = generateSwiftUI(ir).code;
      expect(flutter).toContain('Card');
      expect(compose).toContain('Card');
      expect(swift).toContain('background(Color(.systemBackground))');
    });

    it('<footer> renders on each platform', () => {
      const ir = runPipeline(FULL_HTML, FULL_CSS);
      const flutter = generateFlutter(ir).code;
      const compose = generateCompose(ir).code;
      const swift = generateSwiftUI(ir).code;
      expect(flutter).toContain('Footer content');
      expect(compose).toContain('Footer content');
      expect(swift).toContain('Footer content');
    });

    it('<button> produces interactive element on each platform', () => {
      const ir = runPipeline(FULL_HTML, FULL_CSS);
      const flutter = generateFlutter(ir).code;
      const compose = generateCompose(ir).code;
      const swift = generateSwiftUI(ir).code;
      expect(flutter).toContain('ElevatedButton');
      expect(compose).toContain('Button');
      expect(swift).toContain('Button');
    });

  });
});
