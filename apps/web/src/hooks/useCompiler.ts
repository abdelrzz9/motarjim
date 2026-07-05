import { useMutation } from '@tanstack/react-query';
import { compile } from '../services/wasmCompiler';
import type { CompileRequest } from '../services/types';

export function useCompiler() {
  return useMutation({
    mutationFn: (request: CompileRequest) => compile(request),
  });
}
