import type { CompileRequest, CompileResult, CompilerCacheEntry } from './types';

const CACHE_SIZE = 100;
const CACHE_TTL_MS = 5 * 60 * 1000;

const cache = new Map<string, CompilerCacheEntry>();

function hashRequest(request: CompileRequest): string {
  const key = `${request.platform}:${request.minify ?? false}:${request.html}:${request.css ?? ''}`;
  let hash = 0;
  for (let i = 0; i < key.length; i++) {
    const char = key.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash |= 0;
  }
  return `${hash}`;
}

function isExpired(entry: CompilerCacheEntry): boolean {
  return Date.now() - entry.timestamp > CACHE_TTL_MS;
}

export const compilerCache = {
  get(request: CompileRequest): CompileResult | null {
    const key = hashRequest(request);
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
    const key = hashRequest(request);
    cache.set(key, {
      result,
      timestamp: Date.now(),
      inputHash: key,
    });
  },

  invalidate(request: CompileRequest): void {
    const key = hashRequest(request);
    cache.delete(key);
  },

  clear(): void {
    cache.clear();
  },

  get size(): number {
    return cache.size;
  },
};
