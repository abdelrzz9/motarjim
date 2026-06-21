// AI-powered semantic detection using Ollama (optional, local-only).
// Falls back to the rule-based detectSemantics() if Ollama is unavailable.
// Features:
//   - Zod schema validation for structured AI output
//   - Proper nodeId mapping (no synthetic IDs)
//   - Few-shot prompt engineering
//   - Retry with exponential backoff
//   - Batching for large trees
//   - Merged AI + rule hints with deduplication
// Must never generate platform code — only semantic hints.

import { z } from 'zod';
import type { StyledNode, SemanticHint, HtmlNode, AiDetectorConfig } from '@html-native/shared';
import { detectSemantics, type SemanticDetector } from './index.js';

function unwrapDetectSemantics(nodes: StyledNode[]): SemanticHint[] {
  const result = detectSemantics(nodes);
  if (!result.ok) return [];
  return result.value;
}

// -- Zod Schemas for Structured Output Validation --

const SemanticComponentSchema = z.object({
  nodeId: z.string(),
  type: z.enum([
    'Card',
    'Hero',
    'Navbar',
    'Dialog',
    'Form',
    'Sidebar',
    'Footer',
    'Header',
    'List',
    'Grid',
    'Pricing',
    'Dashboard',
    'Marketing',
    'ProductCard',
  ]),
  confidence: z.number().min(0).max(1),
  reasoning: z.string().optional(),
});

const AiResponseSchema = z.object({
  components: z.array(SemanticComponentSchema).min(0).max(500),
});

// -- Configuration --

const DEFAULT_CONFIG: Required<AiDetectorConfig> = {
  model: 'qwen2.5:7b',
  baseUrl: 'http://localhost:11434',
  timeout: 30000,
  retryCount: 2,
  batchSize: 50,
  enableEmbeddings: false,
  embeddingModel: 'nomic-embed-text',
};

function resolveConfig(config?: AiDetectorConfig): Required<AiDetectorConfig> {
  return { ...DEFAULT_CONFIG, ...config };
}

// -- Node Serialization (passes real nodeIds) --

interface SerializedNode {
  nodeId: string;
  tag: string;
  class: string;
  id: string;
  styles: Record<string, string>;
  children: SerializedNode[];
}

function serializeStyledNode(nodes: StyledNode[]): SerializedNode[] {
  function walk(node: StyledNode): SerializedNode {
    return {
      nodeId: node.node.nodeId,
      tag: node.node.tagName,
      class: node.node.attributes.find(a => a.name === 'class')?.value || '',
      id: node.node.attributes.find(a => a.name === 'id')?.value || '',
      styles: Object.fromEntries(
        Object.entries(node.styles).filter(([_, v]) => v && typeof v === 'string')
      ),
      children: node.children.map(walk),
    };
  }
  return nodes.map(walk);
}

// -- Few-Shot Prompt --

const FEW_SHOT_EXAMPLES = `
Example 1:
Input:
[{"nodeId": "node_1", "tag": "nav", "class": "navbar", "styles": {"background": "#333"}, "children": [{"nodeId": "node_2", "tag": "h1", "class": "", "styles": {"color": "white"}}]}]
Output:
{"components": [{"nodeId": "node_1", "type": "Navbar", "confidence": 0.95, "reasoning": "nav tag with navbar class and dark background"}]}

Example 2:
Input:
[{"nodeId": "node_5", "tag": "div", "class": "card", "styles": {"border-radius": "12px", "box-shadow": "0 4px 6px rgba(0,0,0,0.1)"}, "children": [{"nodeId": "node_6", "tag": "h2", "class": "", "styles": {}}]}]
Output:
{"components": [{"nodeId": "node_5", "type": "Card", "confidence": 0.9, "reasoning": "div with card class, border-radius and box-shadow"}]}

Example 3:
Input:
[{"nodeId": "node_10", "tag": "section", "class": "hero", "styles": {"padding": "6rem 2rem", "text-align": "center", "background": "#1a1a2e"}, "children": [{"nodeId": "node_11", "tag": "h1", "class": "", "styles": {"font-size": "3rem"}}]}]
Output:
{"components": [{"nodeId": "node_10", "type": "Hero", "confidence": 0.95, "reasoning": "hero class with large centered padding and dark background"}]}

Example 4:
Input:
[{"nodeId": "node_20", "tag": "div", "class": "pricing-card", "styles": {"border": "1px solid #e0e0e0"}, "children": [{"nodeId": "node_21", "tag": "h3", "class": "price", "styles": {}}, {"nodeId": "node_22", "tag": "ul", "class": "features", "styles": {}}]}]
Output:
{"components": [{"nodeId": "node_20", "type": "Pricing", "confidence": 0.85, "reasoning": "pricing-card class with price and feature list"}]}

Example 5:
Input:
[{"nodeId": "node_30", "tag": "form", "class": "contact-form", "styles": {"display": "flex", "flex-direction": "column", "gap": "1rem"}, "children": [{"nodeId": "node_31", "tag": "input", "class": "", "styles": {}}, {"nodeId": "node_32", "tag": "button", "class": "submit", "styles": {}}]}]
Output:
{"components": [{"nodeId": "node_30", "type": "Form", "confidence": 0.95, "reasoning": "form tag with contact-form class and input fields"}]}

Example 6:
Input:
[{"nodeId": "node_40", "tag": "aside", "class": "sidebar", "styles": {"width": "250px", "position": "fixed"}, "children": [{"nodeId": "node_41", "tag": "nav", "class": "side-nav", "styles": {}}]}]
Output:
{"components": [{"nodeId": "node_40", "type": "Sidebar", "confidence": 0.95, "reasoning": "aside with sidebar class and fixed positioning"}]}`;

