import Editor, { type OnMount } from '@monaco-editor/react';
import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { useTheme } from '../../../hooks/useTheme';
import styles from './EditorPanel.module.css';

export default function EditorPanel() {
  const { html, css, activeTab, setHtml, setCss, setActiveTab } = usePlaygroundStore();
  const { theme } = useTheme();

  const handleMount: OnMount = (editor) => {
    editor.focus();
  };

  return (
    <div className={styles.panel}>
      <div className={styles.tabs}>
        <button
          className={`${styles.tab} ${activeTab === 'html' ? styles.active : ''}`}
          onClick={() => setActiveTab('html')}
        >
          HTML
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'css' ? styles.active : ''}`}
          onClick={() => setActiveTab('css')}
        >
          CSS
        </button>
      </div>
      <div className={styles.editor}>
        {activeTab === 'html' && (
          <Editor
            defaultLanguage="html"
            value={html}
            onChange={(val) => setHtml(val ?? '')}
            theme={theme === 'dark' ? 'vs-dark' : 'vs'}
            onMount={handleMount}
            options={{
              minimap: { enabled: false },
              fontSize: 14,
              fontFamily: "var(--font-mono)",
              lineNumbers: 'on',
              scrollBeyondLastLine: false,
              automaticLayout: true,
              tabSize: 2,
              wordWrap: 'on',
            }}
          />
        )}
        {activeTab === 'css' && (
          <Editor
            defaultLanguage="css"
            value={css}
            onChange={(val) => setCss(val ?? '')}
            theme={theme === 'dark' ? 'vs-dark' : 'vs'}
            options={{
              minimap: { enabled: false },
              fontSize: 14,
              fontFamily: "var(--font-mono)",
              lineNumbers: 'on',
              scrollBeyondLastLine: false,
              automaticLayout: true,
              tabSize: 2,
              wordWrap: 'on',
            }}
          />
        )}
      </div>
    </div>
  );
}
