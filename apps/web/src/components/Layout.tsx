import type { ReactNode } from 'react';
import { NavLink } from 'react-router-dom';
import { Icon } from './Icons';
import { Button, Tooltip } from '../design-system';
import { usePlaygroundStore } from '../stores/playgroundStore';
import { useTheme } from '../hooks/useTheme';
import { StatusBar } from './StatusBar';
import { PipelineVisualizer } from '../features/playground/components/PipelineVisualizer';

interface LayoutProps {
  children: ReactNode;
  onCompile: () => void;
  onCancel: () => void;
  isCompiling: boolean;
}

const PLATFORMS = [
  { id: 'flutter' as const, label: 'Flutter', icon: Icon.Flutter, color: 'var(--flutter)' },
  { id: 'compose' as const, label: 'Compose', icon: Icon.Kotlin, color: 'var(--compose)' },
  { id: 'swiftui' as const, label: 'SwiftUI', icon: Icon.Swift, color: 'var(--swiftui)' },
];

export default function Layout({ children, onCompile, onCancel, isCompiling }: LayoutProps) {
  const { platform, setPlatform } = usePlaygroundStore();
  const { theme, toggleTheme } = useTheme();

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      height: '100vh',
      overflow: 'hidden',
      background: 'var(--bg-base)',
    }}>
      <nav style={{
        display: 'flex',
        alignItems: 'center',
        gap: 'var(--space-2)',
        padding: '0 var(--space-3)',
        height: 48,
        borderBottom: '1px solid var(--border-default)',
        background: 'var(--bg-surface)',
        flexShrink: 0,
        userSelect: 'none',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexShrink: 0 }}>
          <Icon.Logo size={20} />
          <span style={{ fontWeight: 700, fontSize: 14, letterSpacing: '-0.03em', color: 'var(--text-primary)' }}>
            motarjim
          </span>
          <span style={{
            fontSize: 10,
            color: 'var(--text-tertiary)',
            fontWeight: 500,
            padding: '1px 6px',
            background: 'var(--bg-elevated)',
            borderRadius: 4,
            border: '1px solid var(--border-subtle)',
          }}>
            Compiler
          </span>
        </div>

        <nav style={{ display: 'flex', gap: 2, flexShrink: 0 }}>
          {[
            { to: '/', label: 'Compiler', icon: Icon.Code },
            { to: '/playground', label: 'JS', icon: Icon.Play },
            { to: '/settings', label: 'Settings', icon: Icon.Settings },
          ].map((item) => {
            const ItemIcon = item.icon;
            return (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.to === '/'}
                style={({ isActive }) => ({
                  display: 'flex',
                  alignItems: 'center',
                  gap: 5,
                  border: 'none',
                  background: isActive ? 'var(--bg-active)' : 'transparent',
                  color: isActive ? 'var(--text-primary)' : 'var(--text-tertiary)',
                  fontSize: 11,
                  fontWeight: 600,
                  padding: '4px 8px',
                  borderRadius: 5,
                  cursor: 'pointer',
                  transition: 'all 120ms var(--ease-out)',
                  textTransform: 'uppercase',
                  letterSpacing: '0.03em',
                  fontFamily: 'inherit',
                  textDecoration: 'none',
                })}
              >
                <ItemIcon size={12} />
                {item.label}
              </NavLink>
            );
          })}
        </nav>

        <div style={{ width: 1, height: 20, background: 'var(--border-subtle)', flexShrink: 0, margin: '0 var(--space-2)' }} />

        <PipelineVisualizer />

        <div style={{ flex: 1 }} />

        <div style={{
          display: 'flex',
          gap: 2,
          background: 'var(--bg-elevated)',
          border: '1px solid var(--border-default)',
          borderRadius: 'var(--radius-sm)',
          padding: 2,
        }}>
          {PLATFORMS.map((p) => {
            const isActive = platform === p.id;
            const PlatformIcon = p.icon;
            return (
              <Tooltip key={p.id} content={`Generate ${p.label} code`}>
                <button
                  onClick={() => setPlatform(p.id)}
                  aria-label={`Switch to ${p.label}`}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 5,
                    border: 'none',
                    background: isActive ? 'var(--bg-active)' : 'transparent',
                    color: isActive ? p.color : 'var(--text-tertiary)',
                    fontSize: 11,
                    fontWeight: 600,
                    padding: '4px 8px',
                    borderRadius: 5,
                    cursor: 'pointer',
                    transition: 'all 120ms var(--ease-out)',
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
                  <PlatformIcon size={12} />
                  {p.label}
                </button>
              </Tooltip>
            );
          })}
        </div>

        <div style={{ width: 1, height: 20, background: 'var(--border-subtle)', flexShrink: 0, margin: '0 var(--space-1)' }} />

        <Button
          variant="primary"
          size="md"
          loading={isCompiling}
          icon={isCompiling ? undefined : <Icon.Play size={13} />}
          onClick={isCompiling ? onCancel : onCompile}
          aria-label={isCompiling ? 'Cancel compilation' : 'Compile code'}
        >
          {isCompiling ? 'Compiling...' : 'Compile'}
        </Button>

        <Tooltip content={`Switch to ${theme === 'light' ? 'dark' : 'light'} theme`}>
          <button
            onClick={toggleTheme}
            aria-label={`Switch to ${theme === 'light' ? 'dark' : 'light'} theme`}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              border: 'none',
              background: 'transparent',
              color: 'var(--text-tertiary)',
              width: 28,
              height: 28,
              borderRadius: 'var(--radius-sm)',
              cursor: 'pointer',
              transition: 'all 120ms var(--ease-out)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'var(--bg-hover)';
              e.currentTarget.style.color = 'var(--text-secondary)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'transparent';
              e.currentTarget.style.color = 'var(--text-tertiary)';
            }}
          >
            {theme === 'light' ? <Icon.Moon size={14} /> : <Icon.Sun size={14} />}
          </button>
        </Tooltip>
      </nav>

      <main style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
        position: 'relative',
      }}>
        {children}
      </main>

      <StatusBar />
    </div>
  );
}
