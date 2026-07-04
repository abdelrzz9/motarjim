import type { HtmlNode, StyledNode, ResolvedStyles } from '@motarjim/shared';
import { computeStyle } from '@motarjim/css-analyzer';
import type {
  IrNode,
  SemanticIR,
  LayoutIR,
  ComputedStyle,
  AccessibilityMetadata,
  ResponsiveVariant,
  SourceSpan,
  HeadingLevel,
  ButtonVariant,
  InputType,
  ImageFit,
  ImageLoading,
  TextType,
  FlexDirection,
  FlexLayout,
  MainAxisAlignment,
  CrossAxisAlignment,
  FlexWrap,
  GridLayout,
  GridTrack,
  ScrollAxis,
} from '@motarjim/shared/ir-v2.js';
import {
  defaultAccessibility,
  emptySourceSpan,
  defaultComputedStyle,
  defaultTargetIR,
} from '@motarjim/shared/ir-v2.js';

// ============================================================
// Low-level: single HtmlNode → IrNode
//   (caller provides the node's own computed style)
// ============================================================

export interface TransformOptions {
  computedStyle?: ComputedStyle;
  responsiveVariants?: ResponsiveVariant[];
}

export function htmlNodeToIr(
  node: HtmlNode,
  options?: TransformOptions,
): IrNode {
  const cs = options?.computedStyle ?? defaultComputedStyle();
  const rvs = options?.responsiveVariants ?? [];

  return {
    id: node.nodeId,
    sourceSpan: node.sourceSpan ?? emptySourceSpan(),
    accessibility: extractAccessibility(node),
    computedStyle: cs,
    responsiveVariants: rvs,
    semantics: inferSemantics(node, cs),
    layout: inferLayout(node, cs),
    target: defaultTargetIR(),
    children: transformChildren(node),
  };
}

function transformChildren(parent: HtmlNode): IrNode[] {
  return parent.children
    .filter(child => !shouldSkipTextChild(child, parent))
    .map(child => {
      if (child.tagName === '#text') {
        return textNodeToIr(child);
      }
      return htmlNodeToIr(child);
    });
}

function shouldSkipTextChild(child: HtmlNode, parent: HtmlNode): boolean {
  if (child.tagName !== '#text') return false;
  return TEXT_CONTAINER_TAGS.has(parent.tagName);
}

const TEXT_CONTAINER_TAGS = new Set([
  'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  'p', 'span', 'a', 'label',
  'figcaption', 'caption', 'li',
  'hgroup', 'title',
]);

function textNodeToIr(node: HtmlNode): IrNode {
  const content = node.value ?? '';
  return {
    id: node.nodeId,
    sourceSpan: node.sourceSpan ?? emptySourceSpan(),
    accessibility: defaultAccessibility(),
    computedStyle: defaultComputedStyle(),
    responsiveVariants: [],
    semantics: { role: 'text', content, textType: 'span' },
    layout: { strategy: 'none' },
    target: defaultTargetIR(),
    children: [],
  };
}

// ============================================================
// High-level: StyledNode tree → IrNode tree
//   (properly resolves each node's own styles via cascade)
// ============================================================

export function styledNodeToIrV2(
  styled: StyledNode,
  responsiveVariants?: ResponsiveVariant[],
): IrNode {
  const cs = computeStyle(styled.styles);
  const rvs = responsiveVariants ?? [];

  return {
    id: styled.node.nodeId,
    sourceSpan: styled.node.sourceSpan ?? emptySourceSpan(),
    accessibility: extractAccessibility(styled.node),
    computedStyle: cs,
    responsiveVariants: rvs,
    semantics: inferSemantics(styled.node, cs),
    layout: inferLayout(styled.node, cs),
    target: defaultTargetIR(),
    children: styledChildrenToIr(styled, responsiveVariants),
  };
}

