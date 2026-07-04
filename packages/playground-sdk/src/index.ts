export type PlaygroundTarget = 'flutter' | 'compose' | 'swiftui';

export interface PlaygroundCompileRequest {
  html: string;
  css?: string;
  target: PlaygroundTarget;
}

export interface PlaygroundCompileResult {
  code: string;
  stats: {
    htmlNodes: number;
    componentsDetected: number;
    generatedLines: number;
    duration: number;
    warnings: number;
  };
}

export const playgroundTargets: PlaygroundTarget[] = ['flutter', 'compose', 'swiftui'];

export function runPipeline(request: PlaygroundCompileRequest): PlaygroundCompileResult {
  const started = performance.now();
  const htmlNodes = (request.html.match(/<([a-z][\w-]*)\b/gi) ?? []).length;
  const cssRules = (request.css?.match(/\{/g) ?? []).length;
  const code = renderNativePlaceholder(request.target, request.html, cssRules);

  return {
    code,
    stats: {
      htmlNodes,
      componentsDetected: Math.max(1, Math.min(htmlNodes, 12)),
      generatedLines: code.split('\n').length,
      duration: (performance.now() - started) / 1000,
      warnings: 0,
    },
  };
}

function renderNativePlaceholder(target: PlaygroundTarget, html: string, cssRules: number): string {
  const title = extractFirstText(html) || 'Generated View';
  if (target === 'compose') {
    return `import androidx.compose.material3.*\nimport androidx.compose.runtime.*\n\n@Composable\nfun GeneratedView() {\n    Column {\n        Text(text = ${JSON.stringify(title)})\n        Text(text = "CSS rules: ${cssRules}")\n    }\n}`;
  }
  if (target === 'swiftui') {
    return `import SwiftUI\n\nstruct GeneratedView: View {\n    var body: some View {\n        VStack {\n            Text(${JSON.stringify(title)})\n            Text("CSS rules: ${cssRules}")\n        }\n    }\n}`;
  }
  return `import 'package:flutter/material.dart';\n\nclass GeneratedView extends StatelessWidget {\n  const GeneratedView({super.key});\n\n  @override\n  Widget build(BuildContext context) {\n    return Column(children: [\n      Text(${JSON.stringify(title)}),\n      Text('CSS rules: ${cssRules}'),\n    ]);\n  }\n}`;
}

function extractFirstText(html: string): string {
  return html.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 80);
}
