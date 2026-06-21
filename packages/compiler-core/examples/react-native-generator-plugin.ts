import type { MotarjimPlugin, PluginApi, GeneratorFactory } from '../src/plugin-api.js';
import type { UiNode, GenerateResult, Result } from '@html-native/shared';

// ── Node Emitter interface (self-contained to avoid generator-core dep) ──

interface NodeEmitter {
  indentUnit: string;
  emitText(node: UiNode, indent: string): string;
  emitButton(node: UiNode, indent: string, label: string, children: string[]): string;
  emitRow(indent: string, children: string[]): string;
  emitColumn(indent: string, children: string[]): string;
  emitContainer(node: UiNode, indent: string, children: string[]): string;
  emitCard(node: UiNode, indent: string, children: string[]): string;
  emitImage(node: UiNode, indent: string): string;
  emitTextField(node: UiNode, indent: string): string;
  emitAppBar(indent: string, title: string): string;
  emitScrollView(indent: string, children: string[]): string;
  emitForm(node: UiNode, indent: string, children: string[]): string;
  emitFooter(indent: string, children: string[]): string;
  emitDefault(node: UiNode, indent: string, children: string[]): string;
}

/**
 * React Native generator plugin.
 * Adds a new target platform generator that produces JSX/React Native code.
 *
 * This demonstrates how any developer can add a new target without
 * modifying the core compiler.
 */

// ── Emitter ──────────────────────────────────────────────

const reactNativeEmitter: NodeEmitter = {
  indentUnit: '  ',

  emitText(node: UiNode, indent: string): string {
    const val = node.value ?? '';
    return `${indent}<Text>${escapeJsx(val)}</Text>`;
  },

  emitButton(node: UiNode, indent: string, label: string, _children: string[]): string {
    const a11y = node.accessibility;
    const accessibilityLabel = a11y?.label
      ? `\n${indent}  accessibilityLabel="${escapeJsx(a11y.label)}"`
      : '';
    return `${indent}<TouchableOpacity${accessibilityLabel}>\n${indent}  <Text>${escapeJsx(label)}</Text>\n${indent}</TouchableOpacity>`;
  },

  emitRow(indent: string, children: string[]): string {
    if (!children.length) return `${indent}<View style={{ flexDirection: 'row' }} />`;
    const render = children.join('\n');
    return `${indent}<View style={{ flexDirection: 'row' }}>\n${render}\n${indent}</View>`;
  },

  emitColumn(indent: string, children: string[]): string {
    if (!children.length) return `${indent}<View />`;
    const render = children.join('\n');
    return `${indent}<View>\n${render}\n${indent}</View>`;
  },

  emitContainer(_node: UiNode, indent: string, children: string[]): string {
    if (!children.length) return `${indent}<View />`;
    const render = children.join('\n');
    return `${indent}<View>\n${render}\n${indent}</View>`;
  },

  emitCard(_node: UiNode, indent: string, children: string[]): string {
    const child = children[0] ?? '<View />';
    return `${indent}<View style={{\n${indent}  shadowColor: '#000',\n${indent}  shadowOffset: { width: 0, height: 2 },\n${indent}  shadowOpacity: 0.1,\n${indent}  elevation: 3,\n${indent}  borderRadius: 8,\n${indent}  backgroundColor: '#fff',\n${indent}}}>\n${indent}  ${child}\n${indent}</View>`;
  },

  emitImage(node: UiNode, indent: string): string {
    const src = String(node.properties.src ?? '');
    const alt = node.accessibility?.label || String(node.properties.alt ?? '');
    return `${indent}<Image\n${indent}  source={{ uri: "${escapeJsx(src)}" }}\n${indent}  accessibilityLabel="${escapeJsx(alt)}"\n${indent}  style={{ width: '100%', height: 'auto' }}\n${indent}/>`;
  },

  emitTextField(_node: UiNode, indent: string): string {
    return `${indent}<TextInput style={{ borderWidth: 1, borderColor: '#ccc', borderRadius: 4, padding: 8 }} />`;
  },

  emitAppBar(_indent: string, _title: string): string {
    return '';
  },

  emitScrollView(indent: string, children: string[]): string {
    const render = children.join('\n');
    return `${indent}<ScrollView>\n${render}\n${indent}</ScrollView>`;
  },

  emitForm(_node: UiNode, indent: string, children: string[]): string {
    const render = children.join('\n');
    return `${indent}<View>\n${render}\n${indent}</View>`;
  },

  emitFooter(indent: string, children: string[]): string {
    const render = children.join('\n');
    return `${indent}<View style={{ padding: 16, backgroundColor: '#f8f8f8' }}>\n${render}\n${indent}</View>`;
  },

  emitDefault(node: UiNode, indent: string, children: string[]): string {
    if (!children.length) {
      const val = node.value ?? '';
      if (val) return `${indent}<Text>${escapeJsx(val)}</Text>`;
      return `${indent}<View />`;
    }
    const render = children.join('\n');
    return `${indent}<View>\n${render}\n${indent}</View>`;
  },
};