function styledChildrenToIr(
  parent: StyledNode,
  responsiveVariants?: ResponsiveVariant[],
): IrNode[] {
  return parent.children
    .filter(child => !styledShouldSkipTextChild(child, parent))
    .map(child => styledNodeToIrV2(child, responsiveVariants));
}

function styledShouldSkipTextChild(child: StyledNode, parent: StyledNode): boolean {
  if (child.node.tagName !== '#text') return false;
  return TEXT_CONTAINER_TAGS.has(parent.node.tagName);
}

// computeV2Style is now delegated to @motarjim/css-analyzer's computeStyle
// which uses the shared CSS mapper modules (spacing-mapper, typography-mapper, etc.)

// ============================================================
// Accessibility extraction
// ============================================================

function extractAccessibility(node: HtmlNode): AccessibilityMetadata {
  const attrs = node.attributes;
  const get = (name: string) => attrs.find(a => a.name === name)?.value;
  const has = (name: string) => attrs.some(a => a.name === name);

  return {
    label: get('aria-label') ?? get('alt') ?? undefined,
    hint: get('aria-describedby') ?? get('title') ?? undefined,
    role: get('role') ?? undefined,
    hidden: has('hidden') || get('aria-hidden') === 'true',
    focusable: has('tabindex') && get('tabindex') !== '-1',
    liveRegion: get('aria-live') as AccessibilityMetadata['liveRegion'] ?? undefined,
  };
}

// ============================================================
// Semantic inference
// ============================================================

