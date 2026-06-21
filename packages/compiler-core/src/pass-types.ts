import type { UiNode } from '@html-native/shared';

/**
 * An optimization pass transforms the IR tree.
 * Must be pure (no side effects, no external state).
 * Same contract as `@html-native/optimizer`'s `OptimizationPass`.
 */
export interface OptimizationPass {
  name: string;
  run: (node: UiNode) => UiNode;
}
