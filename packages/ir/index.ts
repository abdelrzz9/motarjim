import { UiNode, StyledNode, SemanticHint } from '@html-native/shared';

export { UiNode } from '@html-native/shared';

export function createIrNode(
  type: UiNode['type'],
  properties: Record<string, unknown> = {},
  children: UiNode[] = [],
  value?: string,
): UiNode {
  return { type, properties, children, value };
}

export function styledNodeToIr(styled: StyledNode, hints: SemanticHint[] = []): UiNode {
  const hasClass = (name: string) =>
    styled.node.attributes.some(a => a.name === 'class' && a.value.split(/\s+/).includes(name));

  let type: UiNode['type'] = 'Unknown';
  const props: Record<string, unknown> = {};
  const tag = styled.node.tagName;

  const hint = hints.find(h => h.node.nodeId === styled.node.nodeId);
  if (hint && hint.confidence > 0.5) {
    type = hint.type;
  } else {
    type = inferType(tag, styled, hasClass);
  }

  Object.assign(props, extractProps(styled));

  const nodeValue = (props.value as string) || undefined;
  delete props.value;

  // For text-like elements (h1-h6, p, span, #text), extract text from child #text nodes
  const textTags = new Set(['h1','h2','h3','h4','h5','h6','p','span','a','label','#text']);
  let effectiveValue = nodeValue;

  if (!effectiveValue && textTags.has(tag)) {
    const textParts: string[] = [];
    for (const child of styled.children) {
      if (child.node.tagName === '#text' && child.node.value) {
        textParts.push(child.node.value);
      }
    }
    if (textParts.length > 0) {
      effectiveValue = textParts.join(' ');
    }
  }

  // Build children, skipping #text nodes whose content was absorbed
  const children = styled.children
    .filter(c => {
      if (effectiveValue && textTags.has(tag) && c.node.tagName === '#text') return false;
      return true;
    })
    .map(c => styledNodeToIr(c, hints));

  return createIrNode(type, props, children, effectiveValue);
}

function inferType(
  tag: string,
  styled: StyledNode,
  hasClass: (name: string) => boolean,
): UiNode['type'] {
  const map: Record<string, UiNode['type']> = {
    div: 'Container',
    span: 'Container',
    h1: 'Text',
    h2: 'Text',
    h3: 'Text',
    h4: 'Text',
    h5: 'Text',
    h6: 'Text',
    p: 'Text',
    button: 'Button',
    img: 'Image',
    input: 'TextField',
    textarea: 'TextArea',
    form: 'Form',
    ul: 'UnorderedList',
    ol: 'OrderedList',
    li: 'ListItem',
    nav: 'Nav',
    header: 'Header',
    footer: 'Footer',
    section: 'Section',
    article: 'Article',
    a: 'Link',
    svg: 'Svg',
  };

  if (tag in map) return map[tag];
  if (tag === '#text') return 'Text';

  const display = styled.styles['display'];
  const flexDirection = styled.styles['flex-direction'];

  if (display === 'flex') {
    if (flexDirection === 'column' || !flexDirection) return 'Column';
    return 'Row';
  }
  if (display === 'grid') return 'Grid';

  return 'Container';
}

function extractProps(styled: StyledNode): Record<string, unknown> {
  const props: Record<string, unknown> = {};
  const s = styled.styles;

  if (s['padding']) props.padding = s['padding'];
  if (s['margin']) props.margin = s['margin'];
  if (s['gap']) props.gap = s['gap'];
  if (s['width']) props.width = s['width'];
  if (s['height']) props.height = s['height'];
  if (s['min-width']) props.minWidth = s['min-width'];
  if (s['max-width']) props.maxWidth = s['max-width'];
  if (s['font-size']) props.fontSize = s['font-size'];
  if (s['font-weight']) props.fontWeight = s['font-weight'];
  if (s['line-height']) props.lineHeight = s['line-height'];
  if (s['text-align']) props.textAlign = s['text-align'];
  if (s['color']) props.color = s['color'];
  if (s['background']) props.background = s['background'];
  if (s['border']) props.border = s['border'];
  if (s['border-radius']) props.borderRadius = s['border-radius'];
  if (s['box-shadow']) props.boxShadow = s['box-shadow'];
  if (s['position']) props.position = s['position'];

  if (styled.node.value) {
    props.value = styled.node.value;
  }

  const srcAttr = styled.node.attributes.find(a => a.name === 'src');
  if (srcAttr) props.src = srcAttr.value;

  const altAttr = styled.node.attributes.find(a => a.name === 'alt');
  if (altAttr) props.alt = altAttr.value;

  const hrefAttr = styled.node.attributes.find(a => a.name === 'href');
  if (hrefAttr) props.href = hrefAttr.value;

  const htmlValueAttr = styled.node.attributes.find(a => a.name === 'value');
  if (htmlValueAttr && styled.node.tagName === 'input') {
    props.value = htmlValueAttr.value;
  }

  return props;
}