function inferSemantics(node: HtmlNode, _cs: ComputedStyle): SemanticIR {
  const tag = node.tagName;
  const attrs = node.attributes;
  const get = (name: string) => attrs.find(a => a.name === name)?.value;

  switch (tag) {
    // -- Interactive --
    case 'button':
      return {
        role: 'button',
        label: get('aria-label') ?? getValue(node) ?? '',
        variant: inferButtonVariant(node),
        disabled: attrs.some(a => a.name === 'disabled'),
        loading: false,
        pressed: get('aria-pressed') === 'true',
      };

    case 'a':
      return {
        role: 'link',
        href: get('href') ?? '#',
        target: get('target') as '_self' | '_blank' | '_parent' | '_top' | undefined,
        rel: get('rel') ?? undefined,
        download: get('download') ?? undefined,
        label: get('aria-label') ?? getValue(node) ?? get('href') ?? '',
      };

    // -- Form controls --
    case 'input': {
      const inputType = (get('type') ?? 'text') as InputType;
      return {
        role: 'input',
        inputType,
        name: get('name') ?? undefined,
        placeholder: get('placeholder') ?? undefined,
        value: get('value') ?? undefined,
        defaultValue: undefined,
        required: attrs.some(a => a.name === 'required'),
        disabled: attrs.some(a => a.name === 'disabled'),
        readonly: attrs.some(a => a.name === 'readonly'),
      };
    }

    case 'textarea':
      return {
        role: 'textarea',
        name: get('name') ?? undefined,
        placeholder: get('placeholder') ?? undefined,
        value: get('value') ?? node.value ?? undefined,
        defaultValue: undefined,
        required: attrs.some(a => a.name === 'required'),
        disabled: attrs.some(a => a.name === 'disabled'),
        readonly: attrs.some(a => a.name === 'readonly'),
        rows: parseInt(get('rows') ?? '4', 10),
        cols: parseInt(get('cols') ?? '20', 10),
      };

    case 'select':
      return {
        role: 'select',
        name: get('name') ?? undefined,
        multiple: attrs.some(a => a.name === 'multiple'),
        required: attrs.some(a => a.name === 'required'),
        disabled: attrs.some(a => a.name === 'disabled'),
        options: [],
      };

    case 'form':
      return {
        role: 'form',
        action: get('action') ?? undefined,
        method: get('method') as 'get' | 'post' | 'dialog' | undefined,
        name: get('name') ?? undefined,
        autoComplete: get('autocomplete') !== 'off',
        fields: [],
      };

    // -- Typography --
    case 'h1': case 'h2': case 'h3': case 'h4': case 'h5': case 'h6':
      return {
        role: 'heading',
        content: getValue(node) ?? '',
        level: parseInt(tag.charAt(1), 10) as HeadingLevel,
      };

    case 'p':
      return {
        role: 'text',
        content: getValue(node) ?? '',
        textType: 'paragraph',
      };

    case 'span':
      return {
        role: 'text',
        content: getValue(node) ?? '',
        textType: 'span',
      };

    case 'label':
      return {
        role: 'text',
        content: getValue(node) ?? '',
        textType: 'label',
        overflow: 'ellipsis',
      };

    case 'figcaption':
    case 'caption':
      return {
        role: 'text',
        content: getValue(node) ?? '',
        textType: 'caption',
      };

    // -- Media --
    case 'img':
      return {
        role: 'image',
        source: get('src') ?? '',
        alt: get('alt') ?? '',
        fit: 'contain' as ImageFit,
        loading: (get('loading') as ImageLoading) ?? 'eager',
        aspectRatio: undefined,
      };

    case 'svg':
      return {
        role: 'icon',
        name: get('name') ?? get('aria-label') ?? 'unknown',
        size: 24,
        color: undefined,
      };

    // -- Lists --
    case 'ul':
      return { role: 'list', ordered: false, items: [], listStyle: 'disc' as const, reversed: false };

    case 'ol':
      return {
        role: 'list',
        ordered: true,
        items: [],
        listStyle: 'decimal' as const,
        start: get('start') ? parseInt(get('start')!, 10) : undefined,
        reversed: attrs.some(a => a.name === 'reversed'),
      };

    case 'li':
      return {
        role: 'list-item',
        label: getValue(node) ?? '',
        value: get('value') ? parseInt(get('value')!, 10) : undefined,
      };

    // -- Tables --
    case 'table':
      return { role: 'table', columns: [], rows: [] };

    // -- Dividers --
    case 'hr':
      return { role: 'divider', orientation: 'horizontal' as const };

    // -- Landmarks --
    case 'nav':
      return { role: 'nav', label: get('aria-label') ?? get('title') ?? undefined };

    case 'header':
      return { role: 'header', label: get('aria-label') ?? undefined };

    case 'footer':
      return { role: 'footer', label: get('aria-label') ?? undefined };

    case 'section':
      return { role: 'section', label: get('aria-label') ?? undefined };

    case 'article':
      return { role: 'article', label: get('aria-label') ?? undefined };

    case 'aside':
      return { role: 'section', label: 'sidebar' };

    // -- Dialog --
    case 'dialog':
      return {
        role: 'dialog',
        modal: !attrs.some(a => a.name === 'open'),
        dismissable: true,
      };

    // -- Container-like --
    case 'div':
    case 'main':
      return { role: 'generic' };

    // -- Unknown --
    default:
      if (tag === '#text') {
        return { role: 'text', content: node.value ?? '', textType: 'span' };
      }
      return { role: 'unknown', originalTag: tag };
  }
}

// ============================================================
// Layout inference
// ============================================================