function buildPrompt(nodes: SerializedNode[]): string {
  const input = JSON.stringify(nodes, null, 2);

  return `You are a semantic UI component detector. Analyze the following HTML/CSS structure and identify semantic components.

${FEW_SHOT_EXAMPLES}

Rules:
- Return ONLY valid JSON matching the schema {"components": [{"nodeId": string, "type": string, "confidence": number, "reasoning"?: string}]}
- nodeId MUST be one of the nodeIds from the input — never invent IDs
- type must be one of: Card, Hero, Navbar, Dialog, Form, Sidebar, Footer, Header, List, Grid, Pricing, Dashboard, Marketing, ProductCard
- confidence must be between 0 and 1
- Only detect components you are highly confident about (confidence > 0.6)
- Do not include markdown, code fences, explanations, or conversation
- Do not generate Flutter, SwiftUI, or Compose code
- Return platform-neutral JSON only

Input:
${input.slice(0, 8000)}`;
}

// -- Retry Logic --

interface RetryConfig {
  maxRetries: number;
  baseDelayMs: number;
  maxDelayMs: number;
}

async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function withRetry<T>(
  fn: () => Promise<T>,
  config: RetryConfig,
): Promise<T> {
  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= config.maxRetries; attempt++) {
    try {
      return await fn();
    } catch (err) {
      lastError = err instanceof Error ? err : new Error(String(err));
      if (attempt < config.maxRetries) {
        const delay = Math.min(
          config.baseDelayMs * Math.pow(2, attempt),
          config.maxDelayMs,
        );
        console.warn(
          `Ollama attempt ${attempt + 1}/${config.maxRetries + 1} failed: ${lastError.message}. Retrying in ${delay}ms...`,
        );
        await sleep(delay);
      }
    }
  }

  throw lastError ?? new Error('Retry exhausted');
}

// -- Batching --

function splitIntoBatches<T>(items: T[], batchSize: number): T[][] {
  const batches: T[][] = [];
  for (let i = 0; i < items.length; i += batchSize) {
    batches.push(items.slice(i, i + batchSize));
  }
  return batches;
}

// -- AI Response Parsing and Validation --

function parseAiResponse(text: string, validNodeIds: Set<string>): SemanticHint[] {
  try {
    const parsed = JSON.parse(text);
    const { components } = AiResponseSchema.parse(parsed);

    return components
      .filter(c => validNodeIds.has(c.nodeId))
      .map(c => ({
        type: c.type as any,
        confidence: c.confidence,
        node: { nodeId: c.nodeId, tagName: 'div', attributes: [], children: [] } as HtmlNode,
        reason: c.reasoning || `AI detected ${c.type}`,
      }));
  } catch {
    return [];
  }
}

// -- Hint Merge Strategy --

function mergeHints(aiHints: SemanticHint[], ruleHints: SemanticHint[]): SemanticHint[] {
  const combined = [...aiHints, ...ruleHints];
  combined.sort((a, b) => b.confidence - a.confidence);

  const seen = new Set<string>();
  return combined.filter(h => {
    const key = `${h.type}-${h.node.nodeId}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

// -- Main AI Detector --

export function createAiDetector(config?: AiDetectorConfig): SemanticDetector {
  const cfg = resolveConfig(config);

  return async (nodes: StyledNode[]): Promise<SemanticHint[]> => {
    const serialized = serializeStyledNode(nodes);
    const validNodeIds = new Set<string>();
    function collectIds(nodes: SerializedNode[]): void {
      for (const n of nodes) {
        validNodeIds.add(n.nodeId);
        collectIds(n.children);
      }
    }
    collectIds(serialized);

    try {
      const batches = splitIntoBatches(serialized, cfg.batchSize);
      const allAiHints: SemanticHint[] = [];

      for (const batch of batches) {
        const batchNodeIds = new Set<string>();
        function collectBatchIds(nodes: SerializedNode[]): void {
          for (const n of nodes) {
            batchNodeIds.add(n.nodeId);
            collectBatchIds(n.children);
          }
        }
        collectBatchIds(batch);

        const prompt = buildPrompt(batch);

        const responseText = await withRetry(
          async () => {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), cfg.timeout);

            try {
              const response = await fetch(`${cfg.baseUrl}/api/generate`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                  model: cfg.model,
                  prompt,
                  stream: false,
                  options: { temperature: 0.1 },
                }),
                signal: controller.signal,
              });

              if (!response.ok) {
                throw new Error(`Ollama returned ${response.status}`);
              }

              const data = await response.json() as { response?: string };
              if (!data.response) {
                throw new Error('Empty response from Ollama');
              }

              return data.response;
            } finally {
              clearTimeout(timeoutId);
            }
          },
          { maxRetries: cfg.retryCount, baseDelayMs: 1000, maxDelayMs: 10000 },
        );

        const batchHints = parseAiResponse(responseText, batchNodeIds);
        allAiHints.push(...batchHints);
      }

      const ruleHints = unwrapDetectSemantics(nodes);

      if (allAiHints.length === 0) {
        return ruleHints;
      }

      return mergeHints(allAiHints, ruleHints);
    } catch (err) {
      console.warn(
        `Ollama unavailable (${(err as Error).message}), falling back to rule-based detector`,
      );
      return unwrapDetectSemantics(nodes);
    }
  };
}
