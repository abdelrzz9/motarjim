import { create } from 'zustand';
import type { Platform, Diagnostic, CompileStats } from '../services/types';

export type EditorTab = 'html' | 'css';
export type OutputTab = 'code' | 'diagnostics' | 'ast';

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
  pipelineStage: number;
  backendOnline: boolean;
  panelRatio: number;
  activeTab: EditorTab;
  outputTab: OutputTab;
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
  setPipelineStage: (stage: number) => void;
  setBackendOnline: (online: boolean) => void;
  setPanelRatio: (ratio: number) => void;
  setActiveTab: (tab: EditorTab) => void;
  setOutputTab: (tab: OutputTab) => void;
  reset: () => void;
}

const DEFAULT_HTML = '';

const DEFAULT_CSS = '';

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
  pipelineStage: -1,
  backendOnline: true,
  panelRatio: 0.5,
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
  setPipelineStage: (pipelineStage) => set({ pipelineStage }),
  setBackendOnline: (backendOnline) => set({ backendOnline }),
  setPanelRatio: (panelRatio) => set({ panelRatio }),
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
    pipelineStage: -1,
  }),
}));
