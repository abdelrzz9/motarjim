/// <reference types="vite/client" />

interface ImportMetaEnv {
  DEV: boolean;
  PROD: boolean;
  MODE: string;
}

interface Window {
  motarjim_wasm_init?: () => Promise<void>;
  WasmCompiler?: new () => unknown;
}
