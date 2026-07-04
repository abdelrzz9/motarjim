import type { Platform } from '../services/types';

export const PLATFORMS: { value: Platform; label: string; color: string }[] = [
  { value: 'flutter', label: 'Flutter', color: '#1389fd' },
  { value: 'compose', label: 'Jetpack Compose', color: '#4285f4' },
  { value: 'swiftui', label: 'SwiftUI', color: '#f05138' },
];

export const PIPELINE_STAGES = [
  { id: 'parse', label: 'Parse', description: 'Parse HTML input' },
  { id: 'css', label: 'CSS', description: 'Process CSS rules' },
  { id: 'ir', label: 'IR', description: 'Generate intermediate representation' },
  { id: 'codegen', label: 'Codegen', description: 'Generate target code' },
];

export const SEVERITY_ICONS: Record<string, string> = {
  error: '✕',
  warning: '⚠',
  info: 'ℹ',
  hint: '💡',
  note: '→',
};

export const DEFAULT_DEBOUNCE_MS = 500;
