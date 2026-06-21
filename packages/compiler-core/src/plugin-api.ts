import type { CompilerContext, CompilerDiagnostic, PhaseId } from './types.js';
import type { PassManager as PassManagerInterface } from './types.js';
import type { UiNode, SemanticRule, PlatformTarget, GenerateResult, Result } from '@html-native/shared';
import type { OptimizationPass } from './pass-types.js';

// ── Plugin Extension Points ──────────────────────────────

/**
 * A semantic rule contributed by a plugin.
 * Same shape as `SemanticRule` from `@html-native/shared` but with
 * the plugin's own metadata attached for diagnostics.
 */
export interface PluginSemanticRule extends SemanticRule {
  pluginId: string;
}

/**
 * An IR transformation runs after IR conversion and before optimization.
 * It receives the full IR tree and must return a valid IR tree.
 * To preserve determinism, transforms must not depend on external state.
 * Multiple transforms compose: output of one feeds the next.
 */
export interface IrTransform {
  id: string;
  name: string;
  description?: string;
  run(ir: UiNode, ctx: CompilerContext): IrTransformResult;
}

export interface IrTransformResult {
  ir: UiNode;
  diagnostics: CompilerDiagnostic[];
}

/**
 * A generator factory creates a platform generator.
 * The factory receives the IR tree and resolves with generated code.
 */
export interface GeneratorFactory {
  platform: PlatformTarget;
  name: string;
  generate(ir: UiNode, ctx: CompilerContext): Result<GenerateResult>;
}

/**
 * An event hook allows plugins to observe pipeline lifecycle
 * without modifying data.
 */
export type PluginEvent =
  | 'phaseStart'
  | 'phaseEnd'
  | 'passStart'
  | 'passEnd'
  | 'compileStart'
  | 'compileEnd';

export type PluginEventHandler = (event: PluginEvent, data: Record<string, unknown>) => void;

// ── Plugin API (what plugins receive) ────────────────────

/**
 * The typed API exposed to plugins during registration.
 * Plugins cannot access the PassManager directly — only these
 * focused extension methods.
 */
export interface PluginApi {
  /** Add semantic detection rules */
  registerSemanticRules(rules: SemanticRule | SemanticRule[]): void;

  /** Add an IR transformation (runs after IR conversion) */
  registerIrTransform(transform: IrTransform): void;

  /** Add a target generator */
  registerGenerator(factory: GeneratorFactory): void;

  /** Add an optimization pass */
  registerOptimizationPass(pass: OptimizationPass): void;

  /** Observe pipeline lifecycle events (read-only) */
  on(event: PluginEvent, handler: PluginEventHandler): void;

  /** Plugin-local key-value store */
  data: Map<string, unknown>;
}

// ── Plugin Definition ────────────────────────────────────

/**
 * A MotarjimPlugin defines extensions through focused lifecycle hooks.
 * Each hook receives a `PluginApi` that exposes only what the plugin
 * is allowed to do — no direct access to PassManager or pipeline internals.
 */
export interface MotarjimPlugin {
  id: string;
  name: string;
  version?: string;
  description?: string;
  /** Invoked at registration time with a limited API surface */
  register(api: PluginApi): void;
}

// ── Plugin Validation ────────────────────────────────────

export interface PluginValidation {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

export function validatePlugin(plugin: MotarjimPlugin): PluginValidation {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (!plugin.id) errors.push('Plugin must have an id');
  else if (!/^[a-z][a-z0-9-]+$/.test(plugin.id))
    errors.push(`Plugin id "${plugin.id}" must match [a-z][a-z0-9-]+`);

  if (!plugin.name) errors.push('Plugin must have a name');
  if (!plugin.register) errors.push('Plugin must implement register()');

  if (plugin.register.length > 1) {
    warnings.push(`Plugin "${plugin.id}" register() takes ${plugin.register.length} parameters; expected 1`);
  }

  return { valid: errors.length === 0, errors, warnings };
}

// ── Plugin Registry (internal) ───────────────────────────

export interface RegisteredPlugin {
  plugin: MotarjimPlugin;
  validation: PluginValidation;
  semanticRules: SemanticRule[];
  irTransforms: IrTransform[];
  generators: GeneratorFactory[];
  optimizationPasses: OptimizationPass[];
  eventHandlers: Map<PluginEvent, PluginEventHandler[]>;
}

/**
 * Internal registry that converts high-level MotarjimPlugin definitions
 * into low-level passes that the PipelineExecutor can run.
 * This is NOT exposed to plugins — only the PipelineExecutor uses it.
 */
export class PluginRegistry {
  private plugins = new Map<string, RegisteredPlugin>();

