import { useState } from 'react';
import { usePlaygroundStore } from '../../../stores/playgroundStore';
import styles from './AstViewer.module.css';

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
      <div className={styles.empty}>
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

  const renderValue = (value: unknown, _key: string, index: number) => {
    if (isObject(value)) {
      const isCollapsed = collapsed.has(`${index}`);
      return (
        <button
          className={styles.toggleBtn}
          onClick={() => toggleCollapse(`${index}`)}
        >
          {isCollapsed ? '{...}' : '{'}
        </button>
      );
    }
    return (
      <span className={styles.value}>
        {String(value)}
      </span>
    );
  };

  return (
    <div className={styles.tree}>
      {nodes.map((node, i) => {
        const indent = node.depth * 16;
        return (
          <div
            key={i}
            className={styles.node}
            style={{ paddingLeft: `${indent}px` }}
          >
            {node.key && (
              <span className={styles.key}>{node.key}:</span>
            )}
            {renderValue(node.value, node.key, i)}
          </div>
        );
      })}
    </div>
  );
}
