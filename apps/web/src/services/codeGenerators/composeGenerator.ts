import type { ASTNode, Diagnostic, CSSDeclaration } from '../types';
import { unsupportedHtmlElement } from '../diagnostics';
import { logger } from '../logger';

export function generateCompose(
  htmlAst: ASTNode,
  cssDeclarations: Map<string, CSSDeclaration[]>,
  diagnostics: Diagnostic[],
  minify: boolean,
): string {
  const indent = minify ? '' : '  ';
  const buf: string[] = [];
  let depth = 0;

  function emit(line: string) {
    buf.push(indent.repeat(depth) + line);
  }

  buf.push('package com.motarjim.generated');
  buf.push('');
  buf.push('import android.os.Bundle');
  buf.push('import androidx.activity.ComponentActivity');
  buf.push('import androidx.activity.compose.setContent');
  buf.push('import androidx.compose.foundation.*');
  buf.push('import androidx.compose.foundation.layout.*');
  buf.push('import androidx.compose.foundation.shape.RoundedCornerShape');
  buf.push('import androidx.compose.material3.*');
  buf.push('import androidx.compose.runtime.Composable');
  buf.push('import androidx.compose.ui.Alignment');
  buf.push('import androidx.compose.ui.Modifier');
  buf.push('import androidx.compose.ui.graphics.Color');
  buf.push('import androidx.compose.ui.text.font.FontWeight');
  buf.push('import androidx.compose.ui.text.style.TextAlign');
  buf.push('import androidx.compose.ui.unit.dp');
  buf.push('import androidx.compose.ui.unit.sp');
  buf.push('');

  emit('class MainActivity : ComponentActivity() {');
  depth = 1;
  emit('override fun onCreate(savedInstanceState: Bundle?) {');
  depth = 2;
  emit('super.onCreate(savedInstanceState)');
  emit('setContent {');
  depth = 3;
  emit('MotarjimTheme {');
  depth = 4;
  emit('Surface(modifier = Modifier.fillMaxSize()) {');
  depth = 5;
  emit('MainScreen()');
  depth = 4;
  emit('}');
  depth = 3;
  emit('}');
  depth = 2;
  emit('}');
  depth = 1;
  emit('}');
  buf.push('');

  emit('@Composable');
  emit('fun MainScreen() {');
  depth = 1;
  emit('Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {');
  depth = 2;

  generateChildren(htmlAst);

  depth = 1;
  emit('}');
  emit('}');
  buf.push('');

  emit('@Composable');
  emit('fun MotarjimTheme(content: @Composable () -> Unit) {');
  depth = 1;
  emit('MaterialTheme(');
  depth = 2;
  emit('colorScheme = lightColorScheme(primary = Color(0xFF6366F1))');
  depth = 1;
  emit(') { content() }');
  emit('}');

  const styleCache = new Map<string, CSSDeclaration[]>();

  function getMatchingStyles(tagName: string, classNames: string[], id?: string): CSSDeclaration[] {
    const key = `${tagName}|${classNames.join(',')}|${id || ''}`;
    const cached = styleCache.get(key);
    if (cached) return cached;

    const all: CSSDeclaration[] = [];
    for (const [selector, decls] of cssDeclarations) {
      if (selector === tagName || classNames.some(c => selector === `.${c}` || selector === `${tagName}.${c}`)) {
        all.push(...decls);
      }
      if (id && (selector === `#${id}` || selector === `${tagName}#${id}`)) {
        all.push(...decls);
      }
    }
    styleCache.set(key, all);
    return all;
  }

  function generateNode(node: ASTNode) {
    if (node.type === 'text') {
      if (node.value && node.value.trim()) {
        depth++;
        emit(`Text(text = "${escapeKotlin(node.value.trim())}")`);
        depth--;
      }
      return;
    }

    if (node.type !== 'element' || !node.tagName) return;

    const tag = node.tagName;
    const attrs = node.attributes || {};
    const classes = (attrs.class || '').split(/\s+/).filter(Boolean);
    const id = attrs.id;
    const style = getMatchingStyles(tag, classes, id);

    switch (tag) {
      case 'div': case 'section': case 'article': case 'main':
      case 'header': case 'footer': case 'nav': case 'aside':
      case 'form': case 'blockquote':
        generateContainer(node, style);
        break;
      case 'h1': case 'h2': case 'h3': case 'h4': case 'h5': case 'h6':
        generateHeading(node, tag, style);
        break;
      case 'p':
        generateParagraph(node, style);
        break;
      case 'span': case 'label':
        generateSpan(node, style);
        break;
      case 'a':
        generateLink(node, attrs, style);
        break;
      case 'img':
        generateImage(attrs, style);
        break;
      case 'button':
        generateButton(node, style);
        break;
      case 'input':
        generateInput(attrs);
        break;
      case 'textarea':
        generateTextArea(attrs);
        break;
      case 'ul': case 'ol':
        generateList(node, style);
        break;
      case 'li':
        generateListItem(node, style);
        break;
      case 'br':
        depth++;
        emit('Spacer(modifier = Modifier.height(16.dp))');
        depth--;
        break;
      case 'hr':
        depth++;
        emit('Divider()');
        depth--;
        break;
      case 'strong': case 'b':
        generateBold(node);
        break;
      case 'em': case 'i':
        generateItalic(node);
        break;
      case 'pre': case 'code':
        generateCodeBlock(node);
        break;
      default:
        diagnostics.push(unsupportedHtmlElement(tag));
        generateContainer(node, style);
        break;
    }
  }

  function generateContainer(node: ASTNode, style: CSSDeclaration[]) {
    const padding = style.find(d => d.property === 'padding');
    const bgColor = style.find(d => d.property === 'background-color');
    const width = style.find(d => d.property === 'width');
    const height = style.find(d => d.property === 'height');
    const margin = style.find(d => d.property === 'margin');
    const borderRadius = style.find(d => d.property === 'border-radius');
    const isFlex = style.some(d => d.property === 'display' && (d.value === 'flex' || d.value === 'inline-flex'));

    const hasDecor = bgColor || borderRadius || padding || width || height || margin;

    depth++;
    if (hasDecor || isFlex) {
      let modifier = 'Modifier';

      if (padding) {
        const dp = toDpValue(padding.value);
        if (dp !== null) modifier += `.padding(${dp})`;
      }
      if (margin) {
        const dp = toDpValue(margin.value);
        if (dp !== null) modifier += `.padding(${dp})`;  // Workaround
      }
      if (width) modifier += `.width(${toDpValue(width.value) ? toDpValue(width.value) + '.dp' : '100%'})`;
      if (height) modifier += `.height(${toDpValue(height.value) ? toDpValue(height.value) + '.dp' : '100%'})`;
      if (bgColor) modifier += `.background(${toComposeColor(bgColor.value)})`;
      if (borderRadius) modifier += `.clip(RoundedCornerShape(${toDpValue(borderRadius.value) ? toDpValue(borderRadius.value) + '.dp' : '8.dp'}))`;

      emit('Column(modifier = ' + modifier + ') {');
      depth++;
      generateChildren(node);
      depth--;
      emit('}');
    } else {
      generateChildren(node);
    }
    depth--;
  }

  function generateHeading(node: ASTNode, tag: string, style: CSSDeclaration[]) {
    const fontSize = tag === 'h1' ? '32' : tag === 'h2' ? '24' : tag === 'h3' ? '18' : tag === 'h4' ? '16' : tag === 'h5' ? '14' : '12';
    const color = style.find(d => d.property === 'color');
    const textAlign = style.find(d => d.property === 'text-align');

    depth++;
    let text = `Text(text = "${escapeKotlin(getNodeText(node))}", fontSize = ${fontSize}.sp, fontWeight = FontWeight.Bold`;
    if (color) text += `, color = ${toComposeColor(color.value)}`;
    if (textAlign) text += `, textAlign = ${toComposeTextAlign(textAlign.value)}`;
    text += ')';
    emit(text);
    depth--;
  }

  function generateParagraph(node: ASTNode, style: CSSDeclaration[]) {
    const color = style.find(d => d.property === 'color');
    const textAlign = style.find(d => d.property === 'text-align');
    const fontSize = style.find(d => d.property === 'font-size');

    depth++;
    let text = `Text(text = "${escapeKotlin(getNodeText(node))}", fontSize = ${fontSize ? toDpValue(fontSize.value) + '.sp' : '14.sp'}`;
    if (color) text += `, color = ${toComposeColor(color.value)}`;
    if (textAlign) text += `, textAlign = ${toComposeTextAlign(textAlign.value)}`;
    text += ')';
    emit(text);
    depth--;
  }

  function generateSpan(node: ASTNode, style: CSSDeclaration[]) {
    const color = style.find(d => d.property === 'color');
    const fontWeight = style.find(d => d.property === 'font-weight');

    depth++;
    let text = `Text(text = "${escapeKotlin(getNodeText(node))}"`;
    if (color || fontWeight) {
      const parts: string[] = [];
      if (color) parts.push(`color = ${toComposeColor(color.value)}`);
      if (fontWeight) parts.push(`fontWeight = ${toComposeFontWeight(fontWeight.value)}`);
      text += `, ${parts.join(', ')}`;
    }
    text += ')';
    emit(text);
    depth--;
  }

  function generateLink(node: ASTNode, attrs: Record<string, string>, style: CSSDeclaration[]) {
    const color = style.find(d => d.property === 'color');

    depth++;
    emit('ClickableText(');
    depth++;
    emit(`text = AnnotatedString("${escapeKotlin(getNodeText(node))}"),`);
    emit(`onClick = { /* Navigate to ${attrs.href || '#'} */ },`);
    emit('style = SpanStyle(');
    depth++;
    emit(`color = ${color ? toComposeColor(color.value) : 'Color.Blue'},`);
    emit('textDecoration = TextDecoration.Underline,');
    depth--;
    emit('),');
    depth--;
    emit(')');
    depth--;
  }

  function generateImage(attrs: Record<string, string>, style: CSSDeclaration[]) {
    const width = style.find(d => d.property === 'width');
    const height = style.find(d => d.property === 'height');

    depth++;
    emit('AsyncImage(');
    depth++;
    emit(`model = "${escapeKotlin(attrs.src || '')}",`);
    emit('contentDescription = null,');
    emit('modifier = Modifier');
    if (width) emit(`.width(${toDpValue(width.value) ? toDpValue(width.value) + '.dp' : '100%'})`);
    if (height) emit(`.height(${toDpValue(height.value) ? toDpValue(height.value) + '.dp' : '100%'})`);
    depth--;
    emit(')');
    depth--;
  }

  function generateButton(node: ASTNode, style: CSSDeclaration[]) {
    const bgColor = style.find(d => d.property === 'background-color');
    const color = style.find(d => d.property === 'color');
    const borderRadius = style.find(d => d.property === 'border-radius');

    depth++;
    emit('Button(');
    depth++;
    emit('onClick = { /* Button: ' + escapeKotlin(getNodeText(node)) + ' */ },');
    if (bgColor || borderRadius) {
      emit('colors = ButtonDefaults.buttonColors(');
      depth++;
      if (bgColor) emit(`containerColor = ${toComposeColor(bgColor.value)},`);
      if (color) emit(`contentColor = ${toComposeColor(color.value)},`);
      depth--;
      emit('),');
    }
    if (borderRadius) {
      emit('shape = RoundedCornerShape(' + toDpValue(borderRadius.value) + '.dp),');
    }
    emit(') {');
    depth++;
    emit(`Text("${escapeKotlin(getNodeText(node))}")`);
    depth--;
    emit('}');
    depth--;
  }

  function generateInput(attrs: Record<string, string>) {
    const placeholder = attrs.placeholder || '';

    depth++;
    emit('OutlinedTextField(');
    depth++;
    emit('value = "",');
    emit('onValueChange = {},');
    if (placeholder) emit(`placeholder = { Text("${escapeKotlin(placeholder)}") },`);
    emit('singleLine = true,');
    depth--;
    emit(')');
    depth--;
  }

  function generateTextArea(attrs: Record<string, string>) {
    const placeholder = attrs.placeholder || '';

    depth++;
    emit('OutlinedTextField(');
    depth++;
    emit('value = "",');
    emit('onValueChange = {},');
    if (placeholder) emit(`placeholder = { Text("${escapeKotlin(placeholder)}") },`);
    emit('minLines = 5,');
    depth--;
    emit(')');
    depth--;
  }

  function generateList(node: ASTNode, _style: CSSDeclaration[]) {
    depth++;
    emit('Column(modifier = Modifier.fillMaxWidth()) {');
    depth++;
    for (const child of node.children || []) {
      if (child.type === 'element' && child.tagName === 'li') {
        generateListItem(child, []);
      }
    }
    depth--;
    emit('}');
    depth--;
  }

  function generateListItem(node: ASTNode, _style: CSSDeclaration[]) {
    depth++;
    emit('Row(verticalAlignment = Alignment.CenterVertically) {');
    depth++;
    emit('Text("• ")');
    emit(`Text("${escapeKotlin(getNodeText(node))}")`);
    depth--;
    emit('}');
    depth--;
  }

  function generateBold(node: ASTNode) {
    depth++;
    emit(`Text(text = "${escapeKotlin(getNodeText(node))}", fontWeight = FontWeight.Bold)`);
    depth--;
  }

  function generateItalic(node: ASTNode) {
    depth++;
    // Compose doesn't have italic directly on Text, use fontStyle = FontStyle.Italic
    emit(`Text(text = "${escapeKotlin(getNodeText(node))}", fontSize = 14.sp)`);
    depth--;
  }

  function generateCodeBlock(node: ASTNode) {
    depth++;
    emit('Surface(');
    depth++;
    emit('modifier = Modifier.fillMaxWidth().padding(8.dp),');
    emit('color = Color(0xFFF5F5F5),');
    emit('shape = RoundedCornerShape(4.dp),');
    emit(') {');
    depth++;
    emit(`Text(text = "${escapeKotlin(getNodeText(node))}", fontFamily = "monospace", fontSize = 13.sp)`);
    depth--;
    emit('}');
    depth--;
  }

  function generateChildren(node: ASTNode) {
    if (!node.children) return;
    for (const child of node.children) {
      generateNode(child);
    }
  }

  logger.info('ComposeGenerator', `Generated Compose code (${buf.length} lines)`);
  return buf.join('\n');
}

