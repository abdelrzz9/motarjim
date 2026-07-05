import { useState, useEffect, useRef, useCallback } from 'react';
import { Icon } from './Icons';

interface Command {
  id: string;
  label: string;
  description?: string;
  shortcut?: string;
  icon?: typeof Icon.Search;
  action: () => void;
}

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  commands: Command[];
}

export function CommandPalette({ open, onClose, commands }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const filtered = commands.filter(c =>
    c.label.toLowerCase().includes(query.toLowerCase()) ||
    c.description?.toLowerCase().includes(query.toLowerCase()),
  );

  useEffect(() => {
    if (open) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(i => Math.min(i + 1, filtered.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(i => Math.max(i - 1, 0));
    } else if (e.key === 'Enter' && filtered[selectedIndex]) {
      e.preventDefault();
      filtered[selectedIndex].action();
      onClose();
    }
  }, [filtered, selectedIndex, onClose]);

  if (!open) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        background: 'var(--bg-overlay)',
        display: 'flex',
        justifyContent: 'center',
        paddingTop: '12vh',
        zIndex: 300,
        animation: 'fade-in 120ms var(--ease-out)',
      }}
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div
        style={{
          width: '90%',
          maxWidth: 560,
          background: 'var(--bg-surface)',
          border: '1px solid var(--border-default)',
          borderRadius: 'var(--radius-lg)',
          boxShadow: 'var(--shadow-xl)',
          overflow: 'hidden',
          height: 'fit-content',
          maxHeight: '60vh',
          display: 'flex',
          flexDirection: 'column',
          animation: 'slide-down 120ms var(--ease-out)',
        }}
      >
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: 10,
          padding: 'var(--space-3) var(--space-4)',
          borderBottom: '1px solid var(--border-subtle)',
        }}>
          <Icon.Search size={14} style={{ color: 'var(--text-tertiary)', flexShrink: 0 }} />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => { setQuery(e.target.value); setSelectedIndex(0); }}
            onKeyDown={handleKeyDown}
            placeholder="Search commands..."
            aria-label="Search commands"
            style={{
              flex: 1,
              border: 'none',
              background: 'transparent',
              color: 'var(--text-primary)',
              fontSize: 13,
              outline: 'none',
              fontFamily: 'inherit',
            }}
          />
          <kbd style={{
            fontSize: 10,
            padding: '2px 5px',
            background: 'var(--bg-elevated)',
            borderRadius: 4,
            color: 'var(--text-tertiary)',
            border: '1px solid var(--border-default)',
          }}>
            ESC
          </kbd>
        </div>

        <div style={{ overflow: 'auto', flex: 1, padding: 4 }}>
          {filtered.length === 0 ? (
            <div style={{ padding: 'var(--space-6)', textAlign: 'center', color: 'var(--text-tertiary)', fontSize: 12 }}>
              No commands found
            </div>
          ) : (
            filtered.map((cmd, i) => {
              const IconComp = cmd.icon;
              return (
                <button
                  key={cmd.id}
                  onClick={() => { cmd.action(); onClose(); }}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 10,
                    width: '100%',
                    border: 'none',
                    background: i === selectedIndex ? 'var(--bg-hover)' : 'transparent',
                    color: 'var(--text-primary)',
                    padding: '8px 10px',
                    borderRadius: 6,
                    cursor: 'pointer',
                    fontSize: 12,
                    textAlign: 'left',
                    fontFamily: 'inherit',
                    transition: 'background 80ms var(--ease-out)',
                  }}
                  onMouseEnter={() => setSelectedIndex(i)}
                >
                  {IconComp && (
                    <span style={{ width: 16, height: 16, display: 'flex', color: 'var(--text-tertiary)', flexShrink: 0 }}>
                      <IconComp size={14} />
                    </span>
                  )}
                  <div style={{ flex: 1 }}>
                    <div>{cmd.label}</div>
                    {cmd.description && (
                      <div style={{ fontSize: 10, color: 'var(--text-tertiary)', marginTop: 1 }}>
                        {cmd.description}
                      </div>
                    )}
                  </div>
                  {cmd.shortcut && (
                    <kbd style={{
                      fontSize: 10,
                      padding: '2px 5px',
                      background: 'var(--bg-elevated)',
                      borderRadius: 4,
                      color: 'var(--text-tertiary)',
                      border: '1px solid var(--border-default)',
                    }}>
                      {cmd.shortcut}
                    </kbd>
                  )}
                </button>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
