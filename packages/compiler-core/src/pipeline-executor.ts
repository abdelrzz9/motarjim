import type {
  PhaseId, PhaseDefinition, CompilerPass, CompilerContext,
  CompilerOptions, CompilerDiagnostic, PassResult,
  PipelineResult, PipelineStats, PhaseStats,
} from './types.js';
import { PHASES, CompilerError } from './types.js';
import { PassManager } from './pass-manager.js';
import { PluginRegistry } from './plugin-api.js';
import type { MotarjimPlugin, GeneratorFactory } from './plugin-api.js';
import type { SemanticRule, PlatformTarget } from '@html-native/shared';

export class PipelineExecutor {
  private passManager: PassManager;
  private ctx: CompilerContext;
  private phaseTimings = new Map<PhaseId, number>();
  private pluginRegistry: PluginRegistry;

  constructor(options: CompilerOptions, passManager?: PassManager) {
    this.passManager = passManager ?? new PassManager();
    this.pluginRegistry = new PluginRegistry();

    this.ctx = {
      options,
      diagnostics: [],
      data: new Map(),
    };

    this.registerLowLevelPlugins(options.plugins ?? []);
    this.registerMotarjimPlugins(options.motarjimPlugins ?? []);
    this.injectPluginPasses();
  }

  private registerLowLevelPlugins(plugins: import('./types.js').CompilerPlugin[]): void {
    for (const plugin of plugins) {
      plugin.register(this.passManager);
    }
  }

  private registerMotarjimPlugins(plugins: MotarjimPlugin[]): void {
    for (const plugin of plugins) {
      const validation = this.pluginRegistry.register(plugin);
      if (!validation.valid) {
        for (const err of validation.errors) {
          this.ctx.diagnostics.push({
            code: 'PLG_001',
            message: err,
            severity: 'error',
            phase: this.toDiagnosticPhase('parse'),
            passId: 'plugin-registry',
          });
        }
      }
      for (const warn of validation.warnings) {
        this.ctx.diagnostics.push({
          code: 'PLG_002',
          message: warn,
          severity: 'warning',
          phase: this.toDiagnosticPhase('parse'),
          passId: 'plugin-registry',
        });
      }
    }
  }

  /** Convert plugin extension points into CompilerPass objects and register them */
  private injectPluginPasses(): void {
    // IR transforms become passes in the 'ir' phase
    const irTransforms = this.pluginRegistry.allIrTransforms;
    for (let i = 0; i < irTransforms.length; i++) {
      const transform = irTransforms[i];
      this.passManager.register({
        id: `plugin:ir:${transform.id}`,
        phase: 'ir',
        name: transform.name,
        description: transform.description,
        after: ['plugin:ir'],
        run: (ctx: CompilerContext) => {
          const ir = ctx.irOutput?.ir;
          if (!ir) {
            return {
              ok: false,
              value: { ir: null as never, componentCount: 0 },
              diagnostics: [{
                code: 'PLG_003',
                message: `IR transform "${transform.id}" skipped: no IR output available`,
                severity: 'warning',
                phase: 'ir',
              }],
            };
          }
          const result = transform.run(ir, ctx);
          ctx.irOutput = { ir: result.ir, componentCount: ctx.irOutput?.componentCount ?? 0 };
          return {
            ok: !result.diagnostics.some(d => d.severity === 'error'),
            value: ctx.irOutput,
            diagnostics: result.diagnostics,
          };
        },
      });
    }

    // Optimization passes become passes in the 'optimize' phase
    const optPasses = this.pluginRegistry.allOptimizationPasses;
    for (const pass of optPasses) {
      this.passManager.register({
        id: `plugin:optimize:${pass.name}`,
        phase: 'optimize',
        name: pass.name,
        after: ['main-optimize'],
        run: (ctx: CompilerContext) => {
          const ir = ctx.optimizeOutput?.ir ?? ctx.irOutput?.ir;
          if (!ir) {
            return {
              ok: false,
              value: { ir: null as never, savings: 0 },
              diagnostics: [{
                code: 'PLG_004',
                message: `Optimization pass "${pass.name}" skipped: no IR to optimize`,
                severity: 'warning',
                phase: 'optimizer',
              }],
            };
          }
          const optimized = pass.run(ir);
          ctx.optimizeOutput = { ir: optimized, savings: ctx.optimizeOutput?.savings ?? 0 };
          return {
            ok: true,
            value: ctx.optimizeOutput,
            diagnostics: [],
          };
        },
      });
    }
  }

  /** Get a generator registered by a plugin (if any) */
  getGenerator(platform: PlatformTarget): GeneratorFactory | undefined {
    return this.pluginRegistry.allGenerators.get(platform);
  }

  /** Get all semantic rules contributed by plugins */
  getPluginSemanticRules(): SemanticRule[] {
    return this.pluginRegistry.allSemanticRules;
  }

  /** Get the plugin registry for inspection */
  get pluginRegistry_(): PluginRegistry {
    return this.pluginRegistry;
  }

