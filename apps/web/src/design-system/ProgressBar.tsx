interface ProgressBarProps {
  value?: number;
  indeterminate?: boolean;
  height?: number;
  color?: string;
}

export function ProgressBar({ value, indeterminate, height = 2, color = 'var(--accent)' }: ProgressBarProps) {
  return (
    <div
      style={{
        width: '100%',
        height,
        background: 'var(--bg-elevated)',
        borderRadius: height,
        overflow: 'hidden',
        flexShrink: 0,
      }}
    >
      <div
        style={{
          height: '100%',
          width: indeterminate ? '40%' : `${Math.min(100, Math.max(0, value ?? 0))}%`,
          background: color,
          borderRadius: height,
          transition: 'width 300ms var(--ease-out)',
          animation: indeterminate ? 'progress-indeterminate 1.5s ease-in-out infinite' : undefined,
        }}
      />
    </div>
  );
}
