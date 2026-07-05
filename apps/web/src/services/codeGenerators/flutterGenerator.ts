import type { ASTNode, Diagnostic, CSSDeclaration } from '../types';
import { unsupportedHtmlElement, createWarningDiagnostic } from '../diagnostics';
import { logger } from '../logger';

export function generateFlutter(
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

  buf.push("import 'package:flutter/material.dart';");
  buf.push('');
  buf.push('void main() => runApp(const MotarjimApp());');
  buf.push('');

  emit('class MotarjimApp extends StatelessWidget {');
  depth = 1;
  emit('const MotarjimApp({super.key});');
  buf.push('');
  emit('@override');
  emit('Widget build(BuildContext context) {');
  depth = 2;
  emit('return MaterialApp(');
  depth = 3;
  emit("title: 'Motarjim Generated',");
  emit('theme: ThemeData(');
  depth = 4;
  emit('useMaterial3: true,');
  emit('colorSchemeSeed: const Color(0xFF6366F1),');
  depth = 3;
  emit('),');
  emit('home: const MotarjimHome(),');
  depth = 2;
  emit(');');
  depth = 1;
  emit('}');
  emit('}');
  buf.push('');

  emit('class MotarjimHome extends StatelessWidget {');
  depth = 1;
  emit('const MotarjimHome({super.key});');
  buf.push('');
  emit('@override');
  emit('Widget build(BuildContext context) {');
  depth = 2;
  emit('return Scaffold(');
  depth = 3;
  emit('body: ');
  depth = 4;

  generateChildren(htmlAst);

  depth = 3;
  emit(');');
  depth = 2;
  emit('}');
  depth = 1;
  emit('}');

  function generateNode(node: ASTNode) {
    if (node.type === 'text') {
      if (node.value && node.value.trim()) {
        const text = escapeDart(node.value.trim());
        depth++;
        emit(`const Text('${text}'),`);
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
      case 'div':
      case 'section':
      case 'article':
      case 'main':
      case 'header':
      case 'footer':
      case 'nav':
      case 'aside':
        generateBox(node, style);
        break;
      case 'h1': case 'h2': case 'h3': case 'h4': case 'h5': case 'h6':
        generateHeading(node, tag, style);
        break;
      case 'p':
        generateParagraph(node, style);
        break;
      case 'span':
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
        emit('const SizedBox(height: 16),');
        depth--;
        break;
      case 'hr':
        depth++;
        emit('const Divider(),');
        depth--;
        break;
      case 'table':
        generateTable(node, style);
        break;
      case 'form':
        generateBox(node, style);
        break;
      case 'label':
        generateSpan(node, style);
        break;
      case 'strong': case 'b':
        generateBold(node);
        break;
      case 'em': case 'i':
        generateItalic(node);
        break;
      case 'blockquote':
        generateBox(node, style);
        break;
      case 'pre': case 'code':
        generateCodeBlock(node);
        break;
      default:
        diagnostics.push(unsupportedHtmlElement(tag));
        generateBox(node, style);
        break;
    }
  }

  function generateBox(node: ASTNode, style: CSSDeclaration[]) {
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
    const justify = style.find(d => d.property === 'justify-content');
    const align = style.find(d => d.property === 'align-items');
    const gap = style.find(d => d.property === 'gap');
    const margin = style.find(d => d.property === 'margin');
    const borderRadius = style.find(d => d.property === 'border-radius');
    const opacity = style.find(d => d.property === 'opacity');
    const position = style.find(d => d.property === 'position');

    const hasExplicitSize = width != null || height != null;
    const hasDecoration = bgColor != null || borderRadius != null;
    const hasPadding = padding != null;
    const hasMargin = margin != null;
    const hasModifiers = hasExplicitSize || hasDecoration || hasPadding || hasMargin || opacity != null;

    if (!hasModifiers && !isFlex && !hasChildren) return;

    if (isFlex && hasChildren) {
      depth++;
      const direction = isColumn ? 'Column' : 'Row';
      emit(`${direction}(`);
      depth++;

      const props: string[] = [];
      if (align) {
        const caa = crossAxisAlignment(align.value);
        if (caa) props.push(`crossAxisAlignment: ${caa}`);
      }
      if (justify) {
        const maa = mainAxisAlignment(justify.value);
        if (maa) props.push(`mainAxisAlignment: ${maa}`);
      }
      if (gap) props.push(`spacing: ${toDouble(gap.value)}`);

      if (hasPadding) {
        const ep = toEdgeInsets(padding!.value);
        if (ep) {
          depth++;
          emit(`padding: ${ep},`);
          depth--;
        }
      }
      if (hasMargin) {
        const ep = toEdgeInsets(margin!.value);
        if (ep) {
          depth++;
          emit(`margin: ${ep},`);
          depth--;
        }
      }

      if (props.length > 0) {
        for (const p of props) {
          emit(p + ',');
        }
      }

      emit('children: [');
      depth++;
      generateChildren(node);
      depth--;
      emit('],');

      if (hasExplicitSize || hasDecoration || opacity != null) {
        if (opacity != null) {
          emit('],');
          depth--;
          emit(`Opacity(opacity: ${parseFloat(opacity.value) || 1},`);
          depth++;
          emit('child: ');
        }

        emit('],');
        depth--;
        emit('),');

        if (hasExplicitSize || hasDecoration) {
          const decoParts: string[] = [];
          if (bgColor) decoParts.push(`color: ${toColor(bgColor.value)}`);
          if (borderRadius) decoParts.push(`borderRadius: ${toBorderRadius(borderRadius.value)}`);
          const deco = decoParts.length > 0 ? `decoration: BoxDecoration(${decoParts.join(', ')}),` : '';

          const containerProps: string[] = [];
          if (width) containerProps.push(`width: ${toDouble(width.value)}`);
          if (height) containerProps.push(`height: ${toDouble(height.value)}`);
          if (deco) containerProps.push(deco);

          emit('Container(');
          depth++;
          for (const cp of containerProps) emit(cp);
          if (hasChildren || opacity != null) {
            emit('child: ');
            depth++;
          }
        }
      }

      if (!hasExplicitSize && !hasDecoration && opacity == null) {
        depth--;
        emit('),');
        depth--;
      }
      return;
    }

    if (position != null && position.value === 'absolute') {
      depth++;
      emit('Stack(');
      depth++;
      emit('children: [');

      const top = style.find(d => d.property === 'top');
      const right = style.find(d => d.property === 'right');
      const bottom = style.find(d => d.property === 'bottom');
      const left = style.find(d => d.property === 'left');

      depth++;
      emit('Positioned(');
      depth++;
      if (top) emit(`top: ${toDouble(top.value)},`);
      if (right) emit(`right: ${toDouble(right.value)},`);
      if (bottom) emit(`bottom: ${toDouble(bottom.value)},`);
      if (left) emit(`left: ${toDouble(left.value)},`);
      emit('child: ');
      depth++;
      emitContainerContent(node, style, hasChildren);
      depth--;
      emit('),');
      depth--;
      emit('],');
      depth--;
      emit('),');
      depth--;
      return;
    }

    if (!hasModifiers && hasChildren) {
      depth++;
      generateChildren(node);
      depth--;
      return;
    }

    depth++;
    emitContainerContent(node, style, hasChildren);
    depth--;
  }

  function emitContainerContent(node: ASTNode, style: CSSDeclaration[], hasChildren: boolean | undefined) {
    const bgColor = style.find(d => d.property === 'background-color' || d.property === 'background');
    const width = style.find(d => d.property === 'width');
    const height = style.find(d => d.property === 'height');
    const padding = style.find(d => d.property === 'padding');
    const margin = style.find(d => d.property === 'margin');
    const borderRadius = style.find(d => d.property === 'border-radius');
    const opacity = style.find(d => d.property === 'opacity');

    const hasExplicitSize = width != null || height != null;
    const hasDecoration = bgColor != null || borderRadius != null;
    const hasPadding = padding != null;
    const hasMargin = margin != null;
    const hasModifiers = hasExplicitSize || hasDecoration || hasPadding || hasMargin || opacity != null;

    if (!hasModifiers) {
      generateChildren(node);
      return;
    }

    if (opacity != null) {
      emit('Opacity(');
      depth++;
      emit(`opacity: ${parseFloat(opacity.value) || 1},`);
      emit('child: ');
      depth++;
      emitContainerContent(node, style, hasChildren);
      depth--;
      emit('),');
      depth--;
      return;
    }

    const decoParts: string[] = [];
    if (bgColor) decoParts.push(`color: ${toColor(bgColor.value)}`);
    if (borderRadius) decoParts.push(`borderRadius: ${toBorderRadius(borderRadius.value)}`);

    const props: string[] = [];
    if (width) props.push(`width: ${toDouble(width.value)}`);
    if (height) props.push(`height: ${toDouble(height.value)}`);
    if (hasPadding && !hasMargin) {
      const ep = toEdgeInsets(padding!.value);
      if (ep) props.push(`padding: ${ep}`);
    }
    if (hasMargin) {
      const ep = toEdgeInsets(margin!.value);
      if (ep) {
        props.push(`padding: ${ep}`);
        if (!hasPadding) {
          diagnostics.push(createWarningDiagnostic(
            'W0401', 'Margin without padding',
            'Margin is applied as padding. Consider using SizedBox for margins.',
          ));
        }
      }
    }
    if (decoParts.length > 0) {
      props.push(`decoration: BoxDecoration(${decoParts.join(', ')})`);
    }

    if (props.length > 0 || hasChildren) {
      emit('Container(');
      depth++;
      for (const p of props) emit(p + ',');
      if (hasChildren) {
        emit('child: ');
        depth++;
        generateChildren(node);
        depth--;
      }
      depth--;
      emit('),');
    }
  }

  function generateChildren(node: ASTNode) {
    if (!node.children) return;
    for (const child of node.children) {
      generateNode(child);
    }
  }

  function generateHeading(node: ASTNode, tag: string, style: CSSDeclaration[]) {
    const fontSize = tag === 'h1' ? '32' : tag === 'h2' ? '24' : tag === 'h3' ? '20' : tag === 'h4' ? '16' : tag === 'h5' ? '14' : '12';
    const color = style.find(d => d.property === 'color');
    const textAlign = style.find(d => d.property === 'text-align');

    depth++;
    emit('Text(');
    depth++;
    emit(`'${escapeDart(getNodeText(node))}',`);
    emit('style: TextStyle(');
    depth++;
    emit(`fontSize: ${fontSize},`);
    emit('fontWeight: FontWeight.bold,');
    if (color) emit(`color: ${toColor(color.value)},`);
    if (textAlign) emit(`textAlign: ${toTextAlign(textAlign.value)},`);
    depth--;
    emit('),');
    depth--;
    emit('),');
    depth--;
  }

  function generateParagraph(node: ASTNode, style: CSSDeclaration[]) {
    const color = style.find(d => d.property === 'color');
    const textAlign = style.find(d => d.property === 'text-align');
    const fontSize = style.find(d => d.property === 'font-size');

    depth++;
    emit('Text(');
    depth++;
    emit(`'${escapeDart(getNodeText(node))}',`);
    emit('style: TextStyle(');
    depth++;
    emit(`fontSize: ${fontSize ? toDouble(fontSize.value) : '14'},`);
    if (color) emit(`color: ${toColor(color.value)},`);
    if (textAlign) emit(`textAlign: ${toTextAlign(textAlign.value)},`);
    depth--;
    emit('),');
    depth--;
    emit('),');
    depth--;
  }

  function generateSpan(node: ASTNode, style: CSSDeclaration[]) {
    const color = style.find(d => d.property === 'color');
    const fontWeight = style.find(d => d.property === 'font-weight');

    depth++;
    const hasStyle = color != null || fontWeight != null;
    if (hasStyle) {
      emit('Text(');
      depth++;
      emit(`'${escapeDart(getNodeText(node))}',`);
      emit('style: TextStyle(');
      depth++;
      if (color) emit(`color: ${toColor(color.value)},`);
      if (fontWeight) emit(`fontWeight: ${toFontWeight(fontWeight.value)},`);
      depth--;
      emit('),');
      depth--;
      emit('),');
    } else {
      emit(`const Text('${escapeDart(getNodeText(node))}'),`);
    }
    depth--;
  }

  function generateLink(node: ASTNode, attrs: Record<string, string>, style: CSSDeclaration[]) {
    const href = attrs.href || '';
    const color = style.find(d => d.property === 'color');

    depth++;
    emit('GestureDetector(');
    depth++;
    emit('onTap: () {}');
    if (href) emit(`// Navigate to ${href}`);
    emit(',');
    emit('child: Text(');
    depth++;
    emit(`'${escapeDart(getNodeText(node))}',`);
    emit('style: TextStyle(');
    depth++;
    emit(`color: ${color ? toColor(color.value) : 'Colors.blue'},`);
    emit('decoration: TextDecoration.underline,');
    depth--;
    emit('),');
    depth--;
    emit('),');
    depth--;
    emit('),');
    depth--;
  }

  function generateImage(attrs: Record<string, string>, style: CSSDeclaration[]) {
    const src = attrs.src || '';
    const width = style.find(d => d.property === 'width');
    const height = style.find(d => d.property === 'height');

    depth++;
    if (src.startsWith('http')) {
      emit('Image.network(');
      depth++;
      emit(`'${escapeDart(src)}',`);
      if (width) emit(`width: ${toDouble(width.value)},`);
      if (height) emit(`height: ${toDouble(height.value)},`);
      emit('fit: BoxFit.cover,');
      depth--;
      emit('),');
    } else {
      emit(`Image.asset('${escapeDart(src || 'placeholder')}',`);
      depth++;
      if (width) emit(`width: ${toDouble(width.value)},`);
      if (height) emit(`height: ${toDouble(height.value)},`);
      depth--;
      emit('),');
    }
    depth--;
  }

  function generateButton(node: ASTNode, style: CSSDeclaration[]) {
    const bgColor = style.find(d => d.property === 'background-color');
    const color = style.find(d => d.property === 'color');
    const borderRadius = style.find(d => d.property === 'border-radius');
    const padding = style.find(d => d.property === 'padding');

    depth++;
    emit('ElevatedButton(');
    depth++;
    emit('onPressed: () {},');
    emit('child: Text(');
    depth++;
    emit(`'${escapeDart(getNodeText(node))}',`);
    depth--;
    emit('),');
    if (bgColor != null || borderRadius != null || padding != null || color != null) {
      emit('style: ElevatedButton.styleFrom(');
      depth++;
      if (bgColor) emit(`backgroundColor: ${toColor(bgColor.value)},`);
      if (color) emit(`foregroundColor: ${toColor(color.value)},`);
      if (borderRadius) emit(`shape: RoundedRectangleBorder(borderRadius: ${toBorderRadius(borderRadius.value)}),`);
      if (padding) {
        const ep = toEdgeInsets(padding.value);
        if (ep) emit(`padding: ${ep},`);
      }
      depth--;
      emit('),');
    }
    depth--;
    emit('),');
    depth--;
  }

  function generateInput(attrs: Record<string, string>) {
    const placeholder = attrs.placeholder || '';

    depth++;
    emit('TextField(');
    depth++;
    if (placeholder) {
      emit(`decoration: const InputDecoration(hintText: '${escapeDart(placeholder)}'),`);
    }
    depth--;
    emit('),');
    depth--;
  }

  function generateTextArea(attrs: Record<string, string>) {
    const placeholder = attrs.placeholder || '';

    depth++;
    emit('TextField(');
    depth++;
    emit('maxLines: 5,');
    if (placeholder) {
      emit(`decoration: const InputDecoration(hintText: '${escapeDart(placeholder)}'),`);
    }
    depth--;
    emit('),');
    depth--;
  }

  function generateList(node: ASTNode, style: CSSDeclaration[]) {
    depth++;
    emit('Column(');
    depth++;
    emit('crossAxisAlignment: CrossAxisAlignment.start,');
    emit('children: [');
    depth++;
    for (const child of node.children || []) {
      if (child.type === 'element' && child.tagName === 'li') {
        generateListItem(child, style);
      }
    }
    depth--;
    emit('],');
    depth--;
    emit('),');
    depth--;
  }

  function generateListItem(node: ASTNode, _style: CSSDeclaration[]) {
    depth++;
    emit('Row(');
    depth++;
    emit('children: [');
    depth++;
    emit("const Text('• '),");
    emit(`Text('${escapeDart(getNodeText(node))}'),`);
    depth--;
    emit('],');
    depth--;
    emit('),');
    depth--;
  }

  function generateBold(node: ASTNode) {
    depth++;
    emit('Text(');
    depth++;
    emit(`'${escapeDart(getNodeText(node))}',`);
    emit('style: const TextStyle(fontWeight: FontWeight.bold),');
    depth--;
    emit('),');
    depth--;
  }

  function generateItalic(node: ASTNode) {
    depth++;
    emit('Text(');
    depth++;
    emit(`'${escapeDart(getNodeText(node))}',`);
    emit('style: const TextStyle(fontStyle: FontStyle.italic),');
    depth--;
    emit('),');
    depth--;
  }

  function generateCodeBlock(node: ASTNode) {
    depth++;
    emit('Container(');
    depth++;
    emit('padding: const EdgeInsets.all(8),');
    emit('decoration: BoxDecoration(color: Colors.grey.shade100, borderRadius: BorderRadius.circular(4)),');
    emit('child: SelectableText(');
    depth++;
    emit(`'${escapeDart(getNodeText(node))}',`);
    emit('style: TextStyle(fontFamily: \'monospace\', fontSize: 13),');
    depth--;
    emit('),');
    depth--;
    emit('),');
    depth--;
  }

  function generateTable(node: ASTNode, _style: CSSDeclaration[]) {
    depth++;
    emit('Table(');
    depth++;
    emit('border: TableBorder.all(),');
    emit('children: [');
    depth++;
    const rows = node.children || [];
    for (const child of rows) {
      if (child.type === 'element' && (child.tagName === 'tr' || child.tagName === 'thead' || child.tagName === 'tbody')) {
        const rowList = child.tagName === 'tr' ? [child] : (child.children || []);
        for (const row of rowList) {
          if (row.type === 'element' && row.tagName === 'tr') {
            emit('TableRow(');
            depth++;
            emit('children: [');
            depth++;
            for (const cell of row.children || []) {
              if (cell.type === 'element' && (cell.tagName === 'td' || cell.tagName === 'th')) {
                const isHeader = cell.tagName === 'th';
                emit(isHeader ? 'TableHeaderCell(child: ' : 'TableCell(child: ');
                depth++;
                emit(`Text('${escapeDart(getNodeText(cell))}',`);
                depth++;
                if (isHeader) emit('style: const TextStyle(fontWeight: FontWeight.bold),');
                depth--;
                emit('),');
                depth--;
                emit('),');
              }
            }
            depth--;
            emit('],');
            depth--;
            emit('),');
          }
        }
      }
    }
    depth--;
    emit('],');
    depth--;
    emit('),');
    depth--;
  }

  logger.info('FlutterGenerator', `Generated Flutter code (${buf.length} lines)`);
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

function escapeDart(s: string): string {
  return s
    .replace(/\\/g, '\\\\')
    .replace(/'/g, "\\'")
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t');
}

function toDouble(value: string): string {
  const num = parseFloat(value);
  if (isNaN(num)) return value;
  return `${num}`;
}

function toBorderRadius(value: string): string {
  const num = parseFloat(value);
  if (isNaN(num)) return `BorderRadius.circular(8)`;
  return `BorderRadius.circular(${num})`;
}

function toColor(value: string): string {
  const v = value.trim();
  if (v.startsWith('#')) {
    const hex = v.slice(1);
    if (hex.length === 3) {
      return `const Color(0xFF${hex[0]}${hex[0]}${hex[1]}${hex[1]}${hex[2]}${hex[2]})`;
    }
    if (hex.length === 6) return `const Color(0xFF${hex.toUpperCase()})`;
    if (hex.length === 8) return `const Color(0x${hex.toUpperCase()})`;
  }
  if (v.startsWith('rgb')) {
    const m = v.match(/\d+/g);
    if (m && m.length >= 3) return `const Color.fromRGBO(${m[0]}, ${m[1]}, ${m[2]}, 1)`;
  }
  const named: Record<string, string> = {
    red: 'Colors.red', blue: 'Colors.blue', green: 'Colors.green',
    white: 'Colors.white', black: 'Colors.black', gray: 'Colors.grey',
    grey: 'Colors.grey', yellow: 'Colors.yellow', orange: 'Colors.orange',
    purple: 'Colors.purple', pink: 'Colors.pink', transparent: 'Colors.transparent',
  };
  return named[v.toLowerCase()] || `const Color(0xFF6366F1)`;
}

function toEdgeInsets(value: string): string | null {
  const parts = value.split(/\s+/).filter(Boolean);
  const nums = parts.map(p => parseFloat(p)).filter(n => !isNaN(n));
  if (nums.length === 0) return null;
  if (nums.length === 1) return `const EdgeInsets.all(${nums[0]})`;
  if (nums.length === 2) return `const EdgeInsets.symmetric(horizontal: ${nums[1]}, vertical: ${nums[0]})`;
  if (nums.length === 4) return `const EdgeInsets.only(left: ${nums[3]}, top: ${nums[0]}, right: ${nums[1]}, bottom: ${nums[2]})`;
  return `const EdgeInsets.all(${nums[0]})`;
}

function crossAxisAlignment(value: string): string | null {
  const v = value.toLowerCase();
  if (v === 'center') return 'CrossAxisAlignment.center';
  if (v === 'flex-start' || v === 'start') return 'CrossAxisAlignment.start';
  if (v === 'flex-end' || v === 'end') return 'CrossAxisAlignment.end';
  if (v === 'stretch') return 'CrossAxisAlignment.stretch';
  return null;
}

function mainAxisAlignment(value: string): string | null {
  const v = value.toLowerCase();
  if (v === 'center') return 'MainAxisAlignment.center';
  if (v === 'flex-start' || v === 'start') return 'MainAxisAlignment.start';
  if (v === 'flex-end' || v === 'end') return 'MainAxisAlignment.end';
  if (v === 'space-between') return 'MainAxisAlignment.spaceBetween';
  if (v === 'space-around') return 'MainAxisAlignment.spaceAround';
  if (v === 'space-evenly') return 'MainAxisAlignment.spaceEvenly';
  return null;
}

function toTextAlign(value: string): string {
  const v = value.toLowerCase().trim();
  if (v === 'center') return 'TextAlign.center';
  if (v === 'right' || v === 'end') return 'TextAlign.right';
  return 'TextAlign.left';
}

function toFontWeight(value: string): string {
  const v = value.toLowerCase().trim();
  if (v === 'bold' || v === '700' || v === '800' || v === '900') return 'FontWeight.bold';
  if (v === 'normal' || v === '400') return 'FontWeight.normal';
  if (v === '300') return 'FontWeight.w300';
  if (v === '500') return 'FontWeight.w500';
  if (v === '600') return 'FontWeight.w600';
  return 'FontWeight.normal';
}
