/**
 * @deprecated Use ./ir-generate.js (generateIr) instead.
 * This module generates Flutter code from the legacy UiNode IR.
 * The IR-based generator (generateIr) consumes the three-layer IrNode
 * with separate semantic, layout, and style information.
 */

import type { UiNode, GenerateResult, PlatformTarget, Result } from '@motarjim/shared';
import { countNodes, escapeStringExtra, NodeEmitter, walkTree } from '@motarjim/generator-core';
import { DiagnosticBag } from '@motarjim/shared/diagnostics.js';

function escapeDart(s: string): string {
  return escapeStringExtra(s, { '$': '\\$' });
}

const flutterEmitter: NodeEmitter = {
  indentUnit: '  ',

  emitText(node: UiNode, _indent: string): string {
    const val = node.value ?? '';
    return `Text("${escapeDart(val)}")`;
  },

  emitButton(node: UiNode, indent: string, label: string, children: string[]): string {
    const childBlock = children.length ? `\n${children.join('\n')}\n${indent}` : '';
    const a11y = node.accessibility;
    const semantics = a11y?.label || a11y?.role
      ? `\n${indent}  semanticLabel: "${escapeDart(a11y?.label || label)}",`
      : '';
    return `ElevatedButton(\n${indent}  onPressed: () {},${semantics}\n${indent}  child: ${childBlock || `Text("${escapeDart(label)}")`},\n${indent})`;
  },

  emitRow(indent: string, children: string[]): string {
    if (!children.length) return `Row(\n${indent}  children: [],\n${indent})`;
    return `Row(\n${indent}  children: [\n${children.join(',\n')},\n${indent}  ],\n${indent})`;
  },

  emitColumn(indent: string, children: string[]): string {
    if (!children.length) return `Column(\n${indent}  children: [],\n${indent})`;
    return `Column(\n${indent}  children: [\n${children.join(',\n')},\n${indent}  ],\n${indent})`;
  },

  emitContainer(node: UiNode, indent: string, children: string[]): string {
    const props = formatProps(node.properties, indent, node.computed);
    if (!children.length) {
      return `${indent}Container(${props ? `\n${props}\n${indent}` : ''})`;
    }
    if (children.length === 1) {
      return `Container(\n${props ? `${props},\n` : ''}${indent}  child: ${children[0]},\n${indent})`;
    }
    const childrenBlock = children.join(',\n');
    return `Container(\n${props ? `${props},\n` : ''}${indent}  child: Column(\n${indent}    children: [\n${childrenBlock},\n${indent}    ],\n${indent}  ),\n${indent})`;
  },

  emitCard(node: UiNode, indent: string, children: string[]): string {
    const child = children[0] || 'SizedBox.shrink()';
    const a11y = node.accessibility;
    const semantics = a11y?.label
      ? `\n${indent}  semanticLabel: "${escapeDart(a11y.label)}",`
      : '';
    return `Card(\n${indent}  child: ${child},${semantics}\n${indent})`;
  },

  emitImage(node: UiNode, indent: string): string {
    const src = (node.properties.src as string) || '';
    const alt = (node.accessibility?.label || node.properties.alt as string || '') as string;
    const semanticLabel = alt ? `\n${indent}  semanticLabel: "${escapeDart(alt)}",` : '';
    return `Image.network("${escapeDart(src)}"${semanticLabel ? `,\n${indent}${semanticLabel}` : ''})`;
  },

  emitTextField(node: UiNode, indent: string): string {
    const a11y = node.accessibility;
    const labelText = escapeDart(a11y?.label || 'Input');
    const hintText = a11y?.hint ? `,\n${indent}    hintText: "${escapeDart(a11y.hint)}"` : '';
    return `TextField(\n${indent}  decoration: InputDecoration(\n${indent}    labelText: "${labelText}"${hintText},\n${indent}    border: OutlineInputBorder(),\n${indent}  ),\n${indent})`;
  },

  emitAppBar(indent: string, title: string): string {
    return `AppBar(\n${indent}  title: Text("${escapeDart(title)}"),\n${indent})`;
  },

  emitScrollView(indent: string, children: string[]): string {
    if (!children.length) return `ListView(\n${indent}  children: [],\n${indent})`;
    return `ListView(\n${indent}  children: [\n${children.join(',\n')},\n${indent}  ],\n${indent})`;
  },

  emitForm(node: UiNode, indent: string, children: string[]): string {
    return `Form(\n${indent}  child: Column(\n${indent}    children: [\n${children.join(',\n')},\n${indent}    ],\n${indent}  ),\n${indent})`;
  },

  emitFooter(indent: string, children: string[]): string {
    return `Container(\n${indent}  child: Column(\n${indent}    children: [\n${children.join(',\n')},\n${indent}    ],\n${indent}  ),\n${indent})`;
  },

  emitDefault(node: UiNode, indent: string, children: string[]): string {
    if (!children.length) {
      const val = node.value ?? '';
      if (val) {
        return `Text("${escapeDart(val)}")`;
      }
      return 'SizedBox.shrink()';
    }
    return `Column(\n${indent}  children: [\n${children.join(',\n')},\n${indent}  ],\n${indent})`;
  },
};

export function generate(node: UiNode, name: string = 'GeneratedView', sourceComments: boolean = false): Result<GenerateResult> {
  const bag = new DiagnosticBag();
  const start = performance.now();

  const body = walkTree(node, flutterEmitter, 0, sourceComments);
  const lines = body.split('\n');
  const indentedBody = lines
    .map((line, i) => i === 0 ? line : `    ${line}`)
    .join('\n');

  const code = `import 'package:flutter/material.dart';

class ${name} extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return ${indentedBody};
  }
}
`;

  return bag.toResult({
    code,
    metadata: {
      platform: 'flutter' as PlatformTarget,
      nodes: countNodes(node),
      duration: Math.round(performance.now() - start),
    },
  });
}

function formatProps(props: Record<string, unknown>, prefix: string, computed?: object): string {
  const lines: string[] = [];
  const i = prefix + '  ';
  const merged = { ...props, ...computed };
  for (const [key, val] of Object.entries(merged)) {
    if (key === 'value') continue;
    if (val === undefined) continue;
    lines.push(`${i}${key}: ${formatValue(val)}`);
  }
  return lines.join(',\n');
}

function formatValue(val: unknown): string {
  if (typeof val === 'string') {
    if (val.match(/^\d+px$/)) return String(parseInt(val));
    if (val.match(/^\d+$/)) return val;
    return `"${escapeDart(val)}"`;
  }
  return String(val);
}
