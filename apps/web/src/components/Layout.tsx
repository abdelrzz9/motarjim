import type { ReactNode } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { usePlaygroundStore } from '../stores/playgroundStore';
import { useTheme } from '../hooks/useTheme';
import { Icon } from './Icons';
import styles from './Layout.module.css';

interface LayoutProps {
  children: ReactNode;
}

const PLATFORMS = [
  { id: 'flutter' as const, label: 'Flutter', color: 'var(--flutter)' },
  { id: 'compose' as const, label: 'Compose', color: 'var(--compose)' },
  { id: 'swiftui' as const, label: 'SwiftUI', color: 'var(--swiftui)' },
];

export default function Layout({ children }: LayoutProps) {
  const { pathname } = useLocation();
  const { platform, setPlatform } = usePlaygroundStore();
  const { theme, toggleTheme } = useTheme();

  return (
    <div className={styles.layout}>
      <nav className={styles.topnav}>
        <Link to="/" className={styles.brand}>
          <span className={styles.brandIcon}>
            <Icon.Logo size={16} />
          </span>
          <span className={styles.brandName}>motarjim</span>
          <span className={styles.brandSuffix}>Compiler</span>
        </Link>

        <div className={styles.divider} />

        <div className={styles.platforms}>
          {PLATFORMS.map((p) => (
            <button
              key={p.id}
              className={`${styles.platformBtn} ${platform === p.id ? styles.platformBtnActive : ''}`}
              onClick={() => setPlatform(p.id)}
              data-target={p.id}
            >
              <span className={styles.dot} style={{ background: p.color }} />
              {p.label}
            </button>
          ))}
        </div>

        <div className={styles.spacer} />

        <div className={styles.nav}>
          <Link
            to="/"
            className={`${styles.navLink} ${pathname === '/' ? styles.navLinkActive : ''}`}
          >
            <Icon.Playground size={13} />
            Playground
          </Link>
          <Link
            to="/playground"
            className={`${styles.navLink} ${pathname === '/playground' ? styles.navLinkActive : ''}`}
          >
            <Icon.Playground size={13} />
            JS
          </Link>
          <Link
            to="/settings"
            className={`${styles.navLink} ${pathname === '/settings' ? styles.navLinkActive : ''}`}
          >
            <Icon.Settings size={13} />
            Settings
          </Link>
        </div>

        <button
          className={styles.themeBtn}
          onClick={toggleTheme}
          aria-label={`Switch to ${theme === 'light' ? 'dark' : 'light'} mode`}
        >
          <Icon.Theme size={14} />
        </button>
      </nav>

      <main className={styles.main}>
        {children}
      </main>
    </div>
  );
}
