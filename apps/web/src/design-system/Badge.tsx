import type { ReactNode } from 'react';

interface BadgeProps {
  variant?: 'default' | 'success' | 'warning' | 'error' | 'info' | 'accent';
  size?: 'sm' | 'md';
  children: ReactNode;
}

const variantStyles: Record<string, { bg: string; color: string }> = {
  default: { bg: 'var(--bg-elevated)', color: 'var(--text-tertiary)' },
  success: { bg: 'var(--success-soft)', color: 'var(--success)' },
  warning: { bg: 'var(--warning-soft)', color: 'var(--warning)' },
  error: { bg: 'var(--error-soft)', color: 'var(--error)' },
  info: { bg: 'var(--info-soft)', color: 'var(--info)' },
  accent: { bg: 'var(--accent-soft)', color: 'var(--accent)' },
};

export function Badge({ variant = 'default', size = 'sm', children }: BadgeProps) {
  const s = variantStyles[variant];
  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 4,
        padding: size === 'sm' ? '2px 6px' : '3px 8px',
        borderRadius: 'var(--radius-sm)',
        fontSize: size === 'sm' ? 10 : 11,
        fontWeight: 600,
        background: s.bg,
        color: s.color,
        whiteSpace: 'nowrap',
        lineHeight: 1.3,
      }}
    >
      {children}
    </span>
  );
}

export function StatusDot({ variant = 'default' }: { variant?: BadgeProps['variant'] }) {
  const s = variantStyles[variant];
  return (
    <span
      style={{
        width: 6,
        height: 6,
        borderRadius: '50%',
        background: s.color,
        display: 'inline-block',
        flexShrink: 0,
      }}
    />
  );
}
