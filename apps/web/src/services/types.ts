export type Platform = 'flutter' | 'compose' | 'swiftui';
export type EditorTab = 'html' | 'css' | 'js';
export type OutputTab = 'code' | 'diagnostics' | 'ast';
export type Severity = 'error' | 'warning' | 'info' | 'hint' | 'note';
export type PipelineStage =
  | 'idle'
  | 'parsing_html'
  | 'parsing_css'
  | 'building_ast'
  | 'processing_javascript'
  | 'building_ir'
  | 'optimizing'
  | 'generating_code'
  | 'complete'
  | 'failed';

export interface SourcePosition {
  line: number;
  column: number;
  offset: number;
}

export interface SourceSpan {
  start: SourcePosition;
  end: SourcePosition;
}

export interface Diagnostic {
  severity: Severity;
  code: string;
  title: string;
  explanation: string;
  location?: SourceSpan;
  suggestions: string[];
  notes: string[];
}

export interface CompileRequest {
  html: string;
  css?: string;
  js?: string;
  platform: Platform;
  minify?: boolean;
}

export interface CompileStats {
  nodesParsed: number;
  cssRules: number;
  irNodes: number;
  jsNodes: number;
  timeMs: number;
  parseTimeMs: number;
  genTimeMs: number;
}

export interface CompileResult {
  success: boolean;
  code: string;
  diagnostics: Diagnostic[];
  stats: CompileStats;
  ast?: unknown;
  ir?: unknown;
}

export interface ASTNode {
  type: string;
  tagName?: string;
  attributes?: Record<string, string>;
  children?: ASTNode[];
  value?: string;
  position?: SourceSpan;
}

export interface CSSRule {
  type: 'rule' | 'comment' | 'font-face' | 'media' | 'keyframes';
  selectors?: string[];
  declarations?: CSSDeclaration[];
  name?: string;
  value?: string;
  rules?: CSSRule[];
  position?: SourceSpan;
}

export interface CSSDeclaration {
  property: string;
  value: string;
  important: boolean;
  position?: SourceSpan;
}

export interface JSNode {
  type: string;
  name?: string;
  value?: unknown;
  body?: JSNode[];
  params?: JSNode[];
  declarations?: JSNode[];
  arguments?: JSNode[];
  callee?: JSNode;
  object?: JSNode;
  property?: JSNode;
  elements?: JSNode[];
  expression?: JSNode;
  consequent?: JSNode;
  alternate?: JSNode;
  test?: JSNode;
  init?: JSNode;
  update?: JSNode;
  left?: JSNode;
  right?: JSNode;
  operator?: string;
  kind?: string;
  raw?: string;
  position?: SourceSpan;
}

export interface IRNode {
  id: string;
  type: string;
  properties: Record<string, unknown>;
  children: IRNode[];
  style?: Record<string, string>;
  events?: Record<string, string>;
  position?: SourceSpan;
}

export type CompileCallback = (stage: PipelineStage) => void;

export interface LogEntry {
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  module: string;
  message: string;
  data?: unknown;
  durationMs?: number;
}

export interface CompilerCacheEntry {
  result: CompileResult;
  timestamp: number;
  inputHash: string;
}
