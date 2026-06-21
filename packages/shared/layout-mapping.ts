// ============================================================
// Cross-Platform Layout Mapping Strategies
// ============================================================
//
// Maps every layout primitive and alignment value from the
// platform-neutral layout system into platform-specific
// widgets, modifiers, and property values for:
//   - Flutter  (Dart)
//   - Compose  (Kotlin)
//   - SwiftUI  (Swift)
// ============================================================

import type {
  ContainerLayout,
  Flex,
  Stack,
  Scroll,
  Direction,
  MainAxisAlignment,
  CrossAxisAlignment,
  Alignment2D,
  Gap,
  WrapMode,
  StackFit,
  Positioned,
  ChildLayout,
  Constraints,
  Axis,
  EdgeInsets,
} from './layout.js';

// ============================================================
// 1.  LayoutMapping Interface
// ============================================================

export interface LayoutMapping {
  platform: 'flutter' | 'compose' | 'swiftui';

  /** The platform widget name for a container layout. */
  containerName(layout: ContainerLayout): string;

  /** Serialize main-axis alignment to platform enum. */
  mainAxisAlignment(value: MainAxisAlignment): string;

  /** Serialize cross-axis alignment to platform enum. */
  crossAxisAlignment(value: CrossAxisAlignment): string;

  /** Serialize 2D alignment to platform literal. */
  alignment2D(value: Alignment2D): string;

  /** Serialize gap to platform expression. */
  gapValue(gap: Gap): string;

  /** Generates platform modifier/widget for sizing constraints. */
  constraintsSnippet(constraints: Constraints): string;

  /** Wraps a child with its per-child layout attributes. */
  wrapChild(child: string, attrs: ChildLayout): string;

  /** Wraps a child with absolute positioning. */
  wrapPositioned(child: string, position: Positioned): string;

  /** Wraps children array in container body. */
  wrapChildren(children: string[], layout: ContainerLayout): string;

  /** Platform scroll axis modifier. */
  scrollAxisCode(axis: Axis): string;
}

// ============================================================
// 2.  Shared helpers
// ============================================================

function gapToNumber(gap: Gap): string {
  switch (gap.kind) {
    case 'none': return '0';
    case 'fixed': return String(gap.value);
    case 'responsive': return String(gap.min);
  }
}

function px(n: number): string {
  return n % 1 === 0 ? String(n) : n.toFixed(1);
}

// ============================================================
// 3.  Flutter Mapping
// ============================================================

