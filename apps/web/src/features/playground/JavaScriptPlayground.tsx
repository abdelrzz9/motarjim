import { useCallback, useRef } from 'react';
import Editor, { type OnMount } from '@monaco-editor/react';
import { useJsPlaygroundStore } from '../../stores/jsPlaygroundStore';
import { useTheme } from '../../hooks/useTheme';
import { useNotificationsStore } from '../../stores/notificationsStore';
import { Icon } from '../../components/Icons';
import styles from './JavaScriptPlayground.module.css';

export default function JavaScriptPlayground() {
  const { code, setCode, clear } = useJsPlaygroundStore();
  const { theme } = useTheme();
  const notify = useNotificationsStore((s) => s.notify);
  const editorRef = useRef<Parameters<OnMount>[0] | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleMount: OnMount = (editor) => {
    editorRef.current = editor;
    editor.focus();
  };

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code);
      notify('Copied to clipboard', 'success');
    } catch {
      notify('Failed to copy', 'error');
    }
  }, [code, notify]);

  const handlePaste = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      setCode(text);
      notify('Code pasted from clipboard', 'success');
    } catch {
      notify('Failed to read clipboard', 'error');
    }
  }, [setCode, notify]);

  const handleFormat = useCallback(() => {
    const editor = editorRef.current;
    if (editor) {
      editor.getAction('editor.action.formatDocument')?.run();
    }
  }, []);

  const handleDownload = useCallback(() => {
    const blob = new Blob([code], { type: 'text/javascript' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'script.js';
    a.click();
    URL.revokeObjectURL(url);
    notify('File downloaded', 'success');
  }, [code, notify]);

  const handleUpload = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      setCode(reader.result as string);
      notify(`Loaded ${file.name}`, 'success');
    };
    reader.readAsText(file);
    e.target.value = '';
  }, [setCode, notify]);

  const handleClear = useCallback(() => {
    clear();
    notify('Editor cleared', 'info');
  }, [clear, notify]);

  return (
    <div className={styles.page}>
      <div className={styles.toolbar}>
        <div className={styles.actions}>
          <button className={styles.actionBtn} onClick={handleCopy} title="Copy">
            <Icon.Copy size={12} />
          </button>
          <button className={styles.actionBtn} onClick={handlePaste} title="Paste">
            <Icon.Paste size={12} />
          </button>
          <button className={styles.actionBtn} onClick={handleFormat} title="Format">
            <Icon.Format size={12} />
          </button>
          <button className={styles.actionBtn} onClick={handleDownload} title="Download">
            <Icon.Download size={12} />
          </button>
          <button className={styles.actionBtn} onClick={handleUpload} title="Upload .js file">
            <Icon.Upload size={12} />
          </button>
          <input ref={fileInputRef} type="file" accept=".js,.mjs" style={{ display: 'none' }} onChange={handleFileChange} />
          <span className={styles.separator} />
          <button className={styles.actionBtn} onClick={handleClear} title="Clear">
            <Icon.Clear size={12} />
          </button>
        </div>
      </div>
      <div className={styles.editor}>
        <Editor
          defaultLanguage="javascript"
          language="javascript"
          value={code}
          onChange={(val) => setCode(val ?? '')}
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
            folding: true,
            lineDecorationsWidth: 6,
            lineNumbersMinChars: 3,
          }}
        />
      </div>
    </div>
  );
}
