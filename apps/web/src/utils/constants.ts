import type { Platform } from '../services/types';

export const PLATFORMS: { value: Platform; label: string; color: string }[] = [
  { value: 'flutter', label: 'Flutter', color: 'var(--flutter)' },
  { value: 'compose', label: 'Jetpack Compose', color: 'var(--compose)' },
  { value: 'swiftui', label: 'SwiftUI', color: 'var(--swiftui)' },
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
