import { useState } from 'react';
import { Icon } from '../../../components/Icons';
import type { Diagnostic } from '../../../services/types';

interface ErrorStateProps {
  diagnostics: Diagnostic[];
}

function SeverityIcon({ severity }: { severity: string }) {
  const props = { size: 16 };
  switch (severity) {
    case 'error': return <Icon.Error {...props} />;
    case 'warning': return <Icon.Warning {...props} />;
    default: return <Icon.Info {...props} />;
  }
}

function SeverityColor(severity: string): string {
  switch (severity) {
    case 'error': return 'var(--error)';
    case 'warning': return 'var(--warning)';
    default: return 'var(--info)';
  }
}

export function ErrorState({ diagnostics }: ErrorStateProps) {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  return (
    <div style={{
      padding: 'var(--space-3)',
      display: 'flex',
      flexDirection: 'column',
      gap: 6,
      animation: 'fade-in 200ms var(--ease-out)',
    }}>
      {diagnostics.map((d, i) => {
        const isExpanded = expandedId === `${i}`;
        return (
          <div
            key={i}
            style={{
              background: 'var(--bg-surface)',
              border: `1px solid ${d.severity === 'error' ? 'rgba(239,68,68,0.2)' : d.severity === 'warning' ? 'rgba(245,166,35,0.2)' : 'var(--border-default)'}`,
              borderRadius: 'var(--radius-md)',
              overflow: 'hidden',
              transition: 'all 200ms var(--ease-out)',
            }}
          >
            <button
              onClick={() => setExpandedId(isExpanded ? null : `${i}`)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 10,
                width: '100%',
                border: 'none',
                background: 'transparent',
                padding: '10px 12px',
                cursor: 'pointer',
                textAlign: 'left',
                fontFamily: 'inherit',
                color: 'var(--text-primary)',
              }}
            >
              <span style={{
                display: 'flex',
                flexShrink: 0,
                color: SeverityColor(d.severity),
              }}>
                <SeverityIcon severity={d.severity} />
              </span>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  fontSize: 12,
                  fontWeight: 600,
                  color: d.severity === 'error' ? 'var(--error)' : 'var(--text-primary)',
                }}>
                  {d.title}
                  {d.location && (
                    <span style={{
                      fontSize: 10,
                      color: 'var(--text-tertiary)',
                      fontWeight: 500,
                      background: 'var(--bg-elevated)',
                      padding: '1px 5px',
                      borderRadius: 4,
                    }}>
                      Line {d.location.start.line}
                    </span>
                  )}
                  <span style={{
                    fontSize: 10,
                    color: 'var(--text-tertiary)',
                    fontWeight: 500,
                    fontFamily: 'var(--font-mono)',
                  }}>
                    {d.code}
                  </span>
                </div>
                <div style={{
                  fontSize: 11,
                  color: 'var(--text-secondary)',
                  marginTop: 2,
                  lineHeight: 1.4,
                }}>
                  {d.explanation}
                </div>
              </div>
              <span style={{
                color: 'var(--text-tertiary)',
                transform: isExpanded ? 'rotate(180deg)' : 'rotate(0deg)',
                transition: 'transform 200ms var(--ease-out)',
                display: 'flex',
              }}>
                <Icon.ChevronDown size={14} />
              </span>
            </button>

            {isExpanded && (
              <div style={{
                padding: '0 12px 10px 38px',
                animation: 'slide-up 150ms var(--ease-out)',
              }}>
                {d.suggestions.length > 0 && (
                  <div style={{ marginTop: 6 }}>
                    <div style={{
                      fontSize: 10,
                      fontWeight: 600,
                      color: 'var(--text-tertiary)',
                      textTransform: 'uppercase',
                      letterSpacing: '0.05em',
                      marginBottom: 4,
                    }}>
                      Possible solutions
                    </div>
                    <ul style={{
                      listStyle: 'none',
                      padding: 0,
                      margin: 0,
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 3,
                    }}>
                      {d.suggestions.map((s, j) => (
                        <li key={j} style={{
                          fontSize: 11,
                          color: 'var(--text-secondary)',
                          paddingLeft: 12,
                          position: 'relative',
                        }}>
                          <span style={{
                            position: 'absolute',
                            left: 0,
                            top: 5,
                            width: 4,
                            height: 4,
                            borderRadius: '50%',
                            background: 'var(--accent)',
                          }} />
                          {s}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
                <div style={{
                  marginTop: 8,
                  display: 'flex',
                  gap: 6,
                }}>
                  <button
                    onClick={() => {
                      navigator.clipboard.writeText(
                        `[${d.code}] ${d.title}: ${d.explanation}`
                      );
                    }}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 4,
                      border: 'none',
                      background: 'var(--bg-elevated)',
                      color: 'var(--text-tertiary)',
                      fontSize: 10,
                      padding: '4px 8px',
                      borderRadius: 4,
                      cursor: 'pointer',
                      fontFamily: 'inherit',
                    }}
                  >
                    <Icon.Copy size={11} />
                    Copy error
                  </button>
                </div>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
