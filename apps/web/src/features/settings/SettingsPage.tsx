import { useTheme } from '../../hooks/useTheme';
import KeyboardShortcuts from '../../components/KeyboardShortcuts';

export default function SettingsPage() {
  const { theme, toggleTheme } = useTheme();

  return (
    <div style={{ padding: '2rem', maxWidth: '40rem', margin: '0 auto' }}>
      <h1 style={{ marginBottom: '2rem', fontSize: '1.5rem' }}>Settings</h1>

      <section style={{ marginBottom: '2rem' }}>
        <h2 style={{ marginBottom: '1rem', fontSize: '1.1rem' }}>Appearance</h2>
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <span style={{ color: 'var(--text-secondary)' }}>
            Theme: {theme === 'light' ? 'Light' : 'Dark'}
          </span>
          <button
            onClick={toggleTheme}
            style={{
              padding: '0.375rem 1rem',
              border: '1px solid var(--border)',
              borderRadius: 'var(--radius)',
              background: 'var(--bg-secondary)',
              color: 'var(--text-primary)',
              cursor: 'pointer',
            }}
          >
            Toggle to {theme === 'light' ? 'Dark' : 'Light'}
          </button>
        </div>
      </section>

      <section>
        <KeyboardShortcuts />
      </section>
    </div>
  );
}
