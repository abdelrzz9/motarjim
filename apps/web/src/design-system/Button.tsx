import type { ButtonHTMLAttributes, ReactNode } from 'react';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
  icon?: ReactNode;
  children?: ReactNode;
}

const variantStyles: Record<string, React.CSSProperties> = {
  primary: {
    background: 'var(--accent)',
    color: '#fff',
    border: 'none',
  },
  secondary: {
    background: 'var(--bg-elevated)',
    color: 'var(--text-secondary)',
    border: '1px solid var(--border-default)',
  },
  ghost: {
    background: 'transparent',
    color: 'var(--text-tertiary)',
    border: 'none',
  },
  danger: {
    background: 'var(--error-soft)',
    color: 'var(--error)',
    border: '1px solid rgba(239,68,68,0.2)',
  },
};

const sizeStyles: Record<string, React.CSSProperties> = {
  sm: { height: 28, fontSize: 11, padding: '0 8px', gap: 4 },
  md: { height: 32, fontSize: 12, padding: '0 12px', gap: 6 },
  lg: { height: 36, fontSize: 13, padding: '0 16px', gap: 8 },
};

export function Button({
  variant = 'secondary',
  size = 'md',
  loading,
  icon,
  children,
  style,
  disabled,
  ...props
}: ButtonProps) {
  const isDisabled = disabled || loading;
  return (
    <button
      disabled={isDisabled}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        borderRadius: 'var(--radius-sm)',
        fontWeight: 500,
        whiteSpace: 'nowrap',
        flexShrink: 0,
        transition: 'all 150ms var(--ease-out)',
        cursor: isDisabled ? 'not-allowed' : 'pointer',
        opacity: isDisabled ? 0.5 : 1,
        fontFamily: 'inherit',
        ...variantStyles[variant],
        ...sizeStyles[size],
        ...style,
      }}
      onMouseEnter={(e) => {
        if (isDisabled) return;
        const el = e.currentTarget;
        if (variant === 'primary') {
          el.style.background = 'var(--accent-hover)';
        } else if (variant === 'secondary') {
          el.style.background = 'var(--bg-hover)';
          el.style.borderColor = 'var(--border-hover)';
          el.style.color = 'var(--text-secondary)';
        } else if (variant === 'ghost') {
          el.style.background = 'var(--bg-hover)';
          el.style.color = 'var(--text-secondary)';
        }
      }}
      onMouseLeave={(e) => {
        const el = e.currentTarget;
        const base = variantStyles[variant];
        el.style.background = base.background as string;
        el.style.borderColor = base.border as string || '';
        el.style.color = base.color as string;
      }}
      {...props}
    >
      {loading ? (
        <span
          style={{
            width: size === 'sm' ? 12 : 14,
            height: size === 'sm' ? 12 : 14,
            border: '2px solid currentColor',
            borderTopColor: 'transparent',
            borderRadius: '50%',
            animation: 'spin 0.6s linear infinite',
          }}
        />
      ) : icon ? (
        <span style={{ display: 'flex', width: size === 'sm' ? 14 : 16, height: size === 'sm' ? 14 : 16, flexShrink: 0 }}>
          {icon}
        </span>
      ) : null}
      {children && <span>{children}</span>}
    </button>
  );
}
