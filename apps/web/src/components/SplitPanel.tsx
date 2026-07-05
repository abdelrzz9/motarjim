import { useState, useRef, useCallback, type ReactNode } from 'react';

interface SplitPanelProps {
  left: ReactNode;
  right: ReactNode;
  defaultRatio?: number;
  minLeftWidth?: number;
  minRightWidth?: number;
}

export function SplitPanel({
  left,
  right,
  defaultRatio = 0.5,
  minLeftWidth = 300,
  minRightWidth = 300,
}: SplitPanelProps) {
  const [ratio, setRatio] = useState(defaultRatio);
  const containerRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    dragging.current = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';

    const handleMouseMove = (e: MouseEvent) => {
      if (!dragging.current || !containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const newRatio = (e.clientX - rect.left) / rect.width;
      const clampedRatio = Math.max(
        minLeftWidth / rect.width,
        Math.min(1 - minRightWidth / rect.width, newRatio),
      );
      setRatio(clampedRatio);
    };

    const handleMouseUp = () => {
      dragging.current = false;
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }, [minLeftWidth, minRightWidth]);

  return (
    <div
      ref={containerRef}
      style={{
        display: 'flex',
        flex: 1,
        overflow: 'hidden',
        position: 'relative',
      }}
    >
      <div style={{
        width: `${ratio * 100}%`,
        minWidth: 0,
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
      }}>
        {left}
      </div>

      <div
        onMouseDown={handleMouseDown}
        style={{
          width: 4,
          cursor: 'col-resize',
          flexShrink: 0,
          position: 'relative',
          zIndex: 10,
          transition: 'background 120ms var(--ease-out)',
        }}
        onMouseEnter={(e) => { e.currentTarget.style.background = 'var(--accent-soft)'; }}
        onMouseLeave={(e) => { e.currentTarget.style.background = 'transparent'; }}
      >
        <div style={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          color: 'var(--text-tertiary)',
          opacity: 0.5,
          pointerEvents: 'none',
        }}>
          <svg width="4" height="24" viewBox="0 0 4 24" fill="none">
            <circle cx="2" cy="4" r="1" fill="currentColor" opacity="0.5" />
            <circle cx="2" cy="12" r="1" fill="currentColor" opacity="0.5" />
            <circle cx="2" cy="20" r="1" fill="currentColor" opacity="0.5" />
          </svg>
        </div>
      </div>

      <div style={{
        flex: 1,
        minWidth: 0,
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
      }}>
        {right}
      </div>
    </div>
  );
}
