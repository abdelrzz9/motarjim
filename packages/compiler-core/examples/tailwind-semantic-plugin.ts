import type { MotarjimPlugin, PluginApi, PluginSemanticRule } from '../src/plugin-api.js';

/**
 * Tailwind CSS semantic detector.
 * Converts Tailwind utility classes into semantic component hints.
 */
export const tailwindSemanticPlugin: MotarjimPlugin = {
  id: 'tailwind-semantic',
  name: 'Tailwind Semantic Detector',
  version: '1.0.0',
  description: 'Detects components from Tailwind CSS utility classes',

  register(api: PluginApi): void {
    const rules: PluginSemanticRule[] = [
      {
        pluginId: 'tailwind-semantic',
        id: 'tw-card',
        componentType: 'Card',
        description: 'Tailwind card: shadow + rounded + bg-white',
        signals: [
          { type: 'class', name: 'shadow',                 value: '*',   valueMatch: 'exists', weight: 0.7 },
          { type: 'class', name: 'rounded',                 value: '*',   valueMatch: 'exists', weight: 0.4 },
          { type: 'class', name: 'bg-white',                value: '*',   valueMatch: 'exists', weight: 0.3 },
          { type: 'class', name: 'p-',                      value: '*',   valueMatch: 'regex', weight: 0.3 },
        ],
        minScore: 0.5,
        priority: 60,
      },
      {
        pluginId: 'tailwind-semantic',
        id: 'tw-navbar',
        componentType: 'NavigationBar',
        description: 'Tailwind navbar: flex + justify-between + items-center',
        signals: [
          { type: 'class', name: 'flex',                    value: '*',   valueMatch: 'exists', weight: 0.5 },
          { type: 'class', name: 'justify-between',         value: '*',   valueMatch: 'exists', weight: 0.6 },
          { type: 'class', name: 'items-center',            value: '*',   valueMatch: 'exists', weight: 0.4 },
          { type: 'class', name: 'bg-',                     value: '*',   valueMatch: 'regex', weight: 0.3 },
        ],
        minScore: 0.5,
        priority: 65,
      },
      {
        pluginId: 'tailwind-semantic',
        id: 'tw-hero',
        componentType: 'HeroSection',
        description: 'Tailwind hero: min-h-screen + flex + items-center + justify-center',
        signals: [
          { type: 'class', name: 'min-h-screen',            value: '*',   valueMatch: 'exists', weight: 0.8 },
          { type: 'class', name: 'flex',                     value: '*',   valueMatch: 'exists', weight: 0.4 },
          { type: 'class', name: 'items-center',             value: '*',   valueMatch: 'exists', weight: 0.3 },
          { type: 'class', name: 'justify-center',           value: '*',   valueMatch: 'exists', weight: 0.3 },
        ],
        minScore: 0.5,
        priority: 70,
      },
      {
        pluginId: 'tailwind-semantic',
        id: 'tw-sidebar',
        componentType: 'Sidebar',
        description: 'Tailwind sidebar: w-64 + fixed + inset-y-0 + left-0',
        signals: [
          { type: 'class', name: 'w-64',                    value: '*',   valueMatch: 'exists', weight: 0.6 },
          { type: 'class', name: 'fixed',                   value: '*',   valueMatch: 'exists', weight: 0.5 },
          { type: 'class', name: 'inset-y-0',               value: '*',   valueMatch: 'exists', weight: 0.7 },
        ],
        minScore: 0.6,
        priority: 65,
      },
      {
        pluginId: 'tailwind-semantic',
        id: 'tw-modal',
        componentType: 'Dialog',
        description: 'Tailwind modal: fixed + inset-0 + z-50 + bg-black/50',
        signals: [
          { type: 'class', name: 'fixed',                   value: '*',   valueMatch: 'exists', weight: 0.4 },
          { type: 'class', name: 'inset-0',                 value: '*',   valueMatch: 'exists', weight: 0.7 },
          { type: 'class', name: 'z-',                      value: '50',  valueMatch: 'regex', weight: 0.5 },
          { type: 'class', name: 'bg-black',               value: '*',   valueMatch: 'exists', weight: 0.3 },
        ],
        minScore: 0.55,
        priority: 65,
      },
      {
        pluginId: 'tailwind-semantic',
        id: 'tw-badge',
        componentType: 'Container',
        description: 'Tailwind badge: inline-flex + items-center + px-2 + rounded-full',
        signals: [
          { type: 'class', name: 'inline-flex',             value: '*',   valueMatch: 'exists', weight: 0.5 },
          { type: 'class', name: 'rounded-full',            value: '*',   valueMatch: 'exists', weight: 0.6 },
          { type: 'class', name: 'px-',                     value: '*',   valueMatch: 'regex',  weight: 0.3 },
        ],
        minScore: 0.5,
        priority: 50,
      },
    ];

    api.registerSemanticRules(rules);
  },
};
