import { useCallback, useRef, useEffect } from 'react';
import { usePlaygroundStore } from '../stores/playgroundStore';
import { compile, cancelCompilation } from '../services/wasmCompiler';
import { logger } from '../services/logger';
import type { CompileRequest, PipelineStage } from '../services/types';

interface UseCompilerReturn {
  compile: () => void;
  cancel: () => void;
  isCompiling: boolean;
}

export function useCompiler(): UseCompilerReturn {
  const store = usePlaygroundStore();
  const compileVersionRef = useRef(0);

  useEffect(() => {
    return () => {
      cancelCompilation();
    };
  }, []);

  const handleCompile = useCallback(() => {
    const state = usePlaygroundStore.getState();
    if (state.isCompiling) {
      logger.info('useCompiler', 'Compilation already in progress, ignoring');
      return;
    }

    if (!state.html.trim()) {
      state.setDiagnostics([{
        severity: 'warning',
        code: 'W0001',
        title: 'No HTML Input',
        explanation: 'Please enter HTML code to compile.',
        suggestions: ['Add HTML markup to the editor.'],
        notes: [],
      }]);
      return;
    }

    const version = compileVersionRef.current + 1;
    compileVersionRef.current = version;

    store.preservePreviousOutput();
    store.setIsCompiling(true);
    store.setDiagnostics([]);
    store.setOutput('');
    store.setPipelineStage('idle');

    const request: CompileRequest = {
      html: state.html,
      css: state.css || '',
      platform: state.platform,
      minify: state.minify,
    };

    logger.info('useCompiler', 'Starting compilation', {
      version,
      platform: request.platform,
      htmlLength: request.html.length,
      cssLength: request.css?.length || 0,
    });

    compile(request, (stage: string) => {
      const currentState = usePlaygroundStore.getState();
      if (currentState.isCompiling) {
        store.setPipelineStage(stage as PipelineStage);
      }
    })
      .then((result) => {
        const currentState = usePlaygroundStore.getState();
        if (!currentState.isCompiling) return;

        if (!result.success && result.diagnostics.length === 0) {
          result.diagnostics.push({
            severity: 'error',
            code: 'E0002',
            title: 'Compilation Error',
            explanation: 'The compilation produced no output.',
            suggestions: ['Check your input for syntax errors.'],
            notes: [],
          });
        }

        store.setOutput(result.code);
        store.setDiagnostics(result.diagnostics);
        store.setStats(result.stats);

        if (result.diagnostics.some(d => d.severity === 'error')) {
          store.setOutputTab('diagnostics');
        }
      })
      .catch((err) => {
        const currentState = usePlaygroundStore.getState();
        if (!currentState.isCompiling) return;

        if (err?.message === 'Compilation cancelled') return;

        logger.error('useCompiler', 'Compilation error', { error: String(err) });
        store.setDiagnostics([{
          severity: 'error',
          code: 'E0003',
          title: 'Unexpected Error',
          explanation: `An unexpected error occurred: ${err instanceof Error ? err.message : String(err)}`,
          suggestions: ['Please try again.', 'If the problem persists, reload the page.'],
          notes: [],
        }]);
      })
      .finally(() => {
        const currentState = usePlaygroundStore.getState();
        if (currentState.isCompiling) {
          store.setIsCompiling(false);
          store.setPipelineStage('idle');
        }
      });
  }, [store]);

  const cancel = useCallback(() => {
    cancelCompilation();
    store.setIsCompiling(false);
    store.setPipelineStage('idle');
  }, [store]);

  const isCompiling = usePlaygroundStore((s) => s.isCompiling);

  return { compile: handleCompile, cancel, isCompiling };
}
