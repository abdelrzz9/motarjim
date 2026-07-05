import type { Diagnostic, Severity, SourceSpan } from './types';

let diagnosticCounter = 0;

export function createDiagnostic(
  severity: Severity,
  code: string,
  title: string,
  explanation: string,
  location?: SourceSpan,
  suggestions?: string[],
  notes?: string[],
): Diagnostic {
  diagnosticCounter++;
  return {
    severity,
    code: code || `D${String(diagnosticCounter).padStart(4, '0')}`,
    title,
    explanation,
    location,
    suggestions: suggestions || [],
    notes: notes || [],
  };
}

export function createErrorDiagnostic(
  code: string,
  title: string,
  explanation: string,
  location?: SourceSpan,
  suggestions?: string[],
  notes?: string[],
): Diagnostic {
  return createDiagnostic('error', code, title, explanation, location, suggestions, notes);
}

export function createWarningDiagnostic(
  code: string,
  title: string,
  explanation: string,
  location?: SourceSpan,
  suggestions?: string[],
  notes?: string[],
): Diagnostic {
  return createDiagnostic('warning', code, title, explanation, location, suggestions, notes);
}

export function createInfoDiagnostic(
  code: string,
  title: string,
  explanation: string,
  location?: SourceSpan,
): Diagnostic {
  return createDiagnostic('info', code, title, explanation, location);
}

export function createHintDiagnostic(
  title: string,
  explanation: string,
  location?: SourceSpan,
): Diagnostic {
  return createDiagnostic('hint', 'H0001', title, explanation, location);
}

export function htmlParseError(
  message: string,
  location?: SourceSpan,
  suggestion?: string,
): Diagnostic {
  return createErrorDiagnostic(
    'E0101',
    'HTML Parse Error',
    message,
    location,
    suggestion ? [suggestion] : undefined,
  );
}

export function cssParseError(
  message: string,
  location?: SourceSpan,
  suggestion?: string,
): Diagnostic {
  return createErrorDiagnostic(
    'E0201',
    'CSS Parse Error',
    message,
    location,
    suggestion ? [suggestion] : undefined,
  );
}

export function missingClosingTag(
  tagName: string,
  openingLocation: SourceSpan,
): Diagnostic {
  return createErrorDiagnostic(
    'E0102',
    `Missing closing tag for <${tagName}>`,
    `The <${tagName}> element starting at line ${openingLocation.start.line} was never closed.`,
    openingLocation,
    [`Add </${tagName}> at the appropriate location.`],
  );
}

export function duplicateId(
  id: string,
  firstLocation: SourceSpan,
  secondLocation: SourceSpan,
): Diagnostic {
  return createWarningDiagnostic(
    'W0101',
    'Duplicate ID',
    `The ID "${id}" is used multiple times in the same document.`,
    secondLocation,
    [`Remove or rename the duplicate ID "${id}".`],
    [`First occurrence at line ${firstLocation.start.line}, column ${firstLocation.start.column}.`],
  );
}

export function unsupportedCssProperty(
  property: string,
  value: string,
  location?: SourceSpan,
): Diagnostic {
  return createWarningDiagnostic(
    'W0201',
    `Unsupported CSS property: ${property}`,
    `The CSS property "${property}: ${value}" is not supported for native UI generation.`,
    location,
    [`Remove or replace "${property}" with a supported alternative.`],
  );
}

export function unsupportedHtmlElement(
  tagName: string,
  location?: SourceSpan,
): Diagnostic {
  return createWarningDiagnostic(
    'W0102',
    `Unsupported HTML element: <${tagName}>`,
    `The <${tagName}> element is not fully supported for native UI generation.`,
    location,
    [`Replace <${tagName}> with a supported element or use a div with appropriate styling.`],
  );
}

export function unsupportedJsApi(
  apiName: string,
  location?: SourceSpan,
): Diagnostic {
  return createWarningDiagnostic(
    'W0301',
    `Unsupported browser API: ${apiName}`,
    `The browser API "${apiName}" is not available in native UI frameworks.`,
    location,
    [`Replace "${apiName}" with a native equivalent or wrap in a platform check.`],
  );
}

export function jsParseError(
  message: string,
  location?: SourceSpan,
  suggestion?: string,
): Diagnostic {
  return createErrorDiagnostic(
    'E0301',
    'JavaScript Parse Error',
    message,
    location,
    suggestion ? [suggestion] : undefined,
  );
}

export function stringToSeverity(s: string): Severity {
  if (s === 'error' || s === 'warning' || s === 'info' || s === 'hint' || s === 'note') {
    return s;
  }
  return 'info';
}

export function formatDiagnosticShort(d: Diagnostic): string {
  const loc = d.location
    ? ` [${d.location.start.line}:${d.location.start.column}]`
    : '';
  return `[${d.severity.toUpperCase()}] ${d.code}${loc}: ${d.title}`;
}
