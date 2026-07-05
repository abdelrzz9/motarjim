import { describe, it, expect } from 'vitest';
import {
  createDiagnostic,
  createErrorDiagnostic,
  createWarningDiagnostic,
  htmlParseError,
  cssParseError,
  missingClosingTag,
  duplicateId,
  unsupportedCssProperty,
  unsupportedHtmlElement,
  formatDiagnosticShort,
} from '../services/diagnostics';

describe('Diagnostics', () => {
  it('creates a basic diagnostic', () => {
    const d = createDiagnostic('error', 'E0001', 'Test Error', 'This is a test error');
    expect(d.severity).toBe('error');
    expect(d.code).toBe('E0001');
    expect(d.title).toBe('Test Error');
    expect(d.explanation).toBe('This is a test error');
    expect(d.suggestions).toEqual([]);
    expect(d.notes).toEqual([]);
  });

  it('creates error diagnostic', () => {
    const d = createErrorDiagnostic('E0002', 'Test', 'Desc');
    expect(d.severity).toBe('error');
  });

  it('creates warning diagnostic', () => {
    const d = createWarningDiagnostic('W0001', 'Test', 'Desc');
    expect(d.severity).toBe('warning');
  });

  it('creates HTML parse error', () => {
    const d = htmlParseError('Unexpected token', undefined, 'Check your syntax');
    expect(d.code).toBe('E0101');
    expect(d.severity).toBe('error');
    expect(d.suggestions).toContain('Check your syntax');
  });

  it('creates CSS parse error', () => {
    const d = cssParseError('Invalid property');
    expect(d.code).toBe('E0201');
    expect(d.severity).toBe('error');
  });

  it('creates missing closing tag error', () => {
    const span = { start: { line: 1, column: 1, offset: 0 }, end: { line: 1, column: 6, offset: 5 } };
    const d = missingClosingTag('div', span);
    expect(d.title).toContain('div');
    expect(d.suggestions[0]).toContain('</div>');
  });

  it('creates duplicate ID warning', () => {
    const span1 = { start: { line: 1, column: 1, offset: 0 }, end: { line: 1, column: 10, offset: 9 } };
    const span2 = { start: { line: 5, column: 1, offset: 50 }, end: { line: 5, column: 10, offset: 59 } };
    const d = duplicateId('myId', span1, span2);
    expect(d.code).toBe('W0101');
    expect(d.severity).toBe('warning');
  });

  it('creates unsupported CSS property warning', () => {
    const d = unsupportedCssProperty('-webkit-transform', 'rotate(90deg)');
    expect(d.code).toBe('W0201');
    expect(d.title).toContain('-webkit-transform');
  });

  it('creates unsupported HTML element warning', () => {
    const d = unsupportedHtmlElement('marquee');
    expect(d.code).toBe('W0102');
    expect(d.title).toContain('marquee');
  });

  it('formats diagnostic short', () => {
    const d = createDiagnostic('error', 'E0001', 'Test', 'Desc');
    const formatted = formatDiagnosticShort(d);
    expect(formatted).toContain('[ERROR]');
    expect(formatted).toContain('E0001');
  });
});
