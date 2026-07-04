import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { formatMs, pluralize } from '../../../utils/formatting';

export default function StatusBar() {
  const { stats, isCompiling, diagnostics } = usePlaygroundStore();
  const errors = diagnostics.filter((d) => d.severity === 'error').length;
  const warnings = diagnostics.filter((d) => d.severity === 'warning').length;

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '1rem',
        padding: '0.25rem 1rem',
        background: 'var(--bg-secondary)',
        borderTop: '1px solid var(--border)',
        fontSize: '0.75rem',
        color: 'var(--text-muted)',
      }}
    >
      {isCompiling && <span>Compiling...</span>}

      {stats && !isCompiling && (
        <>
          <span>{pluralize(stats.nodes_parsed, 'node')} parsed</span>
          <span>{pluralize(stats.css_rules, 'CSS rule')}</span>
          <span>{pluralize(stats.ir_nodes, 'IR node')}</span>
          <span>{formatMs(stats.time_ms)}</span>
        </>
      )}

      <div style={{ flex: 1 }} />

      {errors > 0 && <span style={{ color: 'var(--error)' }}>{errors} error{errors !== 1 ? 's' : ''}</span>}
      {warnings > 0 && <span style={{ color: 'var(--warning)' }}>{warnings} warning{warnings !== 1 ? 's' : ''}</span>}

      {!isCompiling && !stats && errors === 0 && (
        <span>Ready</span>
      )}
    </div>
  );
}