export const flutterLayoutMapping: LayoutMapping = {
  platform: 'flutter',

  containerName(layout: ContainerLayout): string {
    switch (layout.kind) {
      case 'flex':
        return (layout as Flex).direction.startsWith('row') ? 'Row' : 'Column';
      case 'stack':
        return 'Stack';
      case 'scroll':
        return 'SingleChildScrollView';
    }
  },

  mainAxisAlignment(value: MainAxisAlignment): string {
    const map: Record<MainAxisAlignment, string> = {
      start: 'MainAxisAlignment.start',
      center: 'MainAxisAlignment.center',
      end: 'MainAxisAlignment.end',
      'space-between': 'MainAxisAlignment.spaceBetween',
      'space-around': 'MainAxisAlignment.spaceAround',
      'space-evenly': 'MainAxisAlignment.spaceEvenly',
    };
    return map[value];
  },

  crossAxisAlignment(value: CrossAxisAlignment): string {
    const map: Record<CrossAxisAlignment, string> = {
      start: 'CrossAxisAlignment.start',
      center: 'CrossAxisAlignment.center',
      end: 'CrossAxisAlignment.end',
      stretch: 'CrossAxisAlignment.stretch',
      baseline: 'CrossAxisAlignment.baseline',
    };
    return map[value];
  },

  alignment2D(value: Alignment2D): string {
    const { x, y } = value;
    if (x === 'center' && y === 'center') return 'Alignment.center';
    return `Alignment(${flutterAlignX(x)}, ${flutterAlignY(y)})`;
  },

  gapValue(gap: Gap): string {
    return gapToNumber(gap);
  },

  constraintsSnippet(constraints: Constraints): string {
    const parts: string[] = [];
    if (constraints.minWidth !== undefined) parts.push('minWidth: ' + px(constraints.minWidth));
    if (constraints.maxWidth !== undefined) parts.push('maxWidth: ' + px(constraints.maxWidth));
    if (constraints.minHeight !== undefined) parts.push('minHeight: ' + px(constraints.minHeight));
    if (constraints.maxHeight !== undefined) parts.push('maxHeight: ' + px(constraints.maxHeight));
    if (parts.length === 0) return '';
    return 'BoxConstraints(' + parts.join(', ') + ')';
  },

  wrapChild(child: string, attrs: ChildLayout): string {
    let result = child;

    if (attrs.flexGrow !== undefined || attrs.flexShrink !== undefined) {
      const g = attrs.flexGrow ?? 0;
      const s = attrs.flexShrink ?? 1;
      result = 'Expanded(flex: ' + g + ', child: ' + result + ')';
    }

    if (attrs.alignSelf && attrs.alignSelf !== 'stretch') {
      const align = this.crossAxisAlignment(attrs.alignSelf);
      result = 'Align(alignment: ' + align + ', child: ' + result + ')';
    }

    if (attrs.constraints) {
      const c = this.constraintsSnippet(attrs.constraints);
      if (c) result = 'ConstrainedBox(constraints: ' + c + ', child: ' + result + ')';
    }

    if (attrs.margin) {
      result = 'Padding(padding: ' + flutterEdgeInsets(attrs.margin) + ', child: ' + result + ')';
    }

    if (attrs.padding) {
      result = 'Padding(padding: ' + flutterEdgeInsets(attrs.padding) + ', child: ' + result + ')';
    }

    return result;
  },

  wrapPositioned(child: string, position: Positioned): string {
    const params: string[] = [];
    if (position.top !== undefined) params.push('top: ' + px(position.top));
    if (position.right !== undefined) params.push('right: ' + px(position.right));
    if (position.bottom !== undefined) params.push('bottom: ' + px(position.bottom));
    if (position.left !== undefined) params.push('left: ' + px(position.left));
    if (position.width !== undefined) params.push('width: ' + px(position.width));
    if (position.height !== undefined) params.push('height: ' + px(position.height));
    if (params.length === 1) return 'Positioned(' + params[0] + ', child: ' + child + ')';
    return 'Positioned(' + params.join(', ') + ', child: ' + child + ')';
  },

  wrapChildren(children: string[], _layout: ContainerLayout): string {
    if (children.length === 0) return '[]';
    return '[\n' + children.map(c => indent(c, 2)).join(',\n') + ',\n]';
  },

  scrollAxisCode(axis: Axis): string {
    if (axis === 'both') return 'Axis.horizontal';
    return 'Axis.' + axis;
  },
};

function flutterAlignX(x: string): string {
  return x === 'start' ? '-1.0' : x === 'end' ? '1.0' : '0.0';
}

function flutterAlignY(y: string): string {
  return y === 'start' ? '-1.0' : y === 'end' ? '1.0' : '0.0';
}

function flutterEdgeInsets(e: EdgeInsets): string {
  return 'EdgeInsets.only(' +
    'top: ' + e.top + ', ' +
    'right: ' + e.right + ', ' +
    'bottom: ' + e.bottom + ', ' +
    'left: ' + e.left +
  ')';
}

// ============================================================
// 4.  Compose Mapping
// ============================================================