function inferLayout(node: HtmlNode, cs: ComputedStyle): LayoutIR {
  const tag = node.tagName;

  // Leaf semantic nodes have no layout
  if (isLeafTag(tag)) {
    return { strategy: 'none' };
  }

  const display = cs.display;

  // Flex layout (covers flex, inline-flex)
  if (display === 'flex' || display === 'inline-flex') {
    return inferFlexLayout(cs);
  }

  // Grid layout
  if (display === 'grid' || display === 'inline-grid') {
    return inferGridLayout(cs);
  }

  // Interactive elements with children use flex layout implicitly
  if ((tag === 'button' || tag === 'a') && node.children.length > 0) {
    return inferFlexLayout(cs, 'row', 'center');
  }

  // Form is usually column flex
  if (tag === 'form') {
    return inferFlexLayout(cs, 'column');
  }

  // Scroll detection
  if (
    cs.overflowX === 'scroll' || cs.overflowY === 'scroll' ||
    cs.overflowX === 'auto' || cs.overflowY === 'auto'
  ) {
    return {
      strategy: 'scroll',
      axis: cs.overflowY === 'scroll' || cs.overflowY === 'auto' ? 'vertical' : 'horizontal',
      showScrollbar: cs.overflowX === 'scroll' || cs.overflowY === 'scroll',
      scrollBehavior: 'auto',
      pagingEnabled: false,
    };
  }

  // Absolute positioning
  if (cs.position === 'absolute' || cs.position === 'fixed') {
    return {
      strategy: 'absolute',
      position: {
        top: cs.top,
        right: cs.right,
        bottom: cs.bottom,
        left: cs.left,
      },
      zIndex: cs.zIndex ?? 0,
    };
  }

  // Default: box layout
  return { strategy: 'box', sizing: inferSizing(cs) };
}

// ============================================================
// Helper: flex layout inference
// ============================================================

function inferFlexLayout(
  cs: ComputedStyle,
  defaultDirection?: FlexDirection,
  defaultAlign?: CrossAxisAlignment,
): FlexLayout {
  const direction = (cs.flexDirection ?? defaultDirection ?? 'column') as FlexDirection;
  return {
    strategy: 'flex',
    direction,
    wrap: (cs.flexWrap as FlexWrap) ?? 'nowrap',
    justifyContent: (cs.justifyContent as MainAxisAlignment) ?? 'start',
    alignItems: (cs.alignItems as CrossAxisAlignment) ?? (defaultAlign ?? 'stretch'),
    alignContent: (cs.alignContent as CrossAxisAlignment) ?? 'start',
    gap: cs.gap ?? 0,
    flexGrow: cs.flexGrow,
    flexShrink: cs.flexShrink,
    flexBasis: cs.flexBasis,
  };
}

// ============================================================
// Helper: grid layout inference
// ============================================================

function inferGridLayout(cs: ComputedStyle): GridLayout {
  const parseTracks = (raw?: string): GridTrack[] => {
    if (!raw) return [];
    return raw.split(/\s+/).filter(Boolean).map(size => ({ size }));
  };

  return {
    strategy: 'grid',
    columns: parseTracks(cs.gridColumn ?? '1fr'),
    rows: parseTracks(cs.gridRow),
    gap: { row: cs.gap ?? 0, column: cs.gap ?? 0 },
    autoFlow: 'row',
  };
}

// ============================================================
// Helper: sizing inference
// ============================================================

function inferSizing(cs: ComputedStyle): 'fit-content' | 'fill' | 'fixed' {
  if (cs.width === 'auto' || (cs.width == null && cs.height == null)) {
    return 'fit-content';
  }
  if (cs.width === '100%' || cs.width?.includes('vw') || cs.width?.includes('vh')) {
    return 'fill';
  }
  return 'fixed';
}

// ============================================================
// Helpers
// ============================================================

function getValue(node: HtmlNode): string | undefined {
  if (node.value) return node.value;
  const textChild = node.children.find(c => c.tagName === '#text');
  return textChild?.value;
}

function inferButtonVariant(node: HtmlNode): ButtonVariant {
  const get = (name: string) => node.attributes.find(a => a.name === name)?.value;
  const classes = (get('class') ?? '').split(/\s+/);
  if (classes.some(c => /outline/i.test(c))) return 'outlined';
  if (classes.some(c => /text|ghost|link/i.test(c))) return 'text';
  if (node.children.every(c => c.tagName === 'svg' || c.tagName === 'img')) return 'icon';
  return 'filled';
}

const LEAF_TAGS = new Set([
  '#text', 'img', 'input', 'textarea', 'select',
  'hr', 'br', 'svg',
]);

function isLeafTag(tag: string): boolean {
  return LEAF_TAGS.has(tag);
}
