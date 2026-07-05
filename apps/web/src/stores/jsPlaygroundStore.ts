import { create } from 'zustand';

interface JsPlaygroundStore {
  code: string;
  isProcessing: boolean;
  diagnostics: string[];
  setCode: (code: string) => void;
  setIsProcessing: (processing: boolean) => void;
  setDiagnostics: (diagnostics: string[]) => void;
  clear: () => void;
}

export const useJsPlaygroundStore = create<JsPlaygroundStore>((set) => ({
  code: '',
  isProcessing: false,
  diagnostics: [],
  setCode: (code) => set({ code }),
  setIsProcessing: (isProcessing) => set({ isProcessing }),
  setDiagnostics: (diagnostics) => set({ diagnostics }),
  clear: () => set({ code: '', diagnostics: [] }),
}));