  get context(): Readonly<CompilerContext> {
    return this.ctx;
  }

  get passes(): PassManager {
    return this.passManager;
  }

  // ── Public API ────────────────────────────────────────

  async compile(): Promise<PipelineResult> {
    const startTime = performance.now();

    for (const phase of PHASES) {
      const passes = this.passManager.resolveOrder(phase.id);
      if (passes.length === 0) continue;

      const phaseStart = performance.now();
      let errors = false;

      for (const pass of passes) {
        const passResult = await this.runPass(pass);
        if (!passResult.ok) {
          errors = true;
          if (this.shouldAbortOnError(phase.id)) break;
        }
      }

      this.phaseTimings.set(phase.id, performance.now() - phaseStart);
    }

    const totalDuration = performance.now() - startTime;

    return {
      diagnostics: this.ctx.diagnostics,
      outputs: this.ctx,
      durationMs: totalDuration,
      stats: this.buildStats(totalDuration),
    };
  }

  compileSync(): PipelineResult {
    const startTime = performance.now();

    for (const phase of PHASES) {
      const passes = this.passManager.resolveOrder(phase.id);
      if (passes.length === 0) continue;

      const phaseStart = performance.now();
      let errors = false;

      for (const pass of passes) {
        const passResult = this.runPassSync(pass);
        if (!passResult.ok) {
          errors = true;
          if (this.shouldAbortOnError(phase.id)) break;
        }
      }

      this.phaseTimings.set(phase.id, performance.now() - phaseStart);
    }

    const totalDuration = performance.now() - startTime;

    return {
      diagnostics: this.ctx.diagnostics,
      outputs: this.ctx,
      durationMs: totalDuration,
      stats: this.buildStats(totalDuration),
    };
  }

  // ── Phase-level skip/re-run ───────────────────────────

  /**
   * Re-run a specific phase and all subsequent phases.
   * Useful for watch mode: re-compile from a given phase.
   */
  async compileFrom(phaseId: PhaseId): Promise<PipelineResult> {
    const startIdx = PHASES.findIndex(p => p.id === phaseId);
    if (startIdx === -1) throw new Error(`Unknown phase: ${phaseId}`);

    const startTime = performance.now();

    for (let i = startIdx; i < PHASES.length; i++) {
      const phase = PHASES[i];
      const passes = this.passManager.resolveOrder(phase.id);
      if (passes.length === 0) continue;

      const phaseStart = performance.now();
      let errors = false;

      for (const pass of passes) {
        const passResult = await this.runPass(pass);
        if (!passResult.ok) {
          errors = true;
          if (this.shouldAbortOnError(phase.id)) break;
        }
      }

      this.phaseTimings.set(phase.id, performance.now() - phaseStart);
    }

    const totalDuration = performance.now() - startTime;

    return {
      diagnostics: this.ctx.diagnostics,
      outputs: this.ctx,
      durationMs: totalDuration,
      stats: this.buildStats(totalDuration),
    };
  }

  async runPhase(phaseId: PhaseId): Promise<PhaseStats> {
    const phase = PHASES.find(p => p.id === phaseId);
    if (!phase) throw new Error(`Unknown phase: ${phaseId}`);

    const passes = this.passManager.resolveOrder(phaseId);
    if (passes.length === 0) {
      return { phaseId, name: phase.name, durationMs: 0, passCount: 0, diagnosticCount: 0, hasErrors: false };
    }

    const startTime = performance.now();
    let errors = false;

    for (const pass of passes) {
      const passResult = await this.runPass(pass);
      if (!passResult.ok) {
        errors = true;
        if (this.shouldAbortOnError(phaseId)) break;
      }
    }

    const durationMs = performance.now() - startTime;
    this.phaseTimings.set(phaseId, durationMs);

    const phaseDiags = this.ctx.diagnostics.filter(d => d.phaseId === phaseId);
    return {
      phaseId,
      name: phase.name,
      durationMs,
      passCount: passes.length,
      diagnosticCount: phaseDiags.length,
      hasErrors: errors,
    };
  }

  // ── Internal pass execution ───────────────────────────

  private async runPass(pass: CompilerPass): Promise<PassResult<unknown>> {
    const passStart = performance.now();
    try {
      const result: PassResult<unknown> = await pass.run(this.ctx);
      for (const d of result.diagnostics) {
        d.passId = pass.id;
        d.phaseId = pass.phase;
        d.timeMs = performance.now() - passStart;
      }
      this.ctx.diagnostics.push(...result.diagnostics);

      if (result.ok) {
        this.storePhaseOutput(pass.phase, result);
      }
      return result;
    } catch (err) {
      const diag: CompilerDiagnostic = {
        code: 'CMP_001',
        message: `Pass "${pass.id}" threw: ${err instanceof Error ? err.message : String(err)}`,
        severity: 'error',
        phase: this.toDiagnosticPhase(pass.phase),
        passId: pass.id,
        phaseId: pass.phase,
        timeMs: performance.now() - passStart,
      };
      this.ctx.diagnostics.push(diag);
      return { ok: false, value: null, diagnostics: [diag] };
    }
  }

