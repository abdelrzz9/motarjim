// ============================================================
// Cross-Platform Layout System
// ============================================================
//
// Platform-neutral layout primitives that describe how UI
// elements are arranged. Every layout maps to Flutter, Jetpack
// Compose, and SwiftUI without loss.
//
// Two levels:
//   1. Container-level — how a parent arranges its children
//      (Flex, Stack, Scroll)
//   2. Child-level     — how individual children behave within
//      a parent (alignment, flex, constraints, position)
// ============================================================

// ============================================================
// 1.  Direction & Axis Primitives
// ============================================================

export type Direction = 'row' | 'column' | 'row-reverse' | 'column-reverse';

export type Axis = 'horizontal' | 'vertical' | 'both';

export function isHorizontal(dir: Direction): boolean {
  return dir === 'row' || dir === 'row-reverse';
}

export function isReverse(dir: Direction): boolean {
  return dir === 'row-reverse' || dir === 'column-reverse';
}

// ============================================================
// 2.  Alignment
// ============================================================

/**
 * Main-axis alignment (justify-content equivalent).
 * Controls distribution of children along the primary axis.
 */
export type MainAxisAlignment =
  | 'start'
  | 'center'
  | 'end'
  | 'space-between'
  | 'space-around'
  | 'space-evenly';

/**
 * Cross-axis alignment (align-items equivalent).
 * Controls how children align perpendicular to the primary axis.
 */
export type CrossAxisAlignment =
  | 'start'
  | 'center'
  | 'end'
  | 'stretch'
  | 'baseline';

/**
 * 2D alignment within a box (used by Stack, absolute positioning).
 */
export interface Alignment2D {
  x: 'start' | 'center' | 'end' | 'stretch';
  y: 'start' | 'center' | 'end' | 'stretch';
}

export const ALIGN_TOP_LEFT: Alignment2D = { x: 'start', y: 'start' };
export const ALIGN_TOP_CENTER: Alignment2D = { x: 'center', y: 'start' };
export const ALIGN_TOP_RIGHT: Alignment2D = { x: 'end', y: 'start' };
export const ALIGN_CENTER_LEFT: Alignment2D = { x: 'start', y: 'center' };
export const ALIGN_CENTER: Alignment2D = { x: 'center', y: 'center' };
export const ALIGN_CENTER_RIGHT: Alignment2D = { x: 'end', y: 'center' };
export const ALIGN_BOTTOM_LEFT: Alignment2D = { x: 'start', y: 'end' };
export const ALIGN_BOTTOM_CENTER: Alignment2D = { x: 'center', y: 'end' };
export const ALIGN_BOTTOM_RIGHT: Alignment2D = { x: 'end', y: 'end' };
export const ALIGN_STRETCH: Alignment2D = { x: 'stretch', y: 'stretch' };

// ============================================================
// 3.  Spacing
// ============================================================

/**
 * Gap between children in a flex/scroll container.
 * `fixed` uses a constant value; `responsive` adapts to viewport.
 */
export type Gap =
  | { kind: 'fixed'; value: number }
  | { kind: 'responsive'; min: number; max: number }
  | { kind: 'none' };

export const noGap = (): Gap => ({ kind: 'none' });
export const fixedGap = (value: number): Gap => ({ kind: 'fixed', value });

// ============================================================
// 4.  Constraints  (min/max sizing)
// ============================================================

/**
 * Sizing constraints that control how large or small a node
 * can be. Every dimension is optional — unset means "unbounded".
 */
export interface Constraints {
  minWidth?: number;
  maxWidth?: number;
  minHeight?: number;
  maxHeight?: number;
}

export const unbounded = (): Constraints => ({});

export function isConstrained(c: Constraints): boolean {
  return c.minWidth !== undefined || c.maxWidth !== undefined
      || c.minHeight !== undefined || c.maxHeight !== undefined;
}

// ============================================================
// 5.  Intrinsic Sizing
// ============================================================

