import type { ReactNode } from 'react';
import { Link } from 'react-router-dom';
import ThemeToggle from './ThemeToggle';
import styles from './Layout.module.css';

interface LayoutProps {
  children: ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  return (
    <div className={styles.layout}>
      <header className={styles.header}>
        <div className={styles.logo}>
          <Link to="/">Motarjim</Link>
        </div>
        <nav className={styles.nav}>
          <Link to="/">Playground</Link>
          <Link to="/settings">Settings</Link>
        </nav>
        <ThemeToggle />
      </header>
      <main className={styles.main}>
        {children}
      </main>
    </div>
  );
}
