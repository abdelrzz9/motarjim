export type Platform = 'flutter' | 'compose' | 'swiftui';

export interface CompileRequest {
  html: string;
  css?: string;
  platform: Platform;
  minify?: boolean;
  strict?: boolean;
}

export interface CompileResult {
  success: boolean;
  code: string;
  diagnostics: Diagnostic[];
  stats: CompileStats;
  ast?: unknown;
  ir?: unknown;
}

export interface Diagnostic {
  severity: 'error' | 'warning' | 'info' | 'hint' | 'note';
  code: string;
  message: string;
  span?: SourceSpan;
  suggestions: string[];
  notes: string[];
}

export interface SourceSpan {
  start: { line: number; column: number; offset: number };
  end: { line: number; column: number; offset: number };
}

export interface CompileStats {
  nodes_parsed: number;
  css_rules: number;
  ir_nodes: number;
  time_ms: number;
}

export interface PlaygroundState {
  html: string;
  css: string;
  platform: Platform;
  minify: boolean;
  output: string;
  diagnostics: Diagnostic[];
  stats: CompileStats | null;
  ast: unknown;
  ir: unknown;
}
