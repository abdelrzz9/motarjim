import type { ResolvedStyles } from '@motarjim/shared';
import type { ComputedStyle } from '@motarjim/shared/ir-v2.js';
import { parsePx, parseBoxShorthand } from './mappers/spacing-mapper.js';
import { parseTypography } from './mappers/typography-mapper.js';
import { parseLayoutStyles } from './mappers/layout-mapper.js';
import { parseBorderWidth, parseBorderRadius } from './mappers/border-mapper.js';

export { parsePx, parseBoxShorthand };

function parsePxSimple(value: string | undefined): number | undefined {
  return parsePx(value);
}

export function computeStyle(styles: ResolvedStyles): ComputedStyle {
  const r: ComputedStyle = {};

  // Use typography mapper
  const typography = parseTypography(styles);
  if (typography.fontFamily) r.fontFamily = typography.fontFamily;
  if (typography.fontSize) r.fontSize = typography.fontSize;
  if (typography.fontWeight) r.fontWeight = typography.fontWeight;
  if (typography.fontStyle) r.fontStyle = typography.fontStyle;
  if (typography.lineHeight) r.lineHeight = typography.lineHeight;
  if (typography.textAlign) r.textAlign = typography.textAlign as ComputedStyle['textAlign'];
  if (typography.textDecoration) r.textDecoration = typography.textDecoration;
  if (typography.textTransform) r.textTransform = typography.textTransform as ComputedStyle['textTransform'];
  if (typography.color) r.color = typography.color;
  if (typography.letterSpacing) r.letterSpacing = typography.letterSpacing;

  // Margin
  if (styles['margin']) {
    const box = parseBoxShorthand(styles['margin']);
    if (box) {
      r.marginTop = box.top;
      r.marginRight = box.right;
      r.marginBottom = box.bottom;
      r.marginLeft = box.left;
    }
  }
  if (styles['margin-top']) r.marginTop = parsePxSimple(styles['margin-top']);
  if (styles['margin-right']) r.marginRight = parsePxSimple(styles['margin-right']);
  if (styles['margin-bottom']) r.marginBottom = parsePxSimple(styles['margin-bottom']);
  if (styles['margin-left']) r.marginLeft = parsePxSimple(styles['margin-left']);

  // Padding
  if (styles['padding']) {
    const box = parseBoxShorthand(styles['padding']);
    if (box) {
      r.paddingTop = box.top;
      r.paddingRight = box.right;
      r.paddingBottom = box.bottom;
      r.paddingLeft = box.left;
    }
  }
  if (styles['padding-top']) r.paddingTop = parsePxSimple(styles['padding-top']);
  if (styles['padding-right']) r.paddingRight = parsePxSimple(styles['padding-right']);
  if (styles['padding-bottom']) r.paddingBottom = parsePxSimple(styles['padding-bottom']);
  if (styles['padding-left']) r.paddingLeft = parsePxSimple(styles['padding-left']);

  // Border
  if (styles['border-width']) r.borderWidth = parseBorderWidth(styles['border-width']);
  if (styles['border-color']) r.borderColor = styles['border-color'];
  if (styles['border-radius']) r.borderRadius = parseBorderRadius(styles['border-radius']);
  if (styles['box-sizing']) r.boxSizing = styles['box-sizing'] as ComputedStyle['boxSizing'];

  // Use layout mapper
  const layout = parseLayoutStyles(styles);
  if (layout.display) r.display = layout.display;
  if (layout.position) r.position = layout.position as ComputedStyle['position'];
  if (layout.overflowX) r.overflowX = layout.overflowX as ComputedStyle['overflowX'];
  if (layout.overflowY) r.overflowY = layout.overflowY as ComputedStyle['overflowY'];
  if (layout.flexDirection) r.flexDirection = layout.flexDirection;
  if (layout.flexWrap) r.flexWrap = layout.flexWrap;
  if (layout.justifyContent) r.justifyContent = layout.justifyContent;
  if (layout.alignItems) r.alignItems = layout.alignItems;
  if (layout.alignContent) r.alignContent = layout.alignContent;
  if (layout.gap) r.gap = layout.gap;
  if (layout.flexGrow) r.flexGrow = layout.flexGrow;
  if (layout.flexShrink) r.flexShrink = layout.flexShrink;
  if (layout.flexBasis) r.flexBasis = layout.flexBasis;
  if (layout.alignSelf) r.alignSelf = layout.alignSelf as ComputedStyle['alignSelf'];
  if (layout.order) r.order = layout.order;

  // Color / Background
  if (styles['background-color']) r.backgroundColor = styles['background-color'];
  if (styles['background-image']) r.backgroundImage = styles['background-image'];
  if (styles['background']) r.backgroundColor = styles['background'];
  if (styles['opacity']) r.opacity = parseFloat(styles['opacity']) || undefined;

  // Sizing
  if (styles['width']) r.width = styles['width'];
  if (styles['height']) r.height = styles['height'];
  if (styles['min-width']) r.minWidth = styles['min-width'];
  if (styles['max-width']) r.maxWidth = styles['max-width'];
  if (styles['min-height']) r.minHeight = styles['min-height'];
  if (styles['max-height']) r.maxHeight = styles['max-height'];

  // Positioning
  if (styles['top']) r.top = parsePxSimple(styles['top']);
  if (styles['right']) r.right = parsePxSimple(styles['right']);
  if (styles['bottom']) r.bottom = parsePxSimple(styles['bottom']);
  if (styles['left']) r.left = parsePxSimple(styles['left']);
  if (styles['z-index']) r.zIndex = parseInt(styles['z-index'], 10) || undefined;

  // Visibility & Misc
  if (styles['visibility']) r.visibility = styles['visibility'] as ComputedStyle['visibility'];
  if (styles['transform']) r.transform = styles['transform'];
  if (styles['transition']) r.transition = styles['transition'];
  if (styles['box-shadow']) r.boxShadow = styles['box-shadow'];
  if (styles['cursor']) r.cursor = styles['cursor'];

  return r;
}
