import { defineConfig } from 'vitest/config';
import { resolve } from 'path';

export default defineConfig({
  resolve: {
    alias: {
      '@html-native/shared': resolve(__dirname, 'packages/shared/index.ts'),
      '@html-native/parser': resolve(__dirname, 'packages/parser/index.ts'),
      '@html-native/css-analyzer': resolve(__dirname, 'packages/css-analyzer/index.ts'),
      '@html-native/semantic-analyzer': resolve(__dirname, 'packages/semantic-analyzer/index.ts'),
      '@html-native/ir': resolve(__dirname, 'packages/ir/index.ts'),
      '@html-native/optimizer': resolve(__dirname, 'packages/optimizer/index.ts'),
      '@html-native/generator-flutter': resolve(__dirname, 'packages/generators/flutter/index.ts'),
      '@html-native/generator-compose': resolve(__dirname, 'packages/generators/compose/index.ts'),
      '@html-native/generator-swiftui': resolve(__dirname, 'packages/generators/swiftui/index.ts'),
      '@html-native/cli': resolve(__dirname, 'packages/cli/index.ts'),
      '@html-native/semantic-analyzer/ai': resolve(__dirname, 'packages/semantic-analyzer/ai.ts'),
    },
  },
  test: {
    globals: true,
    include: ['packages/**/*.test.ts', 'tests/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['packages/**'],
      exclude: ['**/*.test.ts', '**/node_modules/**'],
    },
  },
});