export const composeLayoutMapping: LayoutMapping = {
  platform: 'compose',

  containerName(layout: ContainerLayout): string {
    switch (layout.kind) {
      case 'flex':
        return (layout as Flex).direction.startsWith('row') ? 'Row' : 'Column';
      case 'stack':
        return 'Box';
      case 'scroll':
        return 'LazyColumn';
    }
  },

  mainAxisAlignment(value: MainAxisAlignment): string {
    const map: Record<MainAxisAlignment, string> = {
      start: 'Arrangement.Start',
      center: 'Arrangement.Center',
      end: 'Arrangement.End',
      'space-between': 'Arrangement.SpaceBetween',
      'space-around': 'Arrangement.SpaceAround',
      'space-evenly': 'Arrangement.SpaceEvenly',
    };
    return map[value];
  },

  crossAxisAlignment(value: CrossAxisAlignment): string {
    const map: Record<CrossAxisAlignment, string> = {
      start: 'Alignment.TopStart',
      center: 'Alignment.CenterStart',
      end: 'Alignment.BottomStart',
      stretch: 'Alignment.CenterStart',
      baseline: 'Alignment.CenterStart',
    };
    return map[value];
  },

  alignment2D(value: Alignment2D): string {
    const { x, y } = value;
    if (x === 'center' && y === 'center') return 'Alignment.Center';
    return 'Alignment(' + composeAlign(x) + ', ' + composeAlign(y) + ')';
  },

  gapValue(gap: Gap): string {
    return gapToNumber(gap) + '.dp';
  },

  constraintsSnippet(constraints: Constraints): string {
    const c = constraints;
    if (c.minWidth === undefined && c.maxWidth === undefined &&
        c.minHeight === undefined && c.maxHeight === undefined) {
      return '';
    }
    if (c.minWidth !== undefined && c.maxWidth !== undefined &&
        c.minHeight === undefined && c.maxHeight === undefined) {
      return 'Modifier.widthIn(' + px(c.minWidth) + '.dp, ' + px(c.maxWidth) + '.dp)';
    }
    if (c.minHeight !== undefined && c.maxHeight !== undefined &&
        c.minWidth === undefined && c.maxWidth === undefined) {
      return 'Modifier.heightIn(' + px(c.minHeight) + '.dp, ' + px(c.maxHeight) + '.dp)';
    }
    const parts: string[] = [];
    if (c.minWidth !== undefined) parts.push('minWidth = ' + px(c.minWidth) + '.dp');
    if (c.maxWidth !== undefined) parts.push('maxWidth = ' + px(c.maxWidth) + '.dp');
    if (c.minHeight !== undefined) parts.push('minHeight = ' + px(c.minHeight) + '.dp');
    if (c.maxHeight !== undefined) parts.push('maxHeight = ' + px(c.maxHeight) + '.dp');
    return 'Modifier.sizeIn(' + parts.join(', ') + ')';
  },

  wrapChild(child: string, attrs: ChildLayout): string {
    let result = child;

    if (attrs.flexGrow !== undefined) {
      result = 'Box(modifier = Modifier.weight(' + attrs.flexGrow + 'f)) {\n' + indent(result, 1) + '\n}';
    }

    if (attrs.alignSelf && attrs.alignSelf !== 'stretch') {
      result = 'Box(modifier = Modifier.align(' + this.crossAxisAlignment(attrs.alignSelf) + ')) {\n' + indent(result, 1) + '\n}';
    }

    if (attrs.constraints) {
      const c = this.constraintsSnippet(attrs.constraints);
      if (c) result = 'Box(modifier = ' + c + ') {\n' + indent(result, 1) + '\n}';
    }

    if (attrs.margin) {
      result = 'Box(modifier = Modifier.padding(' + composeInsets(attrs.margin) + ')) {\n' + indent(result, 1) + '\n}';
    }

    if (attrs.padding) {
      result = 'Box(modifier = Modifier.padding(' + composeInsets(attrs.padding) + ')) {\n' + indent(result, 1) + '\n}';
    }

    return result;
  },

  wrapPositioned(child: string, position: Positioned): string {
    const mods: string[] = [];
    if (position.top !== undefined || position.left !== undefined) {
      mods.push('Modifier.offset(x = ' + px(position.left ?? 0) + '.dp, y = ' + px(position.top ?? 0) + '.dp)');
    }
    if (position.width !== undefined || position.height !== undefined) {
      const w = position.width !== undefined ? px(position.width) + '.dp' : 'IntrinsicSize.Min';
      const h = position.height !== undefined ? px(position.height) + '.dp' : 'IntrinsicSize.Min';
      mods.push('Modifier.size(width = ' + w + ', height = ' + h + ')');
    }
    if (mods.length === 0) return child;
    return 'Box(modifier = ' + mods.join('.\n  ') + ') {\n' + indent(child, 1) + '\n}';
  },

  wrapChildren(children: string[], _layout: ContainerLayout): string {
    return children.join('\n');
  },

  scrollAxisCode(axis: Axis): string {
    if (axis === 'vertical') return 'Modifier.verticalScroll()';
    if (axis === 'horizontal') return 'Modifier.horizontalScroll()';
    return 'Modifier.horizontalScroll(rememberScrollState())';
  },
};

function composeAlign(v: string): string {
  return v === 'start' ? '-1f' : v === 'end' ? '1f' : '0f';
}

function composeInsets(e: EdgeInsets): string {
  return px(e.top) + '.dp, ' + px(e.right) + '.dp, ' + px(e.bottom) + '.dp, ' + px(e.left) + '.dp';
}

// ============================================================
// 5.  SwiftUI Mapping
// ============================================================

