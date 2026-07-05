import { useState, useRef, useEffect, type ReactNode } from 'react';

interface DropdownItem {
  id: string;
  label: string;
  icon?: ReactNode;
  shortcut?: string;
  danger?: boolean;
  divider?: boolean;
  onClick: () => void;
}

interface DropdownProps {
  trigger: ReactNode;
  items: DropdownItem[];
  align?: 'left' | 'right';
}

export function Dropdown({ trigger, items, align = 'left' }: DropdownProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener('mousedown', handler);
    return () => window.removeEventListener('mousedown', handler);
  }, [open]);

  return (
    <div ref={ref} style={{ position: 'relative', display: 'inline-flex' }}>
      <div onClick={() => setOpen(!open)}>{trigger}</div>
      {open && (
        <div
          style={{
            position: 'absolute',
            top: 'calc(100% + 4px)',
            [align]: 0,
            background: 'var(--bg-surface)',
            border: '1px solid var(--border-default)',
            borderRadius: 'var(--radius-md)',
            boxShadow: 'var(--shadow-lg)',
            minWidth: 180,
            zIndex: 100,
            padding: 4,
            animation: 'slide-up 120ms var(--ease-out)',
          }}
        >
          {items.map((item, i) => {
            if (item.divider) {
              return (
                <div
                  key={item.id || `divider-${i}`}
                  style={{ height: 1, background: 'var(--border-subtle)', margin: '4px 0' }}
                />
              );
            }
            return (
              <button
                key={item.id}
                onClick={() => { item.onClick(); setOpen(false); }}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  width: '100%',
                  border: 'none',
                  background: 'transparent',
                  color: item.danger ? 'var(--error)' : 'var(--text-secondary)',
                  fontSize: 12,
                  padding: '6px 8px',
                  borderRadius: 5,
                  cursor: 'pointer',
                  transition: 'all 80ms var(--ease-out)',
                  fontFamily: 'inherit',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = item.danger ? 'var(--error-soft)' : 'var(--bg-hover)';
                  e.currentTarget.style.color = item.danger ? 'var(--error)' : 'var(--text-primary)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'transparent';
                  e.currentTarget.style.color = item.danger ? 'var(--error)' : 'var(--text-secondary)';
                }}
              >
                {item.icon && <span style={{ display: 'flex', width: 16, height: 16, flexShrink: 0 }}>{item.icon}</span>}
                <span style={{ flex: 1, textAlign: 'left' }}>{item.label}</span>
                {item.shortcut && (
                  <span style={{ fontSize: 10, color: 'var(--text-tertiary)' }}>{item.shortcut}</span>
                )}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
