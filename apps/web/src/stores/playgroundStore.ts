import { create } from 'zustand';
import type { Platform, Diagnostic, CompileStats } from '../services/types';

interface PlaygroundStore {
  html: string;
  css: string;
  platform: Platform;
  minify: boolean;
  output: string;
  diagnostics: Diagnostic[];
  stats: CompileStats | null;
  ast: unknown;
  ir: unknown;
  isCompiling: boolean;
  activeTab: 'html' | 'css';
  outputTab: 'code' | 'diagnostics' | 'ast';
  setHtml: (html: string) => void;
  setCss: (css: string) => void;
  setPlatform: (platform: Platform) => void;
  setMinify: (minify: boolean) => void;
  setOutput: (output: string) => void;
  setDiagnostics: (diagnostics: Diagnostic[]) => void;
  setStats: (stats: CompileStats | null) => void;
  setAst: (ast: unknown) => void;
  setIr: (ir: unknown) => void;
  setIsCompiling: (isCompiling: boolean) => void;
  setActiveTab: (tab: 'html' | 'css') => void;
  setOutputTab: (tab: 'code' | 'diagnostics' | 'ast') => void;
  reset: () => void;
}

const DEFAULT_HTML = `<div class="container">
  <h1>Hello, Motarjim!</h1>
  <p>Start editing to see the compiled output.</p>
</div>`;

const DEFAULT_CSS = `.container {
  padding: 20px;
  font-family: system-ui, sans-serif;
}

h1 {
  color: #6366f1;
  font-size: 2rem;
}

p {
  color: #666;
  line-height: 1.6;
}`;

export const usePlaygroundStore = create<PlaygroundStore>((set) => ({
  html: DEFAULT_HTML,
  css: DEFAULT_CSS,
  platform: 'flutter',
  minify: false,
  output: '',
  diagnostics: [],
  stats: null,
  ast: null,
  ir: null,
  isCompiling: false,
  activeTab: 'html',
  outputTab: 'code',
  setHtml: (html) => set({ html }),
  setCss: (css) => set({ css }),
  setPlatform: (platform) => set({ platform }),
  setMinify: (minify) => set({ minify }),
  setOutput: (output) => set({ output }),
  setDiagnostics: (diagnostics) => set({ diagnostics }),
  setStats: (stats) => set({ stats }),
  setAst: (ast) => set({ ast }),
  setIr: (ir) => set({ ir }),
  setIsCompiling: (isCompiling) => set({ isCompiling }),
  setActiveTab: (tab) => set({ activeTab: tab }),
  setOutputTab: (tab) => set({ outputTab: tab }),
  reset: () => set({
    html: DEFAULT_HTML,
    css: DEFAULT_CSS,
    platform: 'flutter',
    minify: false,
    output: '',
    diagnostics: [],
    stats: null,
    ast: null,
    ir: null,
  }),
}));