/**
 * How a node determines its own size along one axis.
 *
 *  - min-content   → narrowest possible (shrink-wrap)
 *  - max-content   → widest possible (no wrapping)
 *  - fit-content   → min-content up to a maximum
 *  - expand        → fill available space (stretch)
 *  - fixed         → exactly `value`
 */
export type IntrinsicSize =
  | { kind: 'min-content' }
  | { kind: 'max-content' }
  | { kind: 'fit-content'; max: number }
  | { kind: 'expand'; flex?: number }
  | { kind: 'fixed'; value: number };

export const minContent = (): IntrinsicSize => ({ kind: 'min-content' });
export const maxContent = (): IntrinsicSize => ({ kind: 'max-content' });
export const fitContent = (max: number): IntrinsicSize => ({ kind: 'fit-content', max });
export const expand = (flex?: number): IntrinsicSize => ({ kind: 'expand', flex });
export const fixedSize = (value: number): IntrinsicSize => ({ kind: 'fixed', value });

/**
 * Two-dimensional intrinsic sizing.
 */
export interface IntrinsicSizing {
  width: IntrinsicSize;
  height: IntrinsicSize;
}

export const intrinsicDefaults = (): IntrinsicSizing => ({
  width: { kind: 'min-content' },
  height: { kind: 'min-content' },
});

// ============================================================
// 6.  Wrap Mode
// ============================================================

export type WrapMode = 'no-wrap' | 'wrap' | 'wrap-reverse';

// ============================================================
// 7.  Container-Level Layout Primitives
// ============================================================

/**
 * A laid-out container arranges its children using one of these
 * strategies. This is the core abstraction that maps to platform
 * layout widgets (Row/Column, HStack/VStack, Box, ScrollView, etc).
 */
export type ContainerLayout = Flex | Stack | Scroll;

// ── Flex (Row / Column) ──────────────────────────────────────

export interface Flex {
  kind: 'flex';
  direction: Direction;
  mainAxisAlignment: MainAxisAlignment;
  crossAxisAlignment: CrossAxisAlignment;
  gap: Gap;
  wrap: WrapMode;
  /**
   * If true, children are distributed in reverse visual order.
   * Maps to `Reversed` in SwiftUI, `reverse: true` in Flutter.
   */
  reversed: boolean;
}

export const flexRow = (overrides?: Partial<Flex>): Flex => ({
  kind: 'flex',
  direction: 'row',
  mainAxisAlignment: 'start',
  crossAxisAlignment: 'center',
  gap: noGap(),
  wrap: 'no-wrap',
  reversed: false,
  ...overrides,
});

export const flexColumn = (overrides?: Partial<Flex>): Flex => ({
  kind: 'flex',
  direction: 'column',
  mainAxisAlignment: 'start',
  crossAxisAlignment: 'stretch',
  gap: noGap(),
  wrap: 'no-wrap',
  reversed: false,
  ...overrides,
});

// ── Stack (overlapping children) ─────────────────────────────

export type StackFit = 'loose' | 'expand' | 'passthrough';

export interface Stack {
  kind: 'stack';
  alignment: Alignment2D;
  fit: StackFit;
  clip: boolean;
}

export const defaultStack = (overrides?: Partial<Stack>): Stack => ({
  kind: 'stack',
  alignment: ALIGN_TOP_LEFT,
  fit: 'loose',
  clip: true,
  ...overrides,
});

// ── Scroll ───────────────────────────────────────────────────

export type SnapAlignment = 'start' | 'center' | 'end';

export interface Scroll {
  kind: 'scroll';
  axis: Axis;
  showIndicator: boolean;
  pagingEnabled: boolean;
  snap?: SnapAlignment;
  alwaysScrollable: boolean;
}

export const verticalScroll = (overrides?: Partial<Scroll>): Scroll => ({
  kind: 'scroll',
  axis: 'vertical',
  showIndicator: true,
  pagingEnabled: false,
  alwaysScrollable: false,
  ...overrides,
});

