import type { HtmlNode } from './index.js';
import type { UiNode } from './index.js';

export function countHtmlNodes(node: HtmlNode): number {
  let count = 1;
  for (const child of node.children) count += countHtmlNodes(child);
  return count;
}

const COMPONENT_TYPES = new Set([
  'Button', 'Card', 'NavigationBar', 'AppBar', 'Drawer',
  'HeroSection', 'Footer', 'Sidebar', 'Dialog', 'Modal',
  'Tabs', 'Form', 'TextField', 'TextArea', 'List',
]);

export function countComponentNodes(node: UiNode): number {
  let count = COMPONENT_TYPES.has(node.type) ? 1 : 0;
  for (const child of node.children) count += countComponentNodes(child);
  return count;
}

export function countNodes(node: UiNode): number {
  let count = 1;
  for (const child of node.children) count += countNodes(child);
  return count;
}
