/// <reference types="vite/client" />

interface Window {
  motarjim_wasm_init?: () => Promise<void>;
  WasmCompiler?: new () => { compile(html: string, css: string | null, platform: string): string };
}
