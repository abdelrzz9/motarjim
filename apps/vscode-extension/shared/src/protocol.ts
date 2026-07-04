export enum MotarjimPlatform {
  Flutter = 'flutter',
  Compose = 'compose',
  SwiftUI = 'swiftui',
}

export enum DiagnosticCode {
  InvalidSyntax = 1001,
  UnknownElement = 1002,
  UnknownAttribute = 1003,
  InvalidAttributeValue = 1004,
  MissingRequiredAttribute = 1005,
  DeprecatedElement = 1006,
  TypeMismatch = 1007,
}

export interface MotarjimConfiguration {
  defaultPlatform: MotarjimPlatform;
  minifyOutput: boolean;
  enableDiagnostics: boolean;
  formatOnSave: boolean;
}

export interface CompileRequest {
  uri: string;
  text: string;
  platform: MotarjimPlatform;
  minify: boolean;
}

export interface CompileResponse {
  success: boolean;
  output?: string;
  error?: string;
}

export interface FormatRequest {
  uri: string;
  text: string;
}

export interface FormatResponse {
  success: boolean;
  formattedText?: string;
  error?: string;
}

export interface DiagnosticRequest {
  uri: string;
  text: string;
}

export interface DiagnosticItem {
  code: DiagnosticCode;
  message: string;
  line: number;
  column: number;
  length: number;
}

export interface DiagnosticResponse {
  diagnostics: DiagnosticItem[];
}

export const LSP_METHODS = {
  COMPILE: 'motarjim/compile',
  FORMAT: 'motarjim/format',
  DIAGNOSE: 'motarjim/diagnose',
  GET_CONFIG: 'motarjim/getConfig',
  SET_CONFIG: 'motarjim/setConfig',
  GET_PLATFORMS: 'motarjim/getPlatforms',
} as const;
