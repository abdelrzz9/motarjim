import type { ReactNode } from 'react';

interface Tab {
  id: string;
  label: string;
  icon?: ReactNode;
  badge?: number | string;
}

interface TabsProps {
  tabs: Tab[];
  activeId: string;
  onChange: (id: string) => void;
  size?: 'sm' | 'md';
}

export function Tabs({ tabs, activeId, onChange, size = 'sm' }: TabsProps) {
  return (
    <div
      role="tablist"
      style={{
        display: 'flex',
        gap: 1,
        background: 'var(--bg-elevated)',
        borderRadius: 'var(--radius-sm)',
        padding: 2,
      }}
    >
      {tabs.map((tab) => {
        const isActive = tab.id === activeId;
        return (
          <button
            key={tab.id}
            role="tab"
            aria-selected={isActive}
            onClick={() => onChange(tab.id)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 5,
              border: 'none',
              background: isActive ? 'var(--bg-active)' : 'transparent',
              color: isActive ? 'var(--text-primary)' : 'var(--text-tertiary)',
              fontSize: size === 'sm' ? 11 : 12,
              fontWeight: 500,
              padding: size === 'sm' ? '3px 8px' : '5px 12px',
              borderRadius: 5,
              cursor: 'pointer',
              transition: 'all 120ms var(--ease-out)',
              whiteSpace: 'nowrap',
              textTransform: 'uppercase',
              letterSpacing: '0.03em',
              fontFamily: 'inherit',
            }}
            onMouseEnter={(e) => {
              if (!isActive) {
                e.currentTarget.style.background = 'var(--bg-hover)';
                e.currentTarget.style.color = 'var(--text-secondary)';
              }
            }}
            onMouseLeave={(e) => {
              if (!isActive) {
                e.currentTarget.style.background = 'transparent';
                e.currentTarget.style.color = 'var(--text-tertiary)';
              }
            }}
          >
            {tab.icon && <span style={{ display: 'flex', width: 14, height: 14 }}>{tab.icon}</span>}
            {tab.label}
            {tab.badge !== undefined && (
              <span
                style={{
                  fontSize: 10,
                  padding: '0 5px',
                  height: 16,
                  display: 'flex',
                  alignItems: 'center',
                  borderRadius: 4,
                  background: isActive ? 'var(--accent-soft)' : 'var(--bg-elevated)',
                  color: isActive ? 'var(--accent)' : 'var(--text-tertiary)',
                  fontWeight: 600,
                }}
              >
                {tab.badge}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
