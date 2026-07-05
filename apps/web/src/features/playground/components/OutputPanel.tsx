import { useState, useCallback, useRef, useEffect } from 'react';
import Editor from '@monaco-editor/react';
import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { useTheme } from '../../../hooks/useTheme';
import { useNotificationsStore } from '../../../stores/notificationsStore';
import { Icon } from '../../../components/Icons';
import DiagnosticsPanel from './DiagnosticsPanel';
import AstViewer from './AstViewer';
import styles from './OutputPanel.module.css';

const PLATFORM_LANG: Record<string, string> = {
  flutter: 'dart',
  compose: 'kotlin',
  swiftui: 'swift',
};

export default function OutputPanel() {
  const { output, diagnostics, platform, isCompiling, pipelineStage, outputTab, setOutputTab } = usePlaygroundStore();
  const { theme } = useTheme();
  const notify = useNotificationsStore((s) => s.notify);
  const [copied, setCopied] = useState(false);
  const [showCompletion, setShowCompletion] = useState(false);
  const prevCompiling = useRef(false);

  useEffect(() => {
    if (prevCompiling.current && !isCompiling && output) {
      setShowCompletion(true);
      const timer = setTimeout(() => setShowCompletion(false), 1200);
      return () => clearTimeout(timer);
    }
    prevCompiling.current = isCompiling;
  }, [isCompiling, output]);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(output);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
      notify('Copied to clipboard', 'success');
    } catch {
      notify('Failed to copy', 'error');
    }
  }, [output, notify]);

  const handleDownload = useCallback(() => {
    const ext = PLATFORM_LANG[platform] || 'txt';
    const blob = new Blob([output], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `GeneratedView.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
    notify('File downloaded', 'success');
  }, [output, platform, notify]);

  const handleFormat = useCallback(() => {
    notify('Format not available for output', 'info');
  }, [notify]);

  const handleFullscreen = useCallback(() => {
    const el = document.querySelector(`.${styles.panel}`);
    if (el && !document.fullscreenElement) {
      el.requestFullscreen().catch(() => {});
    } else if (document.fullscreenElement) {
      document.exitFullscreen().catch(() => {});
    }
  }, []);

  const isEmpty = !output && !isCompiling;

  const getEditorLang = () => {
    if (outputTab !== 'code') return 'plaintext';
    return PLATFORM_LANG[platform] || 'plaintext';
  };

  const errorCount = diagnostics.filter((d) => d.severity === 'error').length;
  const warningCount = diagnostics.filter((d) => d.severity === 'warning').length;

  const STATUS_MESSAGES = [
    'Parsing HTML document…',
    'Building style tree…',
    'Analyzing semantics and structure…',
    'Constructing intermediate representation…',
    'Optimizing layout and widget tree…',
    'Generating native code…',
  ];

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <div className={styles.tabs}>
          <button
            className={`${styles.tab} ${outputTab === 'code' ? styles.tabActive : ''}`}
            onClick={() => setOutputTab('code')}
          >
            CODE
          </button>
          <button
            className={`${styles.tab} ${outputTab === 'diagnostics' ? styles.tabActive : ''}`}
            onClick={() => setOutputTab('diagnostics')}
          >
            DIAGNOSTICS
            {(errorCount > 0 || warningCount > 0) && (
              <span className={styles.badge}>
                {errorCount + warningCount}
              </span>
            )}
          </button>
          <button
            className={`${styles.tab} ${outputTab === 'ast' ? styles.tabActive : ''}`}
            onClick={() => setOutputTab('ast')}
          >
            AST
          </button>
        </div>
        <div className={styles.actions}>
          <button
            className={`${styles.actionBtn} ${copied ? styles.copySuccess : ''}`}
            onClick={handleCopy}
            title="Copy"
          >
            {copied ? <Icon.Check size={12} /> : <Icon.Copy size={12} />}
          </button>
          <button className={styles.actionBtn} onClick={handleDownload} title="Download">
            <Icon.Download size={12} />
          </button>
          <button className={styles.actionBtn} onClick={handleFormat} title="Format">
            <Icon.Format size={12} />
          </button>
          <span className={styles.separator} />
          <button className={styles.actionBtn} onClick={handleFullscreen} title="Fullscreen">
            <Icon.Fullscreen size={12} />
          </button>
        </div>
      </div>
      <div className={styles.content}>
        {isEmpty ? (
          <div className={styles.emptyState}>
            <div className={styles.emptyStateIcon}>
              <Icon.Play size={40} />
            </div>
            <h3 className={styles.emptyStateTitle}>Compile your code</h3>
            <p className={styles.emptyStateDesc}>
              Write HTML and CSS, then click Compile to generate native code.
            </p>
          </div>
        ) : (
          <>
            {outputTab === 'code' && (
              <Editor
                key={platform}
                defaultLanguage={getEditorLang()}
                language={getEditorLang()}
                value={output}
                theme={theme === 'dark' ? 'vs-dark' : 'vs'}
                options={{
                  readOnly: true,
                  minimap: { enabled: false },
                  fontSize: 13,
                  fontFamily: 'var(--font-mono)',
                  lineNumbers: 'on',
                  scrollBeyondLastLine: false,
                  automaticLayout: true,
                  wordWrap: 'on',
                  padding: { top: 8 },
                  glyphMargin: false,
                  folding: false,
                  lineDecorationsWidth: 6,
                  lineNumbersMinChars: 3,
                }}
              />
            )}
            {outputTab === 'diagnostics' && <DiagnosticsPanel />}
            {outputTab === 'ast' && <AstViewer />}
          </>
        )}

        <div className={`${styles.loadingOverlay} ${isCompiling ? styles.loadingOverlayVisible : ''}`}>
          <div className={styles.loadingContent}>
            <div className={styles.loadingMessage}>
              <div className={styles.spinner} />
              Compiling
            </div>
            <div className={styles.loadingProgress}>
              <div
                className={styles.loadingProgressBar}
                style={{ width: `${pipelineStage >= 0 ? ((pipelineStage + 1) / 6) * 100 : 0}%` }}
              />
            </div>
            <div className={styles.loadingStatus}>
              {pipelineStage >= 0 && pipelineStage < STATUS_MESSAGES.length
                ? STATUS_MESSAGES[pipelineStage]
                : 'Preparing...'}
            </div>
          </div>
        </div>

        <div className={`${styles.completionOverlay} ${showCompletion ? styles.completionOverlayVisible : ''}`}>
          <div className={styles.completionCheck}>
            <Icon.Check size={16} />
          </div>
        </div>
      </div>
    </div>
  );
}