export const horizontalScroll = (overrides?: Partial<Scroll>): Scroll => ({
  kind: 'scroll',
  axis: 'horizontal',
  showIndicator: false,
  pagingEnabled: false,
  alwaysScrollable: false,
  ...overrides,
});

// ============================================================
// 8.  Per-Child Layout Attributes
// ============================================================

/**
 * Layout metadata attached to an individual child within a
 * container. These control how the child sizes, aligns, and
 * positions itself relative to its siblings.
 */
export interface ChildLayout {
  // ── Flex child ──────────────────────────────────────────
  /** Flex grow factor (proportion of free space to absorb). */
  flexGrow?: number;
  /** Flex shrink factor (how much to shrink when overflow). */
  flexShrink?: number;
  /** Initial main-axis size before free space is distributed. */
  flexBasis?: number | 'auto';

  // ── Alignment ──────────────────────────────────────────
  /** Override the parent's cross-axis alignment for this child. */
  alignSelf?: CrossAxisAlignment;

  // ── Constraints ────────────────────────────────────────
  /** Min/max bounds applied to this child. */
  constraints?: Constraints;

  // ── Positioning (Stack children) ───────────────────────
  /** Positional offset within a Stack parent. */
  position?: Positioned;

  // ── Visual order ───────────────────────────────────────
  /** Override visual order (CSS `order` equivalent). */
  order?: number;

  // ── Insets ─────────────────────────────────────────────
  /** Margin / padding around this child. */
  margin?: EdgeInsets;
  padding?: EdgeInsets;

  // ── Sizing hints ───────────────────────────────────────
  /** Override the node's intrinsic sizing for layout purposes. */
  sizing?: Partial<IntrinsicSizing>;
}

export interface EdgeInsets {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export const zeroInsets = (): EdgeInsets => ({ top: 0, right: 0, bottom: 0, left: 0 });

export const allInsets = (value: number): EdgeInsets => ({
  top: value, right: value, bottom: value, left: value,
});

export const symmetricInsets = (h: number, v: number): EdgeInsets => ({
  top: v, right: h, bottom: v, left: h,
});

// ============================================================
// 9.  Positioned (absolute layout)
// ============================================================

/**
 * Absolute positioning of a child within its parent.
 * Maps to Flutter `Positioned`, Compose `Modifier.offset`,
 * SwiftUI `.offset()` / `.position()`.
 */
export interface Positioned {
  top?: number;
  right?: number;
  bottom?: number;
  left?: number;
  width?: number;
  height?: number;
}

export const positionedFill = (): Positioned => ({
  top: 0, right: 0, bottom: 0, left: 0,
});

export const positionedCenter = (size?: number): Positioned => ({
  top: undefined, right: undefined, bottom: undefined, left: undefined,
  width: size, height: size,
});

// ============================================================
// 10.  Full Layout Node (container + child attrs)
// ============================================================

/**
 * A layout-aware node combining the container strategy and
 * per-child attributes. This is what the codegen consumes.
 */
export interface LayoutNode {
  id: string;
  container: ContainerLayout;
  child: ChildLayout;
  children: LayoutNode[];
  intrinsicSizing: IntrinsicSizing;
}

// ============================================================
// 11.  Type guards
// ============================================================

export function isFlex(layout: ContainerLayout): layout is Flex {
  return layout.kind === 'flex';
}

export function isStack(layout: ContainerLayout): layout is Stack {
  return layout.kind === 'stack';
}

export function isScroll(layout: ContainerLayout): layout is Scroll {
  return layout.kind === 'scroll';
}

export function isRow(layout: Flex): boolean {
  return layout.direction === 'row' || layout.direction === 'row-reverse';
}

export function isColumn(layout: Flex): boolean {
  return layout.direction === 'column' || layout.direction === 'column-reverse';
}

export function hasFlex(attrs: ChildLayout): boolean {
  return attrs.flexGrow !== undefined || attrs.flexShrink !== undefined;
}

export function hasPosition(attrs: ChildLayout): attrs is ChildLayout & { position: Positioned } {
  return attrs.position !== undefined;
}
