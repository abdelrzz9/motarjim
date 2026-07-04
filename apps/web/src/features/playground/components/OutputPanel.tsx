import Editor from '@monaco-editor/react';
import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { useTheme } from '../../../hooks/useTheme';
import DiagnosticsPanel from './DiagnosticsPanel';
import AstViewer from './AstViewer';
import styles from './OutputPanel.module.css';

export default function OutputPanel() {
  const { output, diagnostics, outputTab, setOutputTab } = usePlaygroundStore();
  const { theme } = useTheme();

  return (
    <div className={styles.panel}>
      <div className={styles.tabs}>
        <button
          className={`${styles.tab} ${outputTab === 'code' ? styles.active : ''}`}
          onClick={() => setOutputTab('code')}
        >
          Generated Code
        </button>
        <button
          className={`${styles.tab} ${outputTab === 'diagnostics' ? styles.active : ''}`}
          onClick={() => setOutputTab('diagnostics')}
        >
          Diagnostics {diagnostics.length > 0 && (
            <span className={styles.badge}>{diagnostics.length}</span>
          )}
        </button>
        <button
          className={`${styles.tab} ${outputTab === 'ast' ? styles.active : ''}`}
          onClick={() => setOutputTab('ast')}
        >
          AST
        </button>
      </div>
      <div className={styles.content}>
        {outputTab === 'code' && (
          <Editor
            defaultLanguage="dart"
            value={output || '// Compiled output will appear here'}
            theme={theme === 'dark' ? 'vs-dark' : 'vs'}
            options={{
              readOnly: true,
              minimap: { enabled: false },
              fontSize: 14,
              fontFamily: "var(--font-mono)",
              lineNumbers: 'on',
              scrollBeyondLastLine: false,
              automaticLayout: true,
              wordWrap: 'on',
            }}
          />
        )}
        {outputTab === 'diagnostics' && <DiagnosticsPanel />}
        {outputTab === 'ast' && <AstViewer />}
      </div>
    </div>
  );
}
