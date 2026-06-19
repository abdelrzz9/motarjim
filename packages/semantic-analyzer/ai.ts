// AI-powered semantic detection using Ollama (optional, local-only).
// Falls back to the rule-based detectSemantics() if Ollama is unavailable.
// Must never generate platform code — only semantic hints.

import { StyledNode, SemanticHint, HtmlNode } from '@html-native/shared';
import { detectSemantics } from './index.js';

export type SemanticDetector = (nodes: StyledNode[]) => Promise<SemanticHint[]>;

export interface AiDetectorConfig {
  model?: string;
  baseUrl?: string;
  timeout?: number;
}

const DEFAULT_CONFIG: Required<AiDetectorConfig> = {
  model: 'qwen2.5:7b',
  baseUrl: 'http://localhost:11434',
  timeout: 30000,
};

function styledToBriefJson(nodes: StyledNode[]): unknown[] {
  function walk(node: StyledNode): unknown {
    return {
      tag: node.node.tagName,
      class: node.node.attributes.find(a => a.name === 'class')?.value || '',
      id: node.node.attributes.find(a => a.name === 'id')?.value || '',
      styles: Object.fromEntries(
        Object.entries(node.styles).filter(([_, v]) => v)
      ),
      children: node.children.map(walk),
    };
  }
  return nodes.map(walk);
}

function parseAiResponse(text: string): SemanticHint[] {
  try {
    const data = JSON.parse(text);
    const items = Array.isArray(data) ? data : (data.hints || data.semantics || []);
    return items.map((item: any, i: number) => ({
      type: item.type || 'Unknown',
      confidence: item.confidence ?? 0.5,
      node: { nodeId: `ai_${i}`, tagName: 'div', attributes: [], children: [] } as HtmlNode,
      reason: item.reason || 'AI detected',
    }));
  } catch {
    return [];
  }
}

export function createAiDetector(config?: AiDetectorConfig): SemanticDetector {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  return async (nodes: StyledNode[]): Promise<SemanticHint[]> => {
    const htmlBrief = JSON.stringify(styledToBriefJson(nodes), null, 2);

    const prompt = `Analyze the following HTML and CSS structure.

Return ONLY a JSON array of detected semantic components.

Detect:
- cards
- navigation
- forms
- hero sections
- dialogs
- reusable components

Format:
[{ "type": "Card", "confidence": 0.95, "reason": "card class with shadow" }]

Do not generate Flutter.
Do not generate SwiftUI.
Do not generate Compose.
Return platform-neutral JSON only.

Input:
${htmlBrief.slice(0, 3000)}`;

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), cfg.timeout);

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

      clearTimeout(timeoutId);

      if (!response.ok) {
        console.warn(`Ollama returned ${response.status}, falling back to rule-based detector`);
        return detectSemantics(nodes);
      }

      const data = await response.json() as { response?: string };
      if (!data.response) {
        return detectSemantics(nodes);
      }

      const aiHints = parseAiResponse(data.response);
      if (aiHints.length === 0) {
        return detectSemantics(nodes);
      }

      const ruleHints = detectSemantics(nodes);
      const combined = [...aiHints, ...ruleHints];
      combined.sort((a, b) => b.confidence - a.confidence);

      const seen = new Set<string>();
      return combined.filter(h => {
        const key = `${h.type}-${h.node.nodeId}`;
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      });
    } catch (err) {
      console.warn(`Ollama unavailable (${(err as Error).message}), falling back to rule-based detector`);
      return detectSemantics(nodes);
    }
  };
}
