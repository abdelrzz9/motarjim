import { UiNode, GenerateResult, PlatformTarget } from '@html-native/shared';
import { countNodes, escapeStringExtra, NodeEmitter, walkTree } from '@html-native/generator-core';

// Flutter generator: wraps widget trees in a StatelessWidget with material.dart imports.

function escapeDart(s: string): string {
  return escapeStringExtra(s, { '$': '\\$' });
}

const flutterEmitter: NodeEmitter = {
  indentUnit: '  ',

  emitText(node: UiNode, _indent: string): string {
    const val = node.value ?? '';
    return `Text("${escapeDart(val)}")`;
  },

  emitButton(indent: string, label: string, children: string[]): string {
    const childBlock = children.length ? `\n${children.join('\n')}\n${indent}` : '';
    return `ElevatedButton(\n${indent}  onPressed: () {},\n${indent}  child: ${childBlock || `Text("${escapeDart(label)}")`},\n${indent})`;
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
    const props = formatProps(node.properties, indent);
    if (!children.length) {
      return `${indent}Container(${props ? `\n${props}\n${indent}` : ''})`;
    }
    if (children.length === 1) {
      return `Container(\n${props ? `${props},\n` : ''}${indent}  child: ${children[0]},\n${indent})`;
    }
    const childrenBlock = children.join(',\n');
    return `Container(\n${props ? `${props},\n` : ''}${indent}  child: Column(\n${indent}    children: [\n${childrenBlock},\n${indent}    ],\n${indent}  ),\n${indent})`;
  },

  emitCard(indent: string, children: string[]): string {
    const child = children[0] || 'SizedBox.shrink()';
    return `Card(\n${indent}  child: ${child},\n${indent})`;
  },

  emitImage(node: UiNode, indent: string): string {
    const src = (node.properties.src as string) || '';
    return `Image.network("${escapeDart(src)}")`;
  },

  emitTextField(indent: string): string {
    return `TextField(\n${indent}  decoration: InputDecoration(\n${indent}    border: OutlineInputBorder(),\n${indent}  ),\n${indent})`;
  },

  emitAppBar(indent: string, title: string): string {
    return `AppBar(\n${indent}  title: Text("${escapeDart(title)}"),\n${indent})`;
  },

  emitScrollView(indent: string, children: string[]): string {
    if (!children.length) return `ListView(\n${indent}  children: [],\n${indent})`;
    return `ListView(\n${indent}  children: [\n${children.join(',\n')},\n${indent}  ],\n${indent})`;
  },

  emitForm(indent: string, children: string[]): string {
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

export function generate(node: UiNode, name: string = 'GeneratedView'): GenerateResult {
  const start = performance.now();

  const body = walkTree(node, flutterEmitter, 0);
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

  return {
    code,
    metadata: {
      platform: 'flutter' as PlatformTarget,
      nodes: countNodes(node),
      duration: Math.round(performance.now() - start),
    },
  };
}

function formatProps(props: Record<string, unknown>, prefix: string): string {
  const lines: string[] = [];
  const i = prefix + '  ';
  for (const [key, val] of Object.entries(props)) {
    if (key === 'value') continue;
    lines.push(`${i}${camelToSnake(key)}: ${formatValue(val)}`);
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

function camelToSnake(s: string): string {
  return s.replace(/[A-Z]/g, c => `_${c.toLowerCase()}`);
}

export { generateNode as generateFlutterNode };

function generateNode(node: UiNode, level: number): string {
  return walkTree(node, flutterEmitter, level);
}
