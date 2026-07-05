import type { ASTNode, Diagnostic, CSSDeclaration } from '../types';
import { unsupportedHtmlElement } from '../diagnostics';
import { logger } from '../logger';

export function generateSwiftUI(
  htmlAst: ASTNode,
  cssDeclarations: Map<string, CSSDeclaration[]>,
  diagnostics: Diagnostic[],
  minify: boolean,
): string {
  const indent = minify ? '' : '    ';
  const buf: string[] = [];
  let depth = 0;

  function emit(line: string) {
    buf.push(indent.repeat(depth) + line);
  }

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

  buf.push('import SwiftUI');
  buf.push('');

  emit('@main');
  emit('struct MotarjimApp: App {');
  depth = 1;
  emit('var body: some Scene {');
  depth = 2;
  emit('WindowGroup {');
  depth = 3;
  emit('ContentView()');
  depth = 2;
  emit('}');
  depth = 1;
  emit('}');
  emit('}');
  buf.push('');

  emit('struct ContentView: View {');
  depth = 1;
  emit('var body: some View {');
  depth = 2;
  emit('ScrollView {');
  depth = 3;
  emit('VStack(alignment: .leading, spacing: 8) {');
  depth = 4;

  generateChildren(htmlAst);

  depth = 3;
  emit('}');
  emit('.padding(16)');
  depth = 2;
  emit('}');
  depth = 1;
  emit('}');
  emit('}');

  function generateNode(node: ASTNode) {
    if (node.type === 'text') {
      if (node.value && node.value.trim()) {
        depth++;
        emit(`Text("${escapeSwift(node.value.trim())}")`);
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
        emit('Divider()');
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
    const hasChildren = node.children && node.children.some(c =>
      (c.type === 'element') || (c.type === 'text' && c.value && c.value.trim())
    );
    const isFlex = style.some(d => d.property === 'display' && (d.value === 'flex' || d.value === 'inline-flex'));
    const flexDir = style.find(d => d.property === 'flex-direction');
    const isColumn = flexDir ? flexDir.value === 'column' : false;
    const padding = style.find(d => d.property === 'padding');
    const bgColor = style.find(d => d.property === 'background-color' || d.property === 'background');
    const width = style.find(d => d.property === 'width');
    const height = style.find(d => d.property === 'height');
    const margin = style.find(d => d.property === 'margin');
    const borderRadius = style.find(d => d.property === 'border-radius');
    const opacity = style.find(d => d.property === 'opacity');
    const align = style.find(d => d.property === 'align-items');
    const gap = style.find(d => d.property === 'gap');

    const hasDecor = bgColor != null || borderRadius != null || padding != null || width != null || height != null || margin != null || opacity != null;

    if (!hasChildren && !hasDecor) return;

    if (isFlex && hasChildren) {
      depth++;
      if (isColumn) {
        emit('VStack(alignment: .leading, spacing: 4) {');
      } else {
        emit('HStack(spacing: 4) {');
      }
      depth++;
      generateChildren(node);
      depth--;
      emit('}');

      const mods: string[] = [];
      if (padding) mods.push(`.padding(${toCGFloat(padding.value)})`);
      if (margin) mods.push(`.padding(${toCGFloat(margin.value)})`);
      if (width) mods.push(`.frame(width: ${toCGFloat(width.value)})`);
      if (height) mods.push(`.frame(height: ${toCGFloat(height.value)})`);
      if (bgColor) mods.push(`.background(${toSwiftUIColor(bgColor.value)})`);
      if (borderRadius) mods.push(`.cornerRadius(${toCGFloat(borderRadius.value)})`);
      if (opacity) mods.push(`.opacity(${parseFloat(opacity.value) / 100})`);
      if (align) {
        if (isColumn) {
          if (align.value === 'center') mods.push('.frame(maxWidth: .infinity, alignment: .center)');
          else if (align.value === 'flex-end') mods.push('.frame(maxWidth: .infinity, alignment: .trailing)');
        } else {
          if (align.value === 'center') mods.push('.frame(maxHeight: .infinity, alignment: .center)');
          else if (align.value === 'flex-end') mods.push('.frame(maxHeight: .infinity, alignment: .bottom)');
        }
      }
      if (gap) mods.push(`.padding(.horizontal, ${toCGFloat(gap.value)})`);

      for (const m of mods) emit(m);
      depth--;
      return;
    }

    if (hasDecor) {
      depth++;
      emit('VStack(alignment: .leading, spacing: 4) {');
      depth++;
      generateChildren(node);
      depth--;
      emit('}');

      const mods: string[] = [];
      if (padding) mods.push(`.padding(${toCGFloat(padding.value)})`);
      if (margin) mods.push(`.padding(${toCGFloat(margin.value)})`);
      if (width) mods.push(`.frame(width: ${toCGFloat(width.value)})`);
      if (height) mods.push(`.frame(height: ${toCGFloat(height.value)})`);
      if (bgColor) mods.push(`.background(${toSwiftUIColor(bgColor.value)})`);
      if (borderRadius) mods.push(`.cornerRadius(${toCGFloat(borderRadius.value)})`);
      if (opacity) mods.push(`.opacity(${parseFloat(opacity.value)})`);

      for (const m of mods) emit(m);
      depth--;
    } else {
      depth++;
      generateChildren(node);
      depth--;
    }
  }

  function generateChildren(node: ASTNode) {
    if (!node.children) return;
    for (const child of node.children) {
      generateNode(child);
    }
  }

  function generateHeading(node: ASTNode, tag: string, style: CSSDeclaration[]) {
    const fontSize = tag === 'h1' ? 'largeTitle' : tag === 'h2' ? 'title' : tag === 'h3' ? 'title2' : tag === 'h4' ? 'title3' : tag === 'h5' ? 'headline' : 'subheadline';
    const color = style.find(d => d.property === 'color');

    depth++;
    emit(`Text("${escapeSwift(getNodeText(node))}")`);
    emit(`.font(.${fontSize})`);
    emit('.fontWeight(.bold)');
    if (color) emit(`.foregroundColor(${toSwiftUIColor(color.value)})`);
    depth--;
  }

  function generateParagraph(node: ASTNode, style: CSSDeclaration[]) {
    const color = style.find(d => d.property === 'color');
    const textAlign = style.find(d => d.property === 'text-align');
    const fontSize = style.find(d => d.property === 'font-size');

    depth++;
    emit(`Text("${escapeSwift(getNodeText(node))}")`);
    if (fontSize) emit(`.font(.system(size: ${toCGFloat(fontSize.value)}))`);
    if (color) emit(`.foregroundColor(${toSwiftUIColor(color.value)})`);
    if (textAlign) emit(`.multilineTextAlignment(.${textAlign.value})`);
    depth--;
  }

  function generateSpan(node: ASTNode, style: CSSDeclaration[]) {
    const color = style.find(d => d.property === 'color');
    const fontWeight = style.find(d => d.property === 'font-weight');

    depth++;
    emit(`Text("${escapeSwift(getNodeText(node))}")`);
    if (fontWeight) emit(`.fontWeight(${toSwiftUIFontWeight(fontWeight.value)})`);
    if (color) emit(`.foregroundColor(${toSwiftUIColor(color.value)})`);
    depth--;
  }

  function generateLink(node: ASTNode, attrs: Record<string, string>, _style: CSSDeclaration[]) {
    depth++;
    emit(`Link("${escapeSwift(getNodeText(node))}", destination: URL(string: "${escapeSwift(attrs.href || '#')}")!)`);
    depth--;
  }

  function generateImage(attrs: Record<string, string>, style: CSSDeclaration[]) {
    const width = style.find(d => d.property === 'width');
    const height = style.find(d => d.property === 'height');
    const src = attrs.src || '';

    depth++;
    emit('AsyncImage(url: URL(string: "' + escapeSwift(src) + '")) { phase in');
    depth++;
    emit('if let image = phase.image {');
    depth++;
    emit('image');
    emit('.resizable()');
    if (width) emit(`.frame(width: ${toCGFloat(width.value)})`);
    if (height) emit(`.frame(height: ${toCGFloat(height.value)})`);
    depth--;
    emit('} else if phase.error != nil {');
    depth++;
    emit('Color.gray');
    depth--;
    emit('} else {');
    depth++;
    emit('ProgressView()');
    depth--;
    emit('}');
    depth--;
    emit('}');
    depth--;
  }

  function generateButton(node: ASTNode, style: CSSDeclaration[]) {
    const bgColor = style.find(d => d.property === 'background-color');
    const color = style.find(d => d.property === 'color');
    const borderRadius = style.find(d => d.property === 'border-radius');

    depth++;
    emit('Button(action: { /* ' + escapeSwift(getNodeText(node)) + ' */ }) {');
    depth++;
    emit(`Text("${escapeSwift(getNodeText(node))}")`);
    if (color) emit(`.foregroundColor(${toSwiftUIColor(color.value)})`);
    depth--;
    emit('}');
    if (bgColor) emit(`.background(${toSwiftUIColor(bgColor.value)})`);
    if (borderRadius) emit(`.cornerRadius(${toCGFloat(borderRadius.value)})`);
    depth--;
  }

  function generateInput(attrs: Record<string, string>) {
    const placeholder = attrs.placeholder || '';
    depth++;
    emit('TextField("' + escapeSwift(placeholder) + '", text: .constant(""))');
    emit('.textFieldStyle(.roundedBorder)');
    depth--;
  }

  function generateTextArea(attrs: Record<string, string>) {
    const placeholder = attrs.placeholder || '';
    depth++;
    emit('TextEditor(text: .constant(""))');
    emit('.overlay(alignment: .topLeading) {');
    depth++;
    emit(`Text("${escapeSwift(placeholder)}").foregroundColor(.gray).padding(8)`);
    depth--;
    emit('}');
    depth--;
  }

  function generateList(node: ASTNode, _style: CSSDeclaration[]) {
    depth++;
    emit('VStack(alignment: .leading, spacing: 4) {');
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
    emit('HStack(spacing: 4) {');
    depth++;
    emit('Text("•")');
    emit(`Text("${escapeSwift(getNodeText(node))}")`);
    depth--;
    emit('}');
    depth--;
  }

  function generateBold(node: ASTNode) {
    depth++;
    emit(`Text("${escapeSwift(getNodeText(node))}").fontWeight(.bold)`);
    depth--;
  }

  function generateItalic(node: ASTNode) {
    depth++;
    emit(`Text("${escapeSwift(getNodeText(node))}").italic()`);
    depth--;
  }

  function generateCodeBlock(node: ASTNode) {
    depth++;
    emit('Text("' + escapeSwift(getNodeText(node)) + '")');
    emit('.font(.system(.body, design: .monospaced))');
    emit('.padding(8)');
    emit('.background(Color.gray.opacity(0.1))');
    emit('.cornerRadius(4)');
    depth--;
  }

  logger.info('SwiftUIGenerator', `Generated SwiftUI code (${buf.length} lines)`);
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

function escapeSwift(s: string): string {
  return s
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t');
}

function toCGFloat(value: string): string {
  const num = parseFloat(value);
  if (isNaN(num)) return value;
  return num % 1 === 0 ? `${num}` : `${num}`;
}

function toSwiftUIColor(value: string): string {
  const v = value.trim();
  if (v.startsWith('#')) {
    const hex = v.slice(1);
    if (hex.length === 6) {
      const r = parseInt(hex.slice(0, 2), 16) / 255;
      const g = parseInt(hex.slice(2, 4), 16) / 255;
      const b = parseInt(hex.slice(4, 6), 16) / 255;
      return `Color(red: ${r}, green: ${g}, blue: ${b})`;
    }
  }
  const named: Record<string, string> = {
    red: '.red', blue: '.blue', green: '.green',
    white: '.white', black: '.black', gray: '.gray',
    yellow: '.yellow', orange: '.orange', purple: '.purple',
    transparent: '.clear',
  };
  return named[v.toLowerCase()] || '.accentColor';
}

function toSwiftUIFontWeight(value: string): string {
  const v = value.toLowerCase().trim();
  if (v === 'bold' || v === '700') return '.bold';
  if (v === 'normal' || v === '400') return '.regular';
  if (v === '300') return '.light';
  if (v === '500') return '.medium';
  if (v === '600') return '.semibold';
  return '.regular';
}