export const swiftuiLayoutMapping: LayoutMapping = {
  platform: 'swiftui',

  containerName(layout: ContainerLayout): string {
    switch (layout.kind) {
      case 'flex':
        return (layout as Flex).direction.startsWith('row') ? 'HStack' : 'VStack';
      case 'stack':
        return 'ZStack';
      case 'scroll':
        return 'ScrollView';
    }
  },

  mainAxisAlignment(value: MainAxisAlignment): string {
    const map: Record<MainAxisAlignment, string> = {
      start: '.leading',
      center: '.center',
      end: '.trailing',
      'space-between': '.center',
      'space-around': '.center',
      'space-evenly': '.center',
    };
    return map[value];
  },

  crossAxisAlignment(value: CrossAxisAlignment): string {
    const map: Record<CrossAxisAlignment, string> = {
      start: '.top',
      center: '.center',
      end: '.bottom',
      stretch: '.center',
      baseline: '.firstTextBaseline',
    };
    return map[value];
  },

  alignment2D(value: Alignment2D): string {
    const { x, y } = value;
    if (x === 'center' && y === 'center') return '.center';
    return 'Alignment(horizontal: ' + swiftuiAlignX(x) + ', vertical: ' + swiftuiAlignY(y) + ')';
  },

  gapValue(gap: Gap): string {
    return px(Number(gapToNumber(gap)));
  },

  constraintsSnippet(constraints: Constraints): string {
    const c = constraints;
    if (c.minWidth === undefined && c.maxWidth === undefined &&
        c.minHeight === undefined && c.maxHeight === undefined) {
      return '';
    }
    const minW = c.minWidth !== undefined ? px(c.minWidth) : 'nil';
    const maxW = c.maxWidth !== undefined ? px(c.maxWidth) : 'nil';
    const minH = c.minHeight !== undefined ? px(c.minHeight) : 'nil';
    const maxH = c.maxHeight !== undefined ? px(c.maxHeight) : 'nil';
    return '.frame(minWidth: ' + minW + ', maxWidth: ' + maxW + ', minHeight: ' + minH + ', maxHeight: ' + maxH + ')';
  },

  wrapChild(child: string, attrs: ChildLayout): string {
    let result = child;

    if (attrs.flexGrow !== undefined) {
      result = result + '\n.frame(maxWidth: .infinity)';
    }

    if (attrs.alignSelf) {
      result = result + '\n.frame(alignment: ' + this.crossAxisAlignment(attrs.alignSelf) + ')';
    }

    if (attrs.constraints) {
      const c = this.constraintsSnippet(attrs.constraints);
      if (c) result = result + '\n' + c;
    }

    if (attrs.margin) {
      result = result + '\n.padding(' + swiftUIEdgeInsets(attrs.margin) + ')';
    }

    if (attrs.padding) {
      result = result + '\n.padding(' + swiftUIEdgeInsets(attrs.padding) + ')';
    }

    return result;
  },

  wrapPositioned(child: string, position: Positioned): string {
    let result = child;
    if (position.top !== undefined || position.left !== undefined) {
      result = result + '\n.position(x: ' + px(position.left ?? 0) + ', y: ' + px(position.top ?? 0) + ')';
    }
    if (position.width !== undefined || position.height !== undefined) {
      const w = position.width !== undefined ? px(position.width) : 'nil';
      const h = position.height !== undefined ? px(position.height) : 'nil';
      result = result + '\n.frame(width: ' + w + ', height: ' + h + ')';
    }
    return result;
  },

  wrapChildren(children: string[], _layout: ContainerLayout): string {
    return children.join('\n');
  },

  scrollAxisCode(axis: Axis): string {
    if (axis === 'vertical') return '.vertical';
    if (axis === 'horizontal') return '.horizontal';
    return '[.vertical, .horizontal]';
  },
};

function swiftuiAlignX(x: string): string {
  return x === 'center' ? '.center' : x === 'start' ? '.leading' : '.trailing';
}

function swiftuiAlignY(y: string): string {
  return y === 'center' ? '.center' : y === 'start' ? '.top' : '.bottom';
}

function swiftUIEdgeInsets(e: EdgeInsets): string {
  return '.init(top: ' + px(e.top) + ', leading: ' + px(e.left) +
    ', bottom: ' + px(e.bottom) + ', trailing: ' + px(e.right) + ')';
}

// ============================================================
// 6.  Helpers
// ============================================================

function indent(text: string, level: number): string {
  const pad = '  '.repeat(level);
  return text.split('\n').map(line => pad + line).join('\n');
}

// ============================================================
// 7.  Lookup
// ============================================================

export type PlatformKind = 'flutter' | 'compose' | 'swiftui';

const _mappings: Record<PlatformKind, LayoutMapping> = {
  flutter: flutterLayoutMapping,
  compose: composeLayoutMapping,
  swiftui: swiftuiLayoutMapping,
};

