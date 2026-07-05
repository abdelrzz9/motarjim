import { create } from 'zustand';
import type {
  Platform, Diagnostic, CompileStats, PipelineStage,
  EditorTab, OutputTab,
} from '../services/types';

interface PlaygroundStore {
  html: string;
  css: string;
  js: string;
  platform: Platform;
  minify: boolean;
  output: string;
  diagnostics: Diagnostic[];
  stats: CompileStats | null;
  ast: unknown;
  ir: unknown;
  isCompiling: boolean;
  pipelineStage: PipelineStage;
  backendOnline: boolean;
  panelRatio: number;
  activeTab: EditorTab;
  outputTab: OutputTab;
  compileVersion: number;
  previousOutput: string;

  setHtml: (html: string) => void;
  setCss: (css: string) => void;
  setJs: (js: string) => void;
  setPlatform: (platform: Platform) => void;
  setMinify: (minify: boolean) => void;
  setOutput: (output: string) => void;
  setDiagnostics: (diagnostics: Diagnostic[]) => void;
  setStats: (stats: CompileStats | null) => void;
  setAst: (ast: unknown) => void;
  setIr: (ir: unknown) => void;
  setIsCompiling: (isCompiling: boolean) => void;
  setPipelineStage: (stage: PipelineStage) => void;
  setBackendOnline: (online: boolean) => void;
  setPanelRatio: (ratio: number) => void;
  setActiveTab: (tab: EditorTab) => void;
  setOutputTab: (tab: OutputTab) => void;
  incrementCompileVersion: () => void;
  preservePreviousOutput: () => void;
  reset: () => void;
}

const DEFAULT_HTML = '';
const DEFAULT_CSS = '';
const DEFAULT_JS = '';

export const usePlaygroundStore = create<PlaygroundStore>((set) => ({
  html: DEFAULT_HTML,
  css: DEFAULT_CSS,
  js: DEFAULT_JS,
  platform: localStorage.getItem('motarjim-platform') as Platform || 'flutter',
  minify: false,
  output: '',
  diagnostics: [],
  stats: null,
  ast: null,
  ir: null,
  isCompiling: false,
  pipelineStage: 'idle',
  backendOnline: true,
  panelRatio: 0.5,
  activeTab: 'html',
  outputTab: 'code',
  compileVersion: 0,
  previousOutput: '',

  setHtml: (html) => set({ html }),
  setCss: (css) => set({ css }),
  setJs: (js) => set({ js }),
  setPlatform: (platform) => {
    localStorage.setItem('motarjim-platform', platform);
    set({ platform });
  },
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
  incrementCompileVersion: () => set((s) => ({ compileVersion: s.compileVersion + 1 })),
  preservePreviousOutput: () => set((s) => ({
    previousOutput: s.output,
  })),

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
    pipelineStage: 'idle',
    previousOutput: '',
  }),
}));