  /** All registered semantic rules (merged across plugins) */
  allSemanticRules: SemanticRule[] = [];

  /** All registered IR transforms (ordered by plugin registration) */
  allIrTransforms: IrTransform[] = [];

  /** All registered generators, keyed by platform */
  allGenerators = new Map<PlatformTarget, GeneratorFactory>();

  /** All registered optimization passes */
  allOptimizationPasses: OptimizationPass[] = [];

  /** Event handlers keyed by event name */
  private eventHandlers = new Map<PluginEvent, PluginEventHandler[]>();

  register(plugin: MotarjimPlugin): PluginValidation {
    const validation = validatePlugin(plugin);
    if (!validation.valid) {
      return validation;
    }

    if (this.plugins.has(plugin.id)) {
      return {
        valid: false,
        errors: [`Plugin "${plugin.id}" is already registered`],
        warnings: [],
      };
    }

    const entry: RegisteredPlugin = {
      plugin,
      validation,
      semanticRules: [],
      irTransforms: [],
      generators: [],
      optimizationPasses: [],
      eventHandlers: new Map(),
    };

    const api: PluginApi = {
      registerSemanticRules: (rules) => {
        const arr = Array.isArray(rules) ? rules : [rules];
        for (const rule of arr) {
          const pluginRule = { ...rule } as PluginSemanticRule;
          pluginRule.pluginId = plugin.id;
          entry.semanticRules.push(pluginRule);
          this.allSemanticRules.push(pluginRule);
        }
      },

      registerIrTransform: (transform) => {
        entry.irTransforms.push(transform);
        this.allIrTransforms.push(transform);
      },

      registerGenerator: (factory) => {
        if (this.allGenerators.has(factory.platform)) {
          validation.warnings.push(
            `Generator for "${factory.platform}" already registered by another plugin; "${plugin.id}" overrides it`,
          );
        }
        entry.generators.push(factory);
        this.allGenerators.set(factory.platform, factory);
      },

      registerOptimizationPass: (pass) => {
        entry.optimizationPasses.push(pass);
        this.allOptimizationPasses.push(pass);
      },

      on: (event, handler) => {
        if (!entry.eventHandlers.has(event)) {
          entry.eventHandlers.set(event, []);
        }
        entry.eventHandlers.get(event)!.push(handler);

        if (!this.eventHandlers.has(event)) {
          this.eventHandlers.set(event, []);
        }
        this.eventHandlers.get(event)!.push(handler);
      },

      data: new Map(),
    };

    try {
      plugin.register(api);
    } catch (err) {
      return {
        valid: false,
        errors: [`Plugin "${plugin.id}" register() threw: ${err instanceof Error ? err.message : String(err)}`],
        warnings: [],
      };
    }

    entry.validation = validation;
    this.plugins.set(plugin.id, entry);
    return validation;
  }

  has(id: string): boolean {
    return this.plugins.has(id);
  }

  get(id: string): RegisteredPlugin | undefined {
    return this.plugins.get(id);
  }

  getAll(): RegisteredPlugin[] {
    return [...this.plugins.values()];
  }

  count(): number {
    return this.plugins.size;
  }

  emit(event: PluginEvent, data: Record<string, unknown>): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(event, data);
        } catch {
          // Silently swallow observer errors — they are read-only
        }
      }
    }
  }

  clear(): void {
    this.plugins.clear();
    this.allSemanticRules = [];
    this.allIrTransforms = [];
    this.allGenerators.clear();
    this.allOptimizationPasses = [];
    this.eventHandlers.clear();
  }
}