export function getLayoutMapping(platform: PlatformKind): LayoutMapping {
  return _mappings[platform];
}

// ============================================================
// 8.  Example Generators (for documentation / template codegen)
// ============================================================

export function layoutExample(layout: ContainerLayout, platform: PlatformKind): string {
  const m = getLayoutMapping(platform);
  const name = m.containerName(layout);

  switch (layout.kind) {
    case 'flex': {
      const flex = layout as Flex;
      const mainA = m.mainAxisAlignment(flex.mainAxisAlignment);
      const crossA = m.crossAxisAlignment(flex.crossAxisAlignment);
      const gap = m.gapValue(flex.gap);

      switch (platform) {
        case 'flutter':
          return name + '(\n  mainAxisAlignment: ' + mainA + ',\n  crossAxisAlignment: ' + crossA + ',\n  children: [ /* ... */ ],\n)';
        case 'compose':
          return name + '(\n  horizontalArrangement: ' + mainA + ',\n  verticalAlignment: ' + crossA + ',\n) { /* ... */ }';
        case 'swiftui':
          return name + '(\n  alignment: ' + crossA + ',\n  spacing: ' + gap + ',\n) { /* ... */ }';
      }
    }

    case 'stack': {
      const stack = layout as Stack;
      const align = m.alignment2D(stack.alignment);

      switch (platform) {
        case 'flutter':
          return name + '(\n  alignment: ' + align + ',\n  clipBehavior: Clip.antiAlias,\n  children: [ /* ... */ ],\n)';
        case 'compose':
          return name + '(\n  modifier = Modifier,\n  contentAlignment: ' + align + ',\n) { /* ... */ }';
        case 'swiftui':
          return name + '(alignment: ' + align + ') { /* ... */ }';
      }
    }

    case 'scroll': {
      const scroll = layout as Scroll;
      const axis = m.scrollAxisCode(scroll.axis);

      switch (platform) {
        case 'flutter':
          return name + '(\n  scrollDirection: ' + axis + ',\n  child: /* ... */,\n)';
        case 'compose':
          return name + ' {\n  items(/* ... */) { /* ... */ }\n}';
        case 'swiftui':
          return name + '(' + axis + ', showsIndicators: ' + scroll.showIndicator + ') { /* ... */ }';
      }
    }
  }
}

// ============================================================
// 9.  Modifier Chain Serialization
// ============================================================

export function serializeChildModifiers(attrs: ChildLayout, platform: PlatformKind): string[] {
  const m = getLayoutMapping(platform);
  const mods: string[] = [];

  if (platform === 'swiftui') {
    if (attrs.padding) mods.push('.padding(' + swiftUIEdgeInsets(attrs.padding) + ')');
    if (attrs.margin) mods.push('.padding(' + swiftUIEdgeInsets(attrs.margin) + ')');
    if (attrs.constraints) {
      const c = m.constraintsSnippet(attrs.constraints);
      if (c) mods.push(c);
    }
    if (attrs.position) {
      const p = attrs.position;
      mods.push('.position(x: ' + px(p.left ?? 0) + ', y: ' + px(p.top ?? 0) + ')');
    }
    if (attrs.flexGrow !== undefined) {
      mods.push('.frame(maxWidth: .infinity)');
    }
  }

  if (platform === 'compose') {
    if (attrs.padding) mods.push('Modifier.padding(' + composeInsets(attrs.padding) + ')');
    if (attrs.margin) mods.push('Modifier.padding(' + composeInsets(attrs.margin) + ')');
    if (attrs.constraints) {
      const c = m.constraintsSnippet(attrs.constraints);
      if (c) mods.push(c);
    }
    if (attrs.alignSelf) mods.push('Modifier.align(' + m.crossAxisAlignment(attrs.alignSelf) + ')');
    if (attrs.flexGrow !== undefined) mods.push('Modifier.weight(' + attrs.flexGrow + 'f)');
  }

  if (platform === 'flutter') {
    if (attrs.padding) mods.push('Padding(padding: ' + flutterEdgeInsets(attrs.padding) + ')');
    if (attrs.margin) mods.push('Padding(padding: ' + flutterEdgeInsets(attrs.margin) + ')');
    if (attrs.constraints) {
      const c = m.constraintsSnippet(attrs.constraints);
      if (c) mods.push('ConstrainedBox(constraints: ' + c + ')');
    }
    if (attrs.flexGrow !== undefined) {
      mods.push('Expanded(flex: ' + attrs.flexGrow + ')');
    }
  }

  return mods;
}

export { indent };
