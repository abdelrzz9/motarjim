import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  root: '.',
  base: '/',
  resolve: {
    alias: {
      '@motarjim/shared': path.resolve(__dirname, '../packages/shared'),
      '@motarjim/parser': path.resolve(__dirname, '../packages/parser'),
      '@motarjim/css-analyzer': path.resolve(__dirname, '../packages/css-analyzer'),
      '@motarjim/semantic-analyzer': path.resolve(__dirname, '../packages/semantic-analyzer'),
      '@motarjim/accessibility-analyzer': path.resolve(__dirname, '../packages/accessibility-analyzer'),
      '@motarjim/ir': path.resolve(__dirname, '../packages/ir'),
      '@motarjim/ir-v2': path.resolve(__dirname, '../packages/ir-v2'),
      '@motarjim/optimizer': path.resolve(__dirname, '../packages/optimizer'),
      '@motarjim/compiler-core': path.resolve(__dirname, '../packages/compiler-core'),
      '@motarjim/pipeline-core': path.resolve(__dirname, '../packages/pipeline-core'),
      '@motarjim/generator-core': path.resolve(__dirname, '../packages/generator-core'),
      '@motarjim/generator-flutter': path.resolve(__dirname, '../packages/generators/flutter'),
      '@motarjim/generator-compose': path.resolve(__dirname, '../packages/generators/compose'),
      '@motarjim/generator-swiftui': path.resolve(__dirname, '../packages/generators/swiftui'),
    },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'esnext',
    commonjsOptions: {
      include: [/packages/, /node_modules/],
    },
    rollupOptions: {
      onLog(level, log) {
        if (level === 'warn' && log.code === 'MODULE_LEVEL_DIRECTIVE') return;
      },
    },
  },
  optimizeDeps: {
    include: [
      '@motarjim/pipeline-core',
      '@motarjim/parser',
      '@motarjim/css-analyzer',
      '@motarjim/semantic-analyzer',
      '@motarjim/accessibility-analyzer',
      '@motarjim/ir',
      '@motarjim/optimizer',
      '@motarjim/compiler-core',
      '@motarjim/generator-core',
      '@motarjim/generator-flutter',
      '@motarjim/generator-compose',
      '@motarjim/generator-swiftui',
      '@motarjim/shared',
      'parse5',
      'postcss',
    ],
  },
});
