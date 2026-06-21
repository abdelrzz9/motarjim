import type {
  HtmlNode, CssStylesheet, StyledNode, SemanticHint,
  NormalizedHint, AccessibilityTree, AccessibilityInfo,
  UiNode, GenerateResult, PlatformTarget, Diagnostic,
} from '@html-native/shared';

// ── Phase System ──────────────────────────────────────────

export type PhaseId =
  | 'parse'
  | 'style'
  | 'semantic'
  | 'accessibility'
  | 'ir'
  | 'optimize'
  | 'generate';

export interface PhaseDefinition {
  id: PhaseId;
  name: string;
  description: string;
  ordinal: number;
}

export const PHASES: PhaseDefinition[] = [
  { id: 'parse',          name: 'Parse',          description: 'Parse HTML input into AST', ordinal: 0 },
  { id: 'style',          name: 'Style',          description: 'Parse CSS and apply styles to AST nodes', ordinal: 1 },
  { id: 'semantic',       name: 'Semantic',       description: 'Detect semantic components and layout intents', ordinal: 2 },
  { id: 'accessibility',  name: 'Accessibility',  description: 'Analyze accessibility tree and collect issues', ordinal: 3 },
  { id: 'ir',             name: 'IR',             description: 'Convert styled AST to platform IR', ordinal: 4 },
  { id: 'optimize',       name: 'Optimize',       description: 'Optimize IR tree (flatten, merge, prune)', ordinal: 5 },
  { id: 'generate',       name: 'Generate',       description: 'Generate platform-native code from IR', ordinal: 6 },
];

// ── Phase Output Contracts ───────────────────────────────

export interface ParseOutput {
  ast: HtmlNode;
  html: string;
  htmlNodeCount: number;
}

export interface StyleOutput {
  styledNodes: StyledNode[];
  stylesheet: CssStylesheet;
  styledNodeCount: number;
}

export interface SemanticOutput {
  hints: (SemanticHint | NormalizedHint)[];
}

export interface AccessibilityOutput {
  tree: AccessibilityTree;
  perNodeInfo: Map<string, AccessibilityInfo>;
  issueCount: number;
}

export interface IrOutput {
  ir: UiNode;
  componentCount: number;
}

export interface OptimizeOutput {
  ir: UiNode;
  savings: number;
}

export interface GenerateOutput {
  code: GenerateResult;
}

export interface PhaseOutputs {
  parse: ParseOutput;
  style: StyleOutput;
  semantic: SemanticOutput;
  accessibility: AccessibilityOutput;
  ir: IrOutput;
  optimize: OptimizeOutput;
  generate: GenerateOutput;
}

// ── Compiler Pass ────────────────────────────────────────

export interface CompilerPass<P extends PhaseId = PhaseId> {
  id: string;
  phase: P;
  name: string;
  description?: string;
  before?: string[];
  after?: string[];
  run(ctx: CompilerContext): PassResult<PhaseOutputs[P]>;
}

export interface PassResult<T> {
  ok: boolean;
  value: T;
  diagnostics: CompilerDiagnostic[];
}

// ── Compiler Diagnostic ──────────────────────────────────

export type CompilerDiagnostic = Diagnostic & {
  passId?: string;
  phaseId?: PhaseId;
  timeMs?: number;
};

// ── Compiler Context ─────────────────────────────────────

export interface CompilerOptions {
  input: string;
  css?: string;
  output?: string;
  target?: PlatformTarget;
  aiEnhance?: boolean;
  aiModel?: string;
  dryRun?: boolean;
  plugins?: CompilerPlugin[];
  /** High-level domain plugins (MotarjimPlugin) */
  motarjimPlugins?: import('./plugin-api.js').MotarjimPlugin[];
}

export interface CompilerContext {
  options: CompilerOptions;
  diagnostics: CompilerDiagnostic[];
  data: Map<string, unknown>;

  // Phase outputs — populated as pipeline executes
  parseOutput?: ParseOutput;
  styleOutput?: StyleOutput;
  semanticOutput?: SemanticOutput;
  accessibilityOutput?: AccessibilityOutput;
  irOutput?: IrOutput;
  optimizeOutput?: OptimizeOutput;
  generateOutput?: GenerateOutput;
}

// ── Plugin ───────────────────────────────────────────────

export interface CompilerPlugin {
  id: string;
  name: string;
  description?: string;
  register(pm: PassManager): void;
}

// ── Pass Manager Interface (used by plugins) ─────────────

export interface PassManager {
  register<P extends PhaseId>(pass: CompilerPass<P>): void;
  unregister(passId: string): boolean;
  hasPass(passId: string): boolean;
  getPass<P extends PhaseId>(passId: string): CompilerPass<P> | undefined;
  getPasses<P extends PhaseId>(phase: P): CompilerPass<P>[];
  getPhasePasses(phase: PhaseId): CompilerPass[];
  phases(): PhaseDefinition[];
}

// ── Pipeline Result ──────────────────────────────────────

export interface PipelineResult {
  diagnostics: CompilerDiagnostic[];
  outputs: CompilerContext;
  durationMs: number;
  stats: PipelineStats;
}

export interface PipelineStats {
  phases: PhaseStats[];
  totalDurationMs: number;
  passCount: number;
}

export interface PhaseStats {
  phaseId: PhaseId;
  name: string;
  durationMs: number;
  passCount: number;
  diagnosticCount: number;
  hasErrors: boolean;
}

// ── Error types ─────────────────────────────────────────

export class CompilerError extends Error {
  diagnostics: CompilerDiagnostic[];

  constructor(diagnostics: CompilerDiagnostic[]) {
    const msg = diagnostics.map(d => `[${d.code}] ${d.message}`).join('\n');
    super(msg);
    this.name = 'CompilerError';
    this.diagnostics = diagnostics;
  }
}
