import { UiNode, GenerateResult, PlatformTarget } from '@html-native/shared';
import { countNodes, escapeString, NodeEmitter, walkTree } from '@html-native/generator-core';

// SwiftUI generator: wraps view trees in a struct conforming to the View protocol.

const swiftuiEmitter: NodeEmitter = {
  indentUnit: '    ',

  emitText(node: UiNode, indent: string): string {
    const val = node.value ?? '';
    return `${indent}Text("${escapeString(val)}")`;
  },

  emitButton(indent: string, label: string, _children: string[]): string {
    const action = `${indent}    // action`;
    return `${indent}Button("${escapeString(label)}") {\n${action}\n${indent}}`;
  },

  emitRow(indent: string, children: string[]): string {
    if (!children.length) return `${indent}HStack {}`;
    return `${indent}HStack {\n${children.join('\n')}\n${indent}}`;
  },

  emitColumn(indent: string, children: string[]): string {
    if (!children.length) return `${indent}VStack {}`;
    return `${indent}VStack {\n${children.join('\n')}\n${indent}}`;
  },

  emitContainer(node: UiNode, indent: string, children: string[]): string {
    if (!children.length) return `${indent}Color.clear`;
    return `${indent}VStack {\n${children.join('\n')}\n${indent}}`;
  },

  emitCard(indent: string, children: string[]): string {
    const child = children[0] || '';
    return `${indent}VStack {\n${child}\n${indent}}\n${indent}.background(Color(.systemBackground))\n${indent}.cornerRadius(12)\n${indent}.shadow(radius: 4)`;
  },

  emitImage(node: UiNode, indent: string): string {
    const src = (node.properties.src as string) || '';
    return `${indent}Image("${escapeString(src)}")\n${indent}.resizable()\n${indent}.aspectRatio(contentMode: .fit)`;
  },

  emitTextField(indent: string): string {
    return `${indent}TextField("Input", text: .constant(""))\n${indent}.textFieldStyle(.roundedBorder)`;
  },

  emitAppBar(indent: string, title: string): string {
    return `${indent}.navigationTitle("${escapeString(title)}")`;
  },

  emitScrollView(indent: string, children: string[]): string {
    if (!children.length) return `${indent}ScrollView {}`;
    return `${indent}ScrollView {\n${indent}    LazyVStack {\n${children.join('\n')}\n${indent}    }\n${indent}}`;
  },

  emitForm(indent: string, children: string[]): string {
    return `${indent}Form {\n${children.join('\n')}\n${indent}}`;
  },

  emitFooter(indent: string, children: string[]): string {
    if (!children.length) return `${indent}Spacer()`;
    return `${indent}VStack {\n${children.join('\n')}\n${indent}}`;
  },

  emitDefault(node: UiNode, indent: string, children: string[]): string {
    if (!children.length) {
      const val = node.value ?? '';
      if (val) {
        return `${indent}Text("${escapeString(val)}")`;
      }
      return `${indent}Spacer()`;
    }
    return `${indent}VStack {\n${children.join('\n')}\n${indent}}`;
  },
};

export function generate(node: UiNode, name: string = 'GeneratedView'): GenerateResult {
  const start = performance.now();

  const body = walkTree(node, swiftuiEmitter, 0);
  const indentedBody = body
    .split('\n')
    .map(line => `    ${line}`)
    .join('\n');

  const code = `import SwiftUI

struct ${name}: View {
    var body: some View {
${indentedBody}
    }
}
`;

  return {
    code,
    metadata: {
      platform: 'swiftui' as PlatformTarget,
      nodes: countNodes(node),
      duration: Math.round(performance.now() - start),
    },
  };
}

export { generateNode as generateSwiftUINode };

function generateNode(node: UiNode, level: number): string {
  return walkTree(node, swiftuiEmitter, level);
}