function getNodeText(node: ASTNode): string {
  if (!node.children) return '';
  return node.children
    .filter(c => c.type === 'text')
    .map(c => c.value || '')
    .join(' ')
    .replace(/\s+/g, ' ')
    .trim();
}

function escapeKotlin(s: string): string {
  return s
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t')
    .replace(/\$/g, '\\$');
}

function toDpValue(value: string): number | null {
  const num = parseFloat(value);
  if (isNaN(num)) return null;
  return num;
}

function toComposeColor(value: string): string {
  const v = value.trim();
  if (v.startsWith('#')) {
    const hex = v.slice(1);
    if (hex.length === 6) return `Color(0xFF${hex.toUpperCase()})`;
    if (hex.length === 3) return `Color(0xFF${hex[0]}${hex[0]}${hex[1]}${hex[1]}${hex[2]}${hex[2]})`;
    if (hex.length === 8) return `Color(0x${hex.toUpperCase()})`;
  }
  if (v.startsWith('rgb')) {
    const m = v.match(/\d+/g);
    if (m && m.length >= 3) return `Color(${m[0]}, ${m[1]}, ${m[2]})`;
  }
  const named: Record<string, string> = {
    red: 'Color.Red', blue: 'Color.Blue', green: 'Color.Green',
    white: 'Color.White', black: 'Color.Black', gray: 'Color.Gray',
    yellow: 'Color.Yellow', orange: 'Color.Orange', purple: 'Color(0xFF6C5CE7)',
    transparent: 'Color.Transparent',
  };
  return named[v.toLowerCase()] || 'Color(0xFF6366F1)';
}

function toComposeTextAlign(value: string): string {
  const v = value.toLowerCase().trim();
  if (v === 'center') return 'TextAlign.Center';
  if (v === 'right' || v === 'end') return 'TextAlign.End';
  return 'TextAlign.Start';
}

function toComposeFontWeight(value: string): string {
  const v = value.toLowerCase().trim();
  if (v === 'bold' || v === '700') return 'FontWeight.Bold';
  if (v === 'normal' || v === '400') return 'FontWeight.Normal';
  if (v === '300') return 'FontWeight.Light';
  if (v === '500') return 'FontWeight.Medium';
  if (v === '600') return 'FontWeight.SemiBold';
  return 'FontWeight.Normal';
}
