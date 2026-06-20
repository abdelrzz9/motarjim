import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';

import { parseHtml } from '@html-native/parser';
import { parseCss, applyStyles, analyzeLayoutIntents, buildResponsiveMetadata } from '@html-native/css-analyzer';
import { detectSemantics } from '@html-native/semantic-analyzer';
import { styledNodeToIr, enrichWithIntentSync } from '@html-native/ir';
import { optimize } from '@html-native/optimizer';
import { generate as generateFlutter } from '@html-native/generator-flutter';
import { generate as generateCompose } from '@html-native/generator-compose';
import { generate as generateSwiftUI } from '@html-native/generator-swiftui';
import type { StyledNode } from '@html-native/shared';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const COMPONENT_TYPES = new Set([
  'Button', 'Card', 'NavigationBar', 'AppBar', 'Drawer',
  'HeroSection', 'Footer', 'Sidebar', 'Dialog', 'Modal',
  'Tabs', 'Form', 'TextField', 'TextArea', 'List',
]);

function countHtmlNodes(node: any): number {
  let count = 1;
  for (const child of node.children) count += countHtmlNodes(child);
  return count;
}

function countComponentNodes(node: any): number {
  let count = COMPONENT_TYPES.has(node.type) ? 1 : 0;
  for (const child of node.children) count += countComponentNodes(child);
  return count;
}

/**
 * Runs the full motarjim pipeline directly on in-memory HTML/CSS strings
 * (no filesystem involved) and returns generated native code + stats.
 */
function convertInMemory(html: string, css: string, target: 'flutter' | 'compose' | 'swiftui') {
  const startTime = Date.now();

  const ast = parseHtml(html);
  const htmlNodes = countHtmlNodes(ast);

  const stylesheet = parseCss(css || '');
  let styledNodes: StyledNode[] = applyStyles(ast.children, stylesheet);
  styledNodes = analyzeLayoutIntents(styledNodes);
  buildResponsiveMetadata(stylesheet); // computed for parity with CLI; not attached here

  const hints = detectSemantics(styledNodes);

  const rootStyled: StyledNode = {
    node: ast,
    styles: {},
    children: styledNodes,
    layoutIntent: { type: 'Stack', properties: {}, confidence: 1 },
  };

  let ir = styledNodeToIr(rootStyled, hints);
  ir = enrichWithIntentSync(ir);

  const componentsDetected = countComponentNodes(ir);
  const optimized = optimize(ir);

  let result;
  switch (target) {
    case 'flutter':
      result = generateFlutter(optimized);
      break;
    case 'compose':
      result = generateCompose(optimized);
      break;
    case 'swiftui':
      result = generateSwiftUI(optimized);
      break;
    default:
      throw new Error(`Unknown target "${target}"`);
  }

  const duration = (Date.now() - startTime) / 1000;

  return {
    code: result.code,
    stats: {
      htmlNodes,
      componentsDetected,
      generatedLines: result.code.split('\n').length,
      target,
      duration,
    },
  };
}

const app = express();
app.use(express.json({ limit: '2mb' }));
app.use(express.static(path.join(__dirname, 'public')));

app.post('/api/convert', (req, res) => {
  const { html, css, target } = req.body ?? {};

  if (typeof html !== 'string' || !html.trim()) {
    return res.status(400).json({ error: 'Missing "html" field.' });
  }
  if (!['flutter', 'compose', 'swiftui'].includes(target)) {
    return res.status(400).json({ error: 'Target must be one of: flutter, compose, swiftui.' });
  }

  try {
    const output = convertInMemory(html, css ?? '', target);
    res.json(output);
  } catch (err: any) {
    res.status(500).json({ error: err?.message ?? 'Conversion failed.' });
  }
});

const PORT = process.env.PORT ? Number(process.env.PORT) : 3000;
app.listen(PORT, () => {
  console.log(`motarjim web UI running at http://localhost:${PORT}`);
});
