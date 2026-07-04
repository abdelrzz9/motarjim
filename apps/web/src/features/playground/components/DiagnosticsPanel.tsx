import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { SEVERITY_ICONS } from '../../../utils/constants';

const SEVERITY_COLORS: Record<string, string> = {
  error: 'var(--error)',
  warning: 'var(--warning)',
  info: 'var(--info)',
  hint: 'var(--success)',
  note: 'var(--text-muted)',
};

export default function DiagnosticsPanel() {
  const { diagnostics } = usePlaygroundStore();

  if (diagnostics.length === 0) {
    return (
      <div style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-muted)' }}>
        No diagnostics
      </div>
    );
  }

  return (
    <div>
      {diagnostics.map((diag, i) => (
        <div
          key={i}
          style={{
            padding: '0.625rem 1rem',
            borderBottom: '1px solid var(--border)',
            fontSize: '0.85rem',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span style={{ color: SEVERITY_COLORS[diag.severity] }}>
              {SEVERITY_ICONS[diag.severity]}
            </span>
            <span style={{ fontWeight: 600 }}>{diag.code}</span>
            <span style={{ color: 'var(--text-secondary)' }}>{diag.message}</span>
          </div>
          {diag.suggestions.length > 0 && (
            <div style={{ marginTop: '0.25rem', paddingLeft: '1.5rem', color: 'var(--text-muted)', fontSize: '0.8rem' }}>
              {diag.suggestions.map((s, j) => (
                <div key={j}>Suggestion: {s}</div>
              ))}
            </div>
          )}
          {diag.notes.length > 0 && (
            <div style={{ marginTop: '0.25rem', paddingLeft: '1.5rem', color: 'var(--text-muted)', fontSize: '0.8rem' }}>
              {diag.notes.map((n, j) => (
                <div key={j}>Note: {n}</div>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