  private runPassSync(pass: CompilerPass): PassResult<unknown> {
    const passStart = performance.now();
    try {
      const result: PassResult<unknown> = pass.run(this.ctx) as PassResult<unknown>;
      // If the pass returned a Promise, fail — sync executor can't handle it
      if (result && typeof (result as unknown as Promise<unknown>)?.then === 'function') {
        const diag: CompilerDiagnostic = {
          code: 'CMP_002',
          message: `Pass "${pass.id}" returned a Promise but compileSync() was used`,
          severity: 'error',
          phase: this.toDiagnosticPhase(pass.phase),
          passId: pass.id,
          phaseId: pass.phase,
        };
        this.ctx.diagnostics.push(diag);
        return { ok: false, value: null, diagnostics: [diag] };
      }

      for (const d of result.diagnostics) {
        d.passId = pass.id;
        d.phaseId = pass.phase;
        d.timeMs = performance.now() - passStart;
      }
      this.ctx.diagnostics.push(...result.diagnostics);

      if (result.ok) {
        this.storePhaseOutput(pass.phase, result);
      }
      return result;
    } catch (err) {
      const diag: CompilerDiagnostic = {
        code: 'CMP_001',
        message: `Pass "${pass.id}" threw: ${err instanceof Error ? err.message : String(err)}`,
        severity: 'error',
        phase: this.toDiagnosticPhase(pass.phase),
        passId: pass.id,
        phaseId: pass.phase,
        timeMs: performance.now() - passStart,
      };
      this.ctx.diagnostics.push(diag);
      return { ok: false, value: null, diagnostics: [diag] };
    }
  }

  private shouldAbortOnError(phaseId: PhaseId): boolean {
    const critical: PhaseId[] = ['parse', 'style', 'ir', 'generate'];
    return critical.includes(phaseId);
  }

  private toDiagnosticPhase(phaseId: PhaseId): import('@html-native/shared').DiagnosticPhase {
    switch (phaseId) {
      case 'parse': return 'parser';
      case 'style': return 'css';
      case 'semantic': return 'semantic';
      case 'accessibility': return 'accessibility';
      case 'ir': return 'ir';
      case 'optimize': return 'optimizer';
      case 'generate': return 'generator';
    }
  }

  private storePhaseOutput(phaseId: PhaseId, result: PassResult<unknown>): void {
    const value = result.value as Record<string, unknown>;
    switch (phaseId) {
      case 'parse':
        this.ctx.parseOutput = value as never;
        break;
      case 'style':
        this.ctx.styleOutput = value as never;
        break;
      case 'semantic':
        this.ctx.semanticOutput = value as never;
        break;
      case 'accessibility':
        this.ctx.accessibilityOutput = value as never;
        break;
      case 'ir':
        this.ctx.irOutput = value as never;
        break;
      case 'optimize':
        this.ctx.optimizeOutput = value as never;
        break;
      case 'generate':
        this.ctx.generateOutput = value as never;
        break;
    }
  }

  // ── Stats ─────────────────────────────────────────────

  private buildStats(totalDurationMs: number): PipelineStats {
    const phases: PhaseStats[] = PHASES.map(phase => {
      const passes = this.passManager.getPhasePasses(phase.id);
      const phaseDiags = this.ctx.diagnostics.filter(d => d.phaseId === phase.id);
      return {
        phaseId: phase.id,
        name: phase.name,
        durationMs: this.phaseTimings.get(phase.id) ?? 0,
        passCount: passes.length,
        diagnosticCount: phaseDiags.length,
        hasErrors: phaseDiags.some(d => d.severity === 'error'),
      };
    });

    return {
      phases,
      totalDurationMs: totalDurationMs,
      passCount: this.passManager.count,
    };
  }

  // ── Diagnostic helpers ────────────────────────────────

  getDiagnostics(filters?: {
    phaseId?: PhaseId;
    passId?: string;
    severity?: 'error' | 'warning' | 'info';
  }): CompilerDiagnostic[] {
    let result = this.ctx.diagnostics;
    if (filters?.phaseId) result = result.filter(d => d.phaseId === filters.phaseId);
    if (filters?.passId) result = result.filter(d => d.passId === filters.passId);
    if (filters?.severity) result = result.filter(d => d.severity === filters.severity);
    return result;
  }

  hasErrors(): boolean {
    return this.ctx.diagnostics.some(d => d.severity === 'error');
  }

  // ── Reset ─────────────────────────────────────────────

  reset(): void {
    this.ctx.diagnostics = [];
    this.ctx.data = new Map();
    this.ctx.parseOutput = undefined;
    this.ctx.styleOutput = undefined;
    this.ctx.semanticOutput = undefined;
    this.ctx.accessibilityOutput = undefined;
    this.ctx.irOutput = undefined;
    this.ctx.optimizeOutput = undefined;
    this.ctx.generateOutput = undefined;
    this.phaseTimings.clear();
  }
}
