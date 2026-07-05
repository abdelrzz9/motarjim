import { Icon } from '../../../components/Icons';
import { Button } from '../../../design-system';

interface EmptyStateProps {
  onLoadSample: () => void;
  onUpload: () => void;
}

export function EmptyState({ onLoadSample, onUpload }: EmptyStateProps) {
  return (
    <div style={{
      flex: 1,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      padding: 'var(--space-8)',
    }}>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 16,
        textAlign: 'center',
        maxWidth: 400,
        animation: 'fade-in 300ms var(--ease-out)',
      }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <h2 style={{
            fontSize: 'var(--text-xl)',
            fontWeight: 700,
            color: 'var(--text-primary)',
            letterSpacing: '-0.02em',
            lineHeight: 1.3,
          }}>
            Convert HTML/CSS into Native UI
          </h2>
          <p style={{
            fontSize: 'var(--text-base)',
            color: 'var(--text-secondary)',
            lineHeight: 1.6,
          }}>
            Paste HTML and CSS in the editor, or load a sample project to get started.
            Generate Flutter, Jetpack Compose, or SwiftUI code.
          </p>
        </div>

        <div style={{ display: 'flex', gap: 8, marginTop: 4 }}>
          <Button variant="primary" size="md" icon={<Icon.Folder size={14} />} onClick={onLoadSample}>
            Load Sample
          </Button>
          <Button variant="secondary" size="md" icon={<Icon.Upload size={14} />} onClick={onUpload}>
            Open File
          </Button>
        </div>

        <div style={{
          display: 'flex',
          gap: 24,
          marginTop: 4,
          padding: 'var(--space-3) var(--space-4)',
          background: 'var(--bg-elevated)',
          borderRadius: 'var(--radius-md)',
          border: '1px solid var(--border-subtle)',
        }}>
          {[
            { icon: Icon.Flutter, label: 'Flutter', color: 'var(--flutter)' },
            { icon: Icon.Kotlin, label: 'Compose', color: 'var(--compose)' },
            { icon: Icon.Swift, label: 'SwiftUI', color: 'var(--swiftui)' },
          ].map((p) => {
            const PlatformIcon = p.icon;
            return (
              <div key={p.label} style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: 4,
              }}>
                <span style={{ color: p.color, opacity: 0.7 }}>
                  <PlatformIcon size={20} />
                </span>
                <span style={{ fontSize: 10, color: 'var(--text-tertiary)', fontWeight: 600 }}>
                  {p.label}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
