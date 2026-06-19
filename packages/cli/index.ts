#!/usr/bin/env node
import { Command } from 'commander';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { resolve } from 'path';
import { parseHtml } from '@html-native/parser';
import { parseCss, applyStyles } from '@html-native/css-analyzer';
import { detectSemantics } from '@html-native/semantic-analyzer';
import { styledNodeToIr } from '@html-native/ir';
import { optimize } from '@html-native/optimizer';
import { generate as generateFlutter } from '@html-native/generator-flutter';
import { generate as generateCompose } from '@html-native/generator-compose';
import { generate as generateSwiftUI } from '@html-native/generator-swiftui';
import { createAiDetector } from '@html-native/semantic-analyzer/ai';

const program = new Command();

program
  .name('html-native')
  .description('Convert HTML/CSS to native UI code')
  .version('0.1.0');

program
  .command('convert')
  .description('Convert HTML to native UI code')
  .requiredOption('-i, --input <path>', 'Input HTML file')
  .option('-c, --css <path>', 'Input CSS file')
  .requiredOption('-t, --target <platform>', 'Target platform: flutter, compose, swiftui')
  .option('-o, --output <path>', 'Output file path')
  .option('--ai-enhance', 'Use Ollama AI for enhanced semantic detection (optional, local-only)')
  .option('--ai-model <model>', 'Ollama model name (default: qwen2.5:7b)')
  .action(async (opts) => {
    const inputPath = resolve(opts.input);
    if (!existsSync(inputPath)) {
      console.error(`Error: File not found: ${inputPath}`);
      process.exit(1);
    }

    const html = readFileSync(inputPath, 'utf-8');
    let css = '';
    if (opts.css) {
      const cssPath = resolve(opts.css);
      if (!existsSync(cssPath)) {
        console.error(`Error: CSS file not found: ${cssPath}`);
        process.exit(1);
      }
      css = readFileSync(cssPath, 'utf-8');
    }

    try {
      // 1. Parse HTML
      const ast = parseHtml(html);

      // 2. Parse & apply CSS
      const stylesheet = parseCss(css);
      const styledNodes = applyStyles(ast.children, stylesheet);

      // 3. Semantic analysis (optional AI enhancement)
      let hints;
      if (opts.aiEnhance) {
        console.error('Using AI-enhanced semantic detection (Ollama)...');
        const aiDetector = createAiDetector(opts.aiModel ? { model: opts.aiModel } : undefined);
        hints = await aiDetector(styledNodes);
      } else {
        hints = detectSemantics(styledNodes);
      }

      const rootStyled: import('@html-native/shared').StyledNode = {
        node: ast,
        styles: {},
        children: styledNodes,
      };

      // 4. Convert to IR
      const ir = styledNodeToIr(rootStyled, hints);

      // 5. Optimize
      const optimized = optimize(ir);

      // 6. Generate code
      let result;
      switch (opts.target) {
        case 'flutter':
          result = generateFlutter(optimized);
          break;
        case 'compose':
          result = generateCompose(optimized);
          break;
        case 'swiftui':
          result = generateSwiftUI(optimized);
          break;
        default:
          console.error(`Error: Unknown target "${opts.target}". Use flutter, compose, or swiftui.`);
          process.exit(1);
      }

      if (opts.output) {
        writeFileSync(resolve(opts.output), result.code, 'utf-8');
        console.log(`Written to ${opts.output}`);
        console.log(`Generated ${result.metadata.nodes} nodes in ${result.metadata.duration}ms`);
      } else {
        console.log(result.code);
      }
    } catch (err) {
      console.error('Error during conversion:', (err as Error).message);
      process.exit(1);
    }
  });

program.parse();
