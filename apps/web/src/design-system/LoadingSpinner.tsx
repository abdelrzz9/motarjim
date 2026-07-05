interface LoadingSpinnerProps {
  size?: number;
  color?: string;
}

export function LoadingSpinner({ size = 16, color = 'var(--accent)' }: LoadingSpinnerProps) {
  return (
    <span
      style={{
        width: size,
        height: size,
        border: '2px solid var(--border-default)',
        borderTopColor: color,
        borderRadius: '50%',
        display: 'inline-block',
        animation: 'spin 0.6s linear infinite',
      }}
    />
  );
}

export function Skeleton({ width = '100%', height = 12, radius = 4 }: { width?: string | number; height?: number; radius?: number }) {
  return (
    <div
      style={{
        width,
        height,
        borderRadius: radius,
        background: 'linear-gradient(90deg, var(--bg-elevated) 0%, var(--bg-hover) 50%, var(--bg-elevated) 100%)',
        backgroundSize: '200% 100%',
        animation: 'shimmer 1.5s ease-in-out infinite',
      }}
    />
  );
}

export function CodeSkeleton({ lines = 6 }: { lines?: number }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10, padding: 'var(--space-4)' }}>
      {Array.from({ length: lines }).map((_, i) => (
        <div key={i} style={{ display: 'flex', gap: 12, alignItems: 'center' }}>
          <Skeleton width={24} height={10} radius={2} />
          <Skeleton width={`${60 + Math.random() * 35}%`} height={10} radius={2} />
        </div>
      ))}
    </div>
  );
}
