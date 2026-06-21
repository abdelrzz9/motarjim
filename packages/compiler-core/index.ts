export {
  PHASES,
  CompilerError,
} from './src/types.js';

export type {
  PhaseId,
  PhaseDefinition,
  CompilerPass,
  PassResult,
  CompilerDiagnostic,
  CompilerContext,
  CompilerOptions,
  CompilerPlugin,
  PassManager as PassManagerInterface,
  PipelineResult,
  PipelineStats,
  PhaseStats,
  PhaseOutputs,
  ParseOutput,
  StyleOutput,
  SemanticOutput,
  AccessibilityOutput,
  IrOutput,
  OptimizeOutput,
  GenerateOutput,
} from './src/types.js';

export { PassManager } from './src/pass-manager.js';
export { PipelineExecutor } from './src/pipeline-executor.js';
export { definePass, createPlugin, composePlugins } from './src/plugin.js';

// Plugin system
export { PluginRegistry, validatePlugin } from './src/plugin-api.js';
export type {
  MotarjimPlugin,
  PluginApi,
  PluginSemanticRule,
  IrTransform,
  IrTransformResult,
  GeneratorFactory,
  PluginEvent,
  PluginEventHandler,
  PluginValidation,
  RegisteredPlugin,
} from './src/plugin-api.js';
export type { OptimizationPass } from './src/pass-types.js';
