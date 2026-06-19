import { describe, it, expect, vi } from 'vitest';
import { createAiDetector } from '../packages/semantic-analyzer/ai.js';
import { detectSemantics } from '../packages/semantic-analyzer/index.js';
import { parseHtml } from '../packages/parser/index.js';
import { parseCss, applyStyles } from '../packages/css-analyzer/index.js';

describe('AI Enhancement Layer', () => {
  const html = '<nav class="navbar"><h1>Title</h1></nav><div class="card"><p>Content</p></div>';
  const css = '.navbar { background: #333; } .card { padding: 16px; border-radius: 8px; box-shadow: 0 2px 4px; }';
  const ast = parseHtml(html);
  const sheet = parseCss(css);
  const styled = applyStyles(ast.children, sheet);

  it('falls back to rule-based detector when Ollama is unavailable', async () => {
    const detector = createAiDetector({ baseUrl: 'http://localhost:19999', timeout: 500 });
    const hints = await detector(styled);
    const ruleHints = detectSemantics(styled);
    expect(hints.length).toBeGreaterThanOrEqual(ruleHints.length);
  });

  it('works synchronously through the pipeline without AI flag', () => {
    const hints = detectSemantics(styled);
    expect(hints.length).toBeGreaterThan(0);
    expect(hints.some(h => h.type === 'NavigationBar')).toBe(true);
    expect(hints.some(h => h.type === 'Card')).toBe(true);
  });

  it('never generates platform code', () => {
    const hints = detectSemantics(styled);
    for (const hint of hints) {
      expect(hint.type).not.toMatch(/flutter|compose|swiftui|dart|kotlin|swift/i);
    }
  });
});
