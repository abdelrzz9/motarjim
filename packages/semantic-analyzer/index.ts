import {
  HtmlNode,
  StyledNode,
  ResolvedStyles,
  SemanticHint,
  UiNodeType,
} from '@html-native/shared';

export type { SemanticDetector } from './ai.js';

export function detectSemantics(
  styledNodes: StyledNode[],
): SemanticHint[] {
  const hints: SemanticHint[] = [];

  function walk(node: StyledNode) {
    const tag = node.node.tagName;
    const classes = getClasses(node.node);
    const styles = node.styles;

    // Nav detection
    if (tag === 'nav' || classes.includes('nav') || classes.includes('navbar') || classes.includes('navigation')) {
      hints.push({ type: 'NavigationBar', confidence: 0.9, node: node.node, reason: 'nav tag or class' });
    }

    // Card detection
    if (classes.includes('card') || classes.includes('card-body') || classes.includes('card-content')) {
      hints.push({ type: 'Card', confidence: 0.85, node: node.node, reason: 'card class' });
    } else if (tag === 'div' && hasShadow(styles) && hasRoundedCorners(styles)) {
      hints.push({ type: 'Card', confidence: 0.6, node: node.node, reason: 'shadow + rounded corners' });
    }

    // Hero section
    if (classes.includes('hero') || classes.includes('hero-section') || classes.includes('banner')) {
      hints.push({ type: 'HeroSection', confidence: 0.9, node: node.node, reason: 'hero class' });
    }

    // Dialog/Modal
    if (classes.includes('modal') || classes.includes('dialog') || classes.includes('overlay')) {
      hints.push({ type: 'Dialog', confidence: 0.9, node: node.node, reason: 'modal/dialog class' });
    }

    // AppBar
    if (tag === 'header' && (classes.includes('app-bar') || classes.includes('toolbar') || classes.includes('topbar'))) {
      hints.push({ type: 'AppBar', confidence: 0.85, node: node.node, reason: 'header with app-bar class' });
    }

    // Sidebar
    if (classes.includes('sidebar') || classes.includes('side-bar') || classes.includes('drawer')) {
      hints.push({ type: 'Sidebar', confidence: 0.9, node: node.node, reason: 'sidebar class' });
    }

    // Footer
    if (tag === 'footer' || classes.includes('footer')) {
      hints.push({ type: 'Footer', confidence: 0.9, node: node.node, reason: 'footer tag or class' });
    }

    // Tabs
    if (classes.includes('tabs') || classes.includes('tab-bar') || classes.includes('tab-container')) {
      hints.push({ type: 'Tabs', confidence: 0.85, node: node.node, reason: 'tabs class' });
    }

    // Form detection
    if (tag === 'form') {
      hints.push({ type: 'Form', confidence: 0.9, node: node.node, reason: 'form tag' });
    }

    for (const child of node.children) {
      walk(child);
    }
  }

  for (const node of styledNodes) {
    walk(node);
  }

  return hints;
}

function getClasses(node: HtmlNode): string[] {
  const classAttr = node.attributes.find(a => a.name === 'class');
  return classAttr ? classAttr.value.split(/\s+/) : [];
}

function hasShadow(styles: ResolvedStyles): boolean {
  return 'box-shadow' in styles || 'boxShadow' in styles;
}

function hasRoundedCorners(styles: ResolvedStyles): boolean {
  const val = styles['border-radius'] || styles['borderRadius'];
  return !!val && val !== '0' && val !== '0px';
}
