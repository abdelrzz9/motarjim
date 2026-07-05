import { Modal } from '../design-system';

interface ShortcutGroup {
  title: string;
  shortcuts: { keys: string; action: string }[];
}

const GROUPS: ShortcutGroup[] = [
  {
    title: 'Compilation',
    shortcuts: [
      { keys: 'Ctrl+Enter', action: 'Compile code' },
      { keys: 'Ctrl+S', action: 'Save / Compile' },
      { keys: 'Ctrl+Shift+F', action: 'Format code' },
    ],
  },
  {
    title: 'Editing',
    shortcuts: [
      { keys: 'Ctrl+C', action: 'Copy output' },
      { keys: 'Ctrl+Z', action: 'Undo' },
      { keys: 'Ctrl+Shift+Z', action: 'Redo' },
    ],
  },
  {
    title: 'Navigation',
    shortcuts: [
      { keys: 'Ctrl+K', action: 'Command palette' },
      { keys: 'Ctrl+1', action: 'Focus HTML editor' },
      { keys: 'Ctrl+2', action: 'Focus CSS editor' },
    ],
  },
  {
    title: 'General',
    shortcuts: [
      { keys: 'Escape', action: 'Close modals / dialogs' },
      { keys: '?', action: 'Show keyboard shortcuts' },
    ],
  },
];

interface KeyboardShortcutModalProps {
  open: boolean;
  onClose: () => void;
}

export function KeyboardShortcutModal({ open, onClose }: KeyboardShortcutModalProps) {
  return (
    <Modal open={open} onClose={onClose} title="Keyboard Shortcuts" width={420}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
        {GROUPS.map((group) => (
          <div key={group.title}>
            <h4 style={{
              fontSize: 10,
              fontWeight: 600,
              textTransform: 'uppercase',
              letterSpacing: '0.06em',
              color: 'var(--text-tertiary)',
              marginBottom: 8,
            }}>
              {group.title}
            </h4>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
              {group.shortcuts.map((s) => (
                <div
                  key={s.keys}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '5px 0',
                  }}
                >
                  <span style={{ fontSize: 12, color: 'var(--text-secondary)' }}>{s.action}</span>
                  <kbd style={{
                    display: 'flex',
                    gap: 4,
                    alignItems: 'center',
                  }}>
                    {s.keys.split('+').map((key, i) => (
                      <span key={i} style={{
                        display: 'inline-flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        minWidth: 22,
                        height: 20,
                        padding: '0 5px',
                        background: 'var(--bg-elevated)',
                        borderRadius: 4,
                        fontSize: 10,
                        fontWeight: 600,
                        color: 'var(--text-secondary)',
                        border: '1px solid var(--border-default)',
                      }}>
                        {key}
                      </span>
                    ))}
                  </kbd>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </Modal>
  );
}
