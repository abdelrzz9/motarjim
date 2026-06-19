import { UiNode, GenerateResult, PlatformTarget } from '@html-native/shared';
import { countNodes, escapeString, NodeEmitter, walkTree } from '@html-native/generator-core';

// Compose generator: wraps composable trees in a @Composable function with Material3 imports.

const composeEmitter: NodeEmitter = {
  indentUnit: '    ',

  emitText(node: UiNode, indent: string): string {
    const val = node.value ?? '';
    return `${indent}Text(text = "${escapeString(val)}")`;
  },

  emitButton(indent: string, label: string, children: string[]): string {
    const onClick = 'onClick = { }';
    const content = children.length
      ? children.join('\n')
      : `${indent}    Text(text = "${escapeString(label)}")`;
    return `${indent}Button(\n${indent}    ${onClick}\n${indent}) {\n${content}\n${indent}}`;
  },

  emitRow(indent: string, children: string[]): string {
    if (!children.length) return `${indent}Row {}`;
    return `${indent}Row(\n${indent}    horizontalArrangement = Arrangement.spacedBy(8.dp)\n${indent}) {\n${children.join('\n')}\n${indent}}`;
  },

  emitColumn(indent: string, children: string[]): string {
    if (!children.length) return `${indent}Column {}`;
    return `${indent}Column {\n${children.join('\n')}\n${indent}}`;
  },

  emitContainer(node: UiNode, indent: string, children: string[]): string {
    if (!children.length) return `${indent}Box(modifier = Modifier)`;
    return `${indent}Box(modifier = Modifier) {\n${children.join('\n')}\n${indent}}`;
  },

  emitCard(indent: string, children: string[]): string {
    const child = children[0] || '';
    return `${indent}Card(\n${indent}    modifier = Modifier\n${indent}) {\n${child}\n${indent}}`;
  },

  emitImage(node: UiNode, indent: string): string {
    const src = (node.properties.src as string) || '';
    return `${indent}Image(\n${indent}    painter = painterResource(id = R.drawable.${escapeString(src)}),\n${indent}    contentDescription = "${escapeString((node.properties.alt as string) || '')}"\n${indent})`;
  },

  emitTextField(indent: string): string {
    return `${indent}OutlinedTextField(\n${indent}    value = "",\n${indent}    onValueChange = { },\n${indent}    label = { Text("Input") }\n${indent})`;
  },

  emitAppBar(indent: string, title: string): string {
    return `${indent}TopAppBar(\n${indent}    title = { Text("${escapeString(title)}") }\n${indent})`;
  },

  emitScrollView(indent: string, children: string[]): string {
    if (!children.length) return `${indent}LazyColumn {}`;
    return `${indent}LazyColumn {\n${children.join('\n')}\n${indent}}`;
  },

  emitForm(indent: string, children: string[]): string {
    return `${indent}Column {\n${children.join('\n')}\n${indent}}`;
  },

  emitFooter(indent: string, children: string[]): string {
    return `${indent}Column {\n${children.join('\n')}\n${indent}}`;
  },

  emitDefault(node: UiNode, indent: string, children: string[]): string {
    if (!children.length) {
      const val = node.value ?? '';
      if (val) {
        return `${indent}Text(text = "${escapeString(val)}")`;
      }
      return `${indent}Spacer(modifier = Modifier.size(0.dp))`;
    }
    return `${indent}Column {\n${children.join('\n')}\n${indent}}`;
  },
};

export function generate(node: UiNode, name: string = 'GeneratedView'): GenerateResult {
  const start = performance.now();

  const body = walkTree(node, composeEmitter, 0);
  const indentedBody = body
    .split('\n')
    .map(line => `    ${line}`)
    .join('\n');

  const code = `import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.ui.Modifier

@Composable
fun ${name}() {
${indentedBody}
}
`;

  return {
    code,
    metadata: {
      platform: 'compose' as PlatformTarget,
      nodes: countNodes(node),
      duration: Math.round(performance.now() - start),
    },
  };
}

export { generateNode as generateComposeNode };

function generateNode(node: UiNode, level: number): string {
  return walkTree(node, composeEmitter, level);
}
