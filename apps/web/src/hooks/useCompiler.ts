import { useMutation } from '@tanstack/react-query';
import { compile } from '../services/wasmCompiler';
import { usePlaygroundStore } from '../stores/playgroundStore';
import type { CompileRequest } from '../services/types';

export function useCompiler() {
  const store = usePlaygroundStore();

  const mutation = useMutation({
    mutationFn: (request: CompileRequest) => compile(request),
    onMutate: () => {
      store.setIsCompiling(true);
    },
    onSuccess: (result) => {
      store.setOutput(result.code);
      store.setDiagnostics(result.diagnostics || []);
      store.setStats(result.stats || null);
      store.setAst(result.ast || null);
      store.setIr(result.ir || null);
    },
    onError: (error: Error) => {
      store.setDiagnostics([{
        severity: 'error',
        code: 'E9999',
        message: error.message,
        suggestions: [],
        notes: [],
      }]);
    },
    onSettled: () => {
      store.setIsCompiling(false);
    },
  });

  return mutation;
}
