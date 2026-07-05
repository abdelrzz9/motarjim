import { usePlaygroundStore } from '../stores/playgroundStore';
import { StatusDot, Badge } from '../design-system';
import { formatMs } from '../utils/formatting';

export function StatusBar() {
  const { stats, backendOnline, diagnostics } = usePlaygroundStore();

  const errors = diagnostics.filter(d => d.severity === 'error').length;
  const warnings = diagnostics.filter(d => d.severity === 'warning').length;

  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      padding: '0 var(--space-3)',
      height: 26,
      borderTop: '1px solid var(--border-subtle)',
      background: 'var(--bg-surface)',
      flexShrink: 0,
      fontSize: 10,
      color: 'var(--text-tertiary)',
      userSelect: 'none',
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-3)' }}>
        <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
          <StatusDot variant={backendOnline ? 'success' : 'error'} />
          Backend: {backendOnline ? 'Online' : 'Offline'}
        </span>
        {stats && (
          <>
            <span style={{ opacity: 0.4 }}>|</span>
            <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
              Generated in {formatMs(stats.timeMs)}
            </span>
            <span style={{ opacity: 0.4 }}>|</span>
            <span>{stats.nodesParsed} nodes · {stats.cssRules} rules</span>
          </>
        )}
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
        {errors > 0 && <Badge variant="error">{errors} error{errors !== 1 ? 's' : ''}</Badge>}
        {warnings > 0 && <Badge variant="warning">{warnings} warning{warnings !== 1 ? 's' : ''}</Badge>}
        <span style={{ opacity: 0.5 }}>UTF-8</span>
      </div>
    </div>
  );
}
