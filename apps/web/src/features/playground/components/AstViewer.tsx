import { useState } from 'react';
import { usePlaygroundStore } from '../../../stores/playgroundStore';

interface TreeNode {
  key: string;
  value: unknown;
  depth: number;
}

function flattenTree(obj: unknown, depth = 0): TreeNode[] {
  const nodes: TreeNode[] = [];
  if (obj === null || obj === undefined) {
    nodes.push({ key: '', value: String(obj), depth });
    return nodes;
  }
  if (typeof obj !== 'object') {
    nodes.push({ key: '', value: String(obj), depth });
    return nodes;
  }
  if (Array.isArray(obj)) {
    if (obj.length === 0) {
      nodes.push({ key: '', value: '[]', depth });
    } else {
      obj.forEach((item, index) => {
        nodes.push({ key: `[${index}]`, value: item, depth });
        nodes.push(...flattenTree(item, depth + 1));
      });
    }
    return nodes;
  }
  const entries = Object.entries(obj as Record<string, unknown>);
  if (entries.length === 0) {
    nodes.push({ key: '', value: '{}', depth });
  } else {
    entries.forEach(([k, v]) => {
      nodes.push({ key: k, value: v, depth });
      nodes.push(...flattenTree(v, depth + 1));
    });
  }
  return nodes;
}

export default function AstViewer() {
  const { ast } = usePlaygroundStore();
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());

  if (!ast) {
    return (
      <div style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-muted)' }}>
        No AST available
      </div>
    );
  }

  const nodes = flattenTree(ast);

  const toggleCollapse = (path: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const isObject = (val: unknown): val is Record<string, unknown> | unknown[] =>
    val !== null && typeof val === 'object';

  return (
    <div style={{ padding: '0.5rem', fontFamily: 'var(--font-mono)', fontSize: '0.8rem' }}>
      {nodes.map((node, i) => {
        const indent = node.depth * 1.25;
        return (
          <div
            key={i}
            style={{
              marginLeft: `${indent}rem`,
              display: 'flex',
              alignItems: 'center',
              gap: '0.375rem',
              lineHeight: '1.6',
            }}
          >
            {node.key && (
              <span style={{ color: 'var(--accent)' }}>{node.key}:</span>
            )}
            {isObject(node.value) ? (
              <button
                onClick={() => toggleCollapse(`${i}`)}
                style={{
                  background: 'none',
                  border: 'none',
                  cursor: 'pointer',
                  color: 'var(--text-primary)',
                  fontFamily: 'var(--font-mono)',
                  fontSize: '0.8rem',
                }}
              >
                {collapsed.has(`${i}`) ? '{...}' : '{'}
              </button>
            ) : (
              <span style={{ color: 'var(--text-secondary)' }}>
                {String(node.value)}
              </span>
            )}
          </div>
        );
      })}
    </div>
  );
}
