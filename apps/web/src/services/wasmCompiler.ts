import type { CompileRequest, CompileResult } from './types';
import { compile as pipelineCompile, cancelCurrentCompilation } from './compilerPipeline';
import { logger } from './logger';

let wasmModule: Record<string, unknown> | null = null;
let wasmAvailable: boolean | null = null;

function loadScript(url: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const existing = document.querySelector(`script[src="${url}"]`);
    if (existing) { resolve(); return; }
    const script = document.createElement('script');
    script.src = url;
    script.type = 'module';
    script.onload = () => resolve();
    script.onerror = () => reject(new Error(`Failed to load ${url}`));
    document.head.appendChild(script);
  });
}

async function loadWasm(): Promise<Record<string, unknown> | null> {
  if (wasmModule) return wasmModule;
  if (wasmAvailable === false) return null;
  try {
    await loadScript('/wasm/motarjim_wasm.js');
    if (window.motarjim_wasm_init) await window.motarjim_wasm_init();
    const WasmCtor = window.WasmCompiler;
    if (!WasmCtor) throw new Error('WasmCompiler not found');
    wasmModule = new WasmCtor() as unknown as Record<string, unknown>;
    wasmAvailable = true;
    logger.info('WasmCompiler', 'WASM module loaded successfully');
    return wasmModule;
  } catch {
    logger.warn('WasmCompiler', 'WASM not available, using TypeScript pipeline');
    wasmAvailable = false;
    return null;
  }
}

export async function compile(
  request: CompileRequest,
  onStage?: (stage: string) => void,
): Promise<CompileResult> {
  const compiler = await loadWasm();
  if (compiler) {
    try {
      const compileFn = compiler.compile as (html: string, css: string | null, platform: string) => string;
      const resultJson = compileFn(request.html, request.css || null, request.platform);
      const result = JSON.parse(resultJson) as CompileResult;

      if (result && typeof result === 'object' && 'code' in result) {
        logger.info('WasmCompiler', 'WASM compilation successful');
        return result;
      }
      throw new Error('Invalid WASM result');
    } catch (err) {
      logger.warn('WasmCompiler', 'WASM compile failed, falling back to TS pipeline', { error: String(err) });
      wasmAvailable = false;
      wasmModule = null;
    }
  }

  return pipelineCompile(request, onStage as ((stage: string) => void) | undefined);
}

export function cancelCompilation(): void {
  cancelCurrentCompilation();
}
