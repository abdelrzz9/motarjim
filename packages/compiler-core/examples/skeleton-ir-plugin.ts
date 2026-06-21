import type { MotarjimPlugin, PluginApi, IrTransform } from '../src/plugin-api.js';
import type { UiNode } from '@html-native/shared';

/**
 * Skeleton-screen IR transform.
 * Detects loading states (aria-busy="true" or class="skeleton")
 * and replaces the node tree with a skeleton placeholder structure.
 */
export const skeletonIrPlugin: MotarjimPlugin = {
  id: 'skeleton-ir',
  name: 'Skeleton Screen Transform',
  version: '1.0.0',
  description: 'Replaces loading-state content with skeleton placeholder IR',

  register(api: PluginApi): void {
    const transform: IrTransform = {
      id: 'skeleton-replace',
      name: 'Skeleton Replacement',
      description: 'Walks the IR tree and replaces skeleton elements',

      run(ir: UiNode, _ctx): { ir: UiNode; diagnostics: any[] } {
        const diagnostics: any[] = [];
        const transformed = walkAndReplace(ir);

        const skeletonCount = countSkeletons(transformed);
        if (skeletonCount > 0) {
          diagnostics.push({
            code: 'SKEL_001',
            message: `Replaced ${skeletonCount} skeleton elements`,
            severity: 'info',
            phase: 'ir',
          });
        }

        return { ir: transformed, diagnostics };
      },
    };

    api.registerIrTransform(transform);
  },
};

function isSkeleton(node: UiNode): boolean {
  const props = node.properties ?? {};
  const sourceTag = node.sourceHtmlTag ?? '';

  if (sourceTag === 'svg') return false;

  const ariaBusy = props['aria-busy'];
  if (ariaBusy === 'true' || ariaBusy === true) return true;

  const cls = String(props.className ?? props.class ?? '');
  if (cls.includes('skeleton') || cls.includes('loading')) return true;

  const role = String(props.role ?? '');
  if (role === 'progressbar' || role === 'alert') {
    if (cls.includes('skeleton') || cls.includes('loading')) return true;
  }

  return false;
}

function walkAndReplace(node: UiNode): UiNode {
  const children = node.children.map(walkAndReplace);

  if (isSkeleton(node)) {
    return buildSkeletonPlaceholder(node);
  }

  return { ...node, children };
}

function buildSkeletonPlaceholder(original: UiNode): UiNode {
  const props = original.properties ?? {};

  let width = inferWidth(props);
  let height = inferHeight(props);

  const shape = detectShape(props);

  return {
    type: 'Container',
    properties: {
      style: 'skeleton',
      width,
      height,
      shape,
      borderRadius: shape === 'circle' ? '50%' : '4px',
      backgroundColor: '#E5E7EB',
      role: 'presentation',
      'aria-hidden': true,
    },
    children: [],
    styles: original.styles,
    sourceHtmlTag: original.sourceHtmlTag,
    originalNodeId: original.originalNodeId,
  };
}

function inferWidth(props: Record<string, unknown>): string {
  const w = String(props.width ?? props.w ?? '');
  if (w.match(/^\d+px$/)) return w;
  if (w.match(/^\d+%$/)) return w;
  if (w === 'full') return '100%';
  return '100%';
}

function inferHeight(props: Record<string, unknown>): string {
  const h = String(props.height ?? props.h ?? '');
  if (h.match(/^\d+px$/)) return h;
  if (h.match(/^\d+%$/)) return h;
  if (h === 'full') return '100%';
  return '20px';
}

function detectShape(props: Record<string, unknown>): string {
  const cls = String(props.className ?? props.class ?? '');
  if (cls.includes('circle') || cls.includes('avatar') || cls.includes('icon')) return 'circle';
  if (cls.includes('text') || cls.includes('line')) return 'text-line';
  return 'rectangle';
}

function countSkeletons(node: UiNode): number {
  let count = 0;
  if (node.properties?.style === 'skeleton') count++;
  for (const child of node.children) {
    count += countSkeletons(child);
  }
  return count;
}