function escapeJsx(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

// ── Walk tree (adapted from generator-core but self-contained) ──

function walkTree(node: UiNode, emitter: NodeEmitter, level: number = 0): string {
  const i = emitter.indentUnit.repeat(level);
  const next = level + 1;

  switch (node.type) {
    case 'Text':
      return emitter.emitText(node, i);
    case 'Button': {
      const label = node.value || 'Button';
      const rendered = node.children.map(c => walkTree(c, emitter, next));
      return emitter.emitButton(node, i, label, rendered);
    }
    case 'Row':
      return emitter.emitRow(i, node.children.map(c => walkTree(c, emitter, next)));
    case 'Column':
      return emitter.emitColumn(i, node.children.map(c => walkTree(c, emitter, next)));
    case 'Card':
      return emitter.emitCard(node, i, node.children.map(c => walkTree(c, emitter, next)));
    case 'Image':
      return emitter.emitImage(node, i);
    case 'TextField':
      return emitter.emitTextField(node, i);
    case 'ScrollView':
    case 'ListView':
    case 'LazyList':
      return emitter.emitScrollView(i, node.children.map(c => walkTree(c, emitter, next)));
    case 'Form':
      return emitter.emitForm(node, i, node.children.map(c => walkTree(c, emitter, next)));
    case 'Container':
      return emitter.emitContainer(node, i, node.children.map(c => walkTree(c, emitter, next)));
    case 'NavigationBar':
    case 'AppBar':
      return emitter.emitAppBar(i, node.value || '');
    case 'Footer':
      return emitter.emitFooter(i, node.children.map(c => walkTree(c, emitter, next)));
    default:
      return emitter.emitDefault(node, i, node.children.map(c => walkTree(c, emitter, next)));
  }
}

// ── Generator Factory ───────────────────────────────────

export const reactNativeGeneratorFactory: GeneratorFactory = {
  platform: 'react-native' as any,
  name: 'React Native (JSX)',

  generate(ir: UiNode, _ctx: any): Result<GenerateResult> {
    const start = performance.now();
    const body = walkTree(ir, reactNativeEmitter, 1);
    const lines = body.split('\n');

    const code = `import React from 'react';
import {
  View,
  Text,
  ScrollView,
  TextInput,
  Image,
  TouchableOpacity,
} from 'react-native';

export default function App() {
  return (
${lines.map(l => `    ${l}`).join('\n')}
  );
}
`;

    return {
      ok: true,
      value: {
        code,
        metadata: {
          platform: 'react-native' as any,
          nodes: countNodes(ir),
          duration: Math.round(performance.now() - start),
        },
      },
      diagnostics: [],
    };
  },
};

function countNodes(node: UiNode): number {
  let count = 1;
  for (const child of node.children) count += countNodes(child);
  return count;
}
