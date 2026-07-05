import { usePlaygroundStore } from '../../../stores/playgroundStore';
import type { Diagnostic } from '../../../services/types';
import styles from './DiagnosticsPanel.module.css';

const SEVERITY_COLORS: Record<string, string> = {
  error: 'var(--error)',
  warning: 'var(--warning)',
  info: 'var(--info)',
  hint: 'var(--success)',
  note: 'var(--text-tertiary)',
};

const SEVERITY_LABELS: Record<string, string> = {
  error: '✕',
  warning: '⚠',
  info: 'ℹ',
  hint: '💡',
  note: '→',
};

export default function DiagnosticsPanel() {
  const { diagnostics } = usePlaygroundStore();

  if (diagnostics.length === 0) {
    return (
      <div className={styles.empty}>
        <div className={styles.emptyIcon}>✓</div>
        <div className={styles.emptyText}>No diagnostics</div>
      </div>
    );
  }

  return (
    <div className={styles.list}>
      {diagnostics.map((diag, i) => (
        <DiagnosticItem key={`${diag.code}-${i}`} diagnostic={diag} index={i} />
      ))}
    </div>
  );
}

function DiagnosticItem({ diagnostic: d }: { diagnostic: Diagnostic; index?: number }) {
  const loc = d.location
    ? `Line ${d.location.start.line}, Col ${d.location.start.column}`
    : null;

  return (
    <div className={`${styles.item} ${styles[d.severity] || ''}`}>
      <div className={styles.itemHeader}>
        <span className={styles.severity} style={{ color: SEVERITY_COLORS[d.severity] }}>
          {SEVERITY_LABELS[d.severity]}
        </span>
        <span className={styles.code}>{d.code}</span>
        <span className={styles.title}>{d.title}</span>
      </div>
      <div className={styles.explanation}>{d.explanation}</div>
      {loc && (
        <div className={styles.location}>at {loc}</div>
      )}
      {d.suggestions.length > 0 && (
        <div className={styles.suggestions}>
          {d.suggestions.map((s, j) => (
            <div key={j} className={styles.suggestion}>
              Suggestion: {s}
            </div>
          ))}
        </div>
      )}
      {d.notes.length > 0 && (
        <div className={styles.notes}>
          {d.notes.map((n, j) => (
            <div key={j} className={styles.note}>
              Note: {n}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
