import { describe, it, expect, beforeEach } from 'vitest';
import { compilerCache } from '../services/cache';
import type { CompileRequest, CompileResult } from '../services/types';

const makeResult = (code: string): CompileResult => ({
  success: true,
  code,
  diagnostics: [],
  stats: { nodesParsed: 0, cssRules: 0, irNodes: 0, jsNodes: 0, timeMs: 0, parseTimeMs: 0, genTimeMs: 0 },
});

describe('Compiler Cache', () => {
  beforeEach(() => {
    compilerCache.clear();
  });

  const request: CompileRequest = {
    html: '<div>test</div>',
    css: '.test { color: red; }',
    platform: 'flutter',
  };

  it('stores and retrieves results', () => {
    const result = makeResult('test code');
    compilerCache.set(request, result);
    const cached = compilerCache.get(request);
    expect(cached).toBeDefined();
    expect(cached!.code).toBe('test code');
  });

  it('returns null for uncached requests', () => {
    const cached = compilerCache.get(request);
    expect(cached).toBeNull();
  });

  it('does not cache failed results', () => {
    const failedResult: CompileResult = {
      ...makeResult(''),
      success: false,
    };
    compilerCache.set(request, failedResult);
    const cached = compilerCache.get(request);
    expect(cached).toBeNull();
  });

  it('invalidates cache entries', () => {
    compilerCache.set(request, makeResult('code'));
    compilerCache.invalidate(request);
    expect(compilerCache.get(request)).toBeNull();
  });

  it('handles different platforms as different cache keys', () => {
    compilerCache.set({ ...request, platform: 'flutter' }, makeResult('flutter'));
    compilerCache.set({ ...request, platform: 'compose' }, makeResult('compose'));

    expect(compilerCache.get({ ...request, platform: 'flutter' })!.code).toBe('flutter');
    expect(compilerCache.get({ ...request, platform: 'compose' })!.code).toBe('compose');
  });
});
