import { describe, it, expect } from 'vitest';
import { parseCss, findMatchingRules } from '../services/cssParser';

describe('CSS Parser', () => {
  it('parses simple rules', () => {
    const { rules, diagnostics } = parseCss('.foo { color: red; }');
    expect(diagnostics).toHaveLength(0);
    expect(rules).toHaveLength(1);
    expect(rules[0].type).toBe('rule');
    expect(rules[0].selectors).toEqual(['.foo']);
    expect(rules[0].declarations).toHaveLength(1);
    expect(rules[0].declarations![0].property).toBe('color');
    expect(rules[0].declarations![0].value).toBe('red');
  });

  it('parses multiple declarations', () => {
    const { rules, diagnostics } = parseCss('div { color: red; font-size: 16px; margin: 10px; }');
    expect(diagnostics).toHaveLength(0);
    expect(rules[0].declarations).toHaveLength(3);
  });

  it('parses multiple selectors', () => {
    const { rules } = parseCss('h1, h2, h3 { color: blue; }');
    expect(rules[0].selectors).toEqual(['h1', 'h2', 'h3']);
  });

  it('handles empty input', () => {
    const { rules, diagnostics } = parseCss('');
    expect(rules).toHaveLength(0);
    expect(diagnostics).toHaveLength(0);
  });

  it('reports unsupported CSS properties', () => {
    const { diagnostics } = parseCss('div { some-unknown-prop: value; }');
    const unsupported = diagnostics.filter(d => d.code === 'W0201');
    expect(unsupported.length).toBeGreaterThan(0);
  });

  it('parses @media rules', () => {
    const { rules } = parseCss('@media (max-width: 600px) { .foo { color: red; } }');
    expect(rules.length).toBeGreaterThan(0);
    const mediaRule = rules.find(r => r.type === 'media');
    expect(mediaRule).toBeDefined();
    expect(mediaRule!.rules).toBeDefined();
    expect(mediaRule!.rules!.length).toBeGreaterThan(0);
  });

  it('parses @keyframes', () => {
    const { rules } = parseCss('@keyframes slide { from { left: 0; } to { left: 100px; } }');
    const kf = rules.find(r => r.type === 'keyframes');
    expect(kf).toBeDefined();
    expect(kf!.name).toBe('slide');
  });

  it('parses @font-face', () => {
    const { rules } = parseCss('@font-face { font-family: "MyFont"; src: url("font.woff"); }');
    const ff = rules.find(r => r.type === 'font-face');
    expect(ff).toBeDefined();
  });

  it('handles comments', () => {
    const { rules, diagnostics } = parseCss('/* comment */ .foo { color: red; } /* another */');
    expect(diagnostics).toHaveLength(0);
    const realRules = rules.filter(r => r.type === 'rule');
    expect(realRules).toHaveLength(1);
  });

  it('handles !important', () => {
    const { rules } = parseCss('div { color: red !important; }');
    expect(rules[0].declarations![0].important).toBe(true);
    expect(rules[0].declarations![0].value).toBe('red');
  });

  it('matches selectors correctly', () => {
    const { rules } = parseCss('.foo { color: red; } div { font-size: 14px; } #bar { margin: 0; }');

    const matches1 = findMatchingRules(rules, 'div', ['foo'], undefined);
    expect(matches1.length).toBeGreaterThan(0);

    const matches2 = findMatchingRules(rules, 'div', []);
    expect(matches2.some(d => d.property === 'font-size')).toBe(true);
  });
});
