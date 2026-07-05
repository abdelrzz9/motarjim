import { useCallback, useRef } from 'react';
import Editor, { type OnMount } from '@monaco-editor/react';
import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { useTheme } from '../../../hooks/useTheme';
import { useNotificationsStore } from '../../../stores/notificationsStore';
import { Icon } from '../../../components/Icons';
import styles from './EditorPanel.module.css';

export default function EditorPanel() {
  const { html, css, activeTab, setHtml, setCss, setActiveTab } = usePlaygroundStore();
  const { theme } = useTheme();
  const notify = useNotificationsStore((s) => s.notify);
  const editorRef = useRef<Parameters<OnMount>[0] | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleMount: OnMount = (editor) => {
    editorRef.current = editor;
    editor.focus();
  };

  const handlePaste = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (activeTab === 'html') setHtml(text);
      else setCss(text);
      notify('Code pasted from clipboard', 'success');
    } catch {
      notify('Failed to read clipboard', 'error');
    }
  }, [activeTab, setHtml, setCss, notify]);

  const handleFormat = useCallback(() => {
    const editor = editorRef.current;
    if (editor) editor.getAction('editor.action.formatDocument')?.run();
  }, []);

  const handleClear = useCallback(() => {
    if (activeTab === 'html') setHtml('');
    else setCss('');
    notify('Cleared', 'info');
  }, [activeTab, setHtml, setCss, notify]);

  const handleUpload = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      const content = reader.result as string;
      if (file.name.endsWith('.css')) {
        setCss(content);
        setActiveTab('css');
      } else {
        setHtml(content);
        setActiveTab('html');
      }
      notify(`Loaded ${file.name}`, 'success');
    };
    reader.readAsText(file);
    e.target.value = '';
  }, [setHtml, setCss, setActiveTab, notify]);

  const handleLoadSample = useCallback(() => {
    setHtml(`<nav class="navbar">
  <h1>My App</h1>
</nav>
<section class="hero">
  <h1>Welcome</h1>
  <p>Build something great</p>
  <button>Get Started</button>
</section>`);
    setCss(`.navbar { background: #333; color: white; padding: 1rem; }
.hero { text-align: center; padding: 4rem; background: #1a1a2e; color: white; }
button { background: blue; color: white; border-radius: 8px; padding: 12px 24px; }`);
    notify('Sample loaded', 'success');
  }, [setHtml, setCss, notify]);

  const value = activeTab === 'html' ? html : css;
  const setValue = activeTab === 'html' ? setHtml : setCss;

  const isEmpty = !html && !css;

  if (isEmpty) {
    return (
      <div className={styles.panel}>
        <div className={styles.header}>
          <div className={styles.tabs}>
            <button
              className={`${styles.tab} ${activeTab === 'html' ? styles.tabActive : ''}`}
              onClick={() => setActiveTab('html')}
            >
              HTML
            </button>
            <button
              className={`${styles.tab} ${activeTab === 'css' ? styles.tabActive : ''}`}
              onClick={() => setActiveTab('css')}
            >
              CSS
            </button>
          </div>
        </div>
        <div className={styles.emptyState}>
          <div className={styles.emptyStateIcon}>
            <Icon.Code size={40} />
          </div>
          <h3 className={styles.emptyStateTitle}>HTML/CSS → Native UI</h3>
          <p className={styles.emptyStateDesc}>
            Paste your HTML and CSS or load a sample project.
          </p>
          <div className={styles.emptyStateActions}>
            <button className="btn btn-secondary btn-sm" onClick={handleLoadSample}>
              <Icon.Sample size={12} /> Load Sample
            </button>
            <button className="btn btn-secondary btn-sm" onClick={handleUpload}>
              <Icon.Upload size={12} /> Open File
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <div className={styles.tabs}>
          <button
            className={`${styles.tab} ${activeTab === 'html' ? styles.tabActive : ''}`}
            onClick={() => setActiveTab('html')}
          >
            HTML
          </button>
          <button
            className={`${styles.tab} ${activeTab === 'css' ? styles.tabActive : ''}`}
            onClick={() => setActiveTab('css')}
          >
            CSS
          </button>
        </div>
        <div className={styles.actions}>
          <button className={styles.actionBtn} onClick={handlePaste} title="Paste">
            <Icon.Paste size={12} />
          </button>
          <button className={styles.actionBtn} onClick={handleFormat} title="Format">
            <Icon.Format size={12} />
          </button>
          <button className={styles.actionBtn} onClick={handleUpload} title="Upload file">
            <Icon.Upload size={12} />
          </button>
          <button className={styles.actionBtn} onClick={handleClear} title="Clear">
            <Icon.Clear size={12} />
          </button>
          <input ref={fileInputRef} type="file" accept=".html,.htm,.css" className={styles.fileInput} onChange={handleFileChange} />
          <span className={styles.separator} />
          <button className={styles.sampleBtn} onClick={handleLoadSample} title="Load sample">
            <Icon.Sample size={12} /> Sample
          </button>
        </div>
      </div>
      <div className={styles.editor}>
        <Editor
          key={activeTab}
          defaultLanguage={activeTab}
          language={activeTab}
          value={value}
          onChange={(val) => setValue(val ?? '')}
          theme={theme === 'dark' ? 'vs-dark' : 'vs'}
          onMount={handleMount}
          options={{
            minimap: { enabled: false },
            fontSize: 13,
            fontFamily: 'var(--font-mono)',
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 2,
            wordWrap: 'on',
            padding: { top: 8 },
            glyphMargin: false,
            folding: false,
            lineDecorationsWidth: 6,
            lineNumbersMinChars: 3,
          }}
        />
      </div>
    </div>
  );
}
