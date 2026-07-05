import type { CompileRequest, CompileResult, CompilerCacheEntry } from './types';

const CACHE_SIZE = 100;
const CACHE_TTL_MS = 5 * 60 * 1000;

const cache = new Map<string, CompilerCacheEntry>();

function cacheKey(request: CompileRequest): string {
  return `${request.platform}:${request.minify ?? false}:${request.html.length}:${request.html}:${request.css ?? ''}`;
}

function isExpired(entry: CompilerCacheEntry): boolean {
  return Date.now() - entry.timestamp > CACHE_TTL_MS;
}

export const compilerCache = {
  get(request: CompileRequest): CompileResult | null {
    const key = cacheKey(request);
    const entry = cache.get(key);
    if (!entry) return null;
    if (isExpired(entry)) {
      cache.delete(key);
      return null;
    }
    return entry.result;
  },

  set(request: CompileRequest, result: CompileResult): void {
    if (!result.success) return;
    if (cache.size >= CACHE_SIZE) {
      const oldest = cache.entries().next().value;
      if (oldest) cache.delete(oldest[0]);
    }
    const key = cacheKey(request);
    cache.set(key, {
      result,
      timestamp: Date.now(),
      inputHash: key,
    });
  },

  invalidate(request: CompileRequest): void {
    const key = cacheKey(request);
    cache.delete(key);
  },

  clear(): void {
    cache.clear();
  },

  get size(): number {
    return cache.size;
  },
};
