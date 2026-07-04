import type { CompileRequest, CompileResult } from './types';

let wasmModule: Record<string, unknown> | null = null;

async function loadWasm(): Promise<Record<string, unknown>> {
  if (wasmModule) return wasmModule;
  try {
    const mod = await import('/wasm/motarjim_wasm.js') as Record<string, unknown>;
    const init = mod.default as (() => Promise<void>) | undefined;
    if (init) {
      await init();
    }
    const WasmCompiler = mod.WasmCompiler as new () => Record<string, unknown>;
    wasmModule = new WasmCompiler();
    return wasmModule;
  } catch (error) {
    console.error('Failed to load WASM compiler:', error);
    throw new Error('WASM compiler failed to load');
  }
}

export async function compile(request: CompileRequest): Promise<CompileResult> {
  const compiler = await loadWasm();
  const compileFn = compiler.compile as (html: string, css: string | null, platform: string) => string;
  const result = JSON.parse(
    compileFn(request.html, request.css || null, request.platform)
  ) as CompileResult;
  return result;
}
