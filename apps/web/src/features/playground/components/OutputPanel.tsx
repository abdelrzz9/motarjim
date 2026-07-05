import { useCallback, useState } from 'react';
import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { Icon } from '../../../components/Icons';
import { Tooltip, Badge } from '../../../design-system';
import { useNotificationsStore } from '../../../stores/notificationsStore';
import { ErrorState } from './ErrorState';
import { EmptyState } from './EmptyState';
import { LoadingState } from './LoadingState';
import type { OutputTab } from '../../../services/types';

const OUTPUT_TABS = [
  { id: 'code' as const, label: 'Code' },
  { id: 'diagnostics' as const, label: 'Diagnostics' },
  { id: 'ast' as const, label: 'AST' },
];

const PLATFORM_TABS = [
  { id: 'flutter' as const, label: 'Flutter', icon: Icon.Flutter, color: 'var(--flutter)' },
  { id: 'compose' as const, label: 'Compose', icon: Icon.Kotlin, color: 'var(--compose)' },
  { id: 'swiftui' as const, label: 'SwiftUI', icon: Icon.Swift, color: 'var(--swiftui)' },
];

function highlightCode(code: string): React.ReactNode[] {
  const lines = code.split('\n');
  return lines.map((line, i) => {
    let remaining = line;
    const tokens: React.ReactNode[] = [];
    let key = 0;

    while (remaining.length > 0) {
      const commentMatch = remaining.match(/^(\/\/.*)/);
      const stringMatch = remaining.match(/^("[^"]*"|'[^']*')/);
      const annotMatch = remaining.match(/^(@\w+)/);
      const numMatch = remaining.match(/^(\b\d+\.?\d*\b)/);
      const keywordMatch = remaining.match(/^(\b(?:import|package|class|struct|enum|extension|func|var|let|const|return|if|else|for|in|while|switch|case|default|break|continue|throw|guard|defer|init|self|super|override|final|lazy|static|mutating|indirect|required|convenience|open|public|internal|fileprivate|private|Widget|StatefulWidget|StatelessWidget|State|build|child|children|padding|margin|color|fontSize|fontWeight|Text|Row|Column|Container|Scaffold|AppBar|Center|EdgeInsets|BoxDecoration|BorderRadius|ElevatedButton|TextField|ListView|Icon|MaterialApp|ThemeData|composable|preview|Modifier|fillMaxSize|padding|background|clickable|Text|Column|Row|Box|Spacer|Button|OutlinedButton|TextField|Card|LazyColumn|Scaffold|Center|padding|colors|MaterialTheme)\b)/);

      if (commentMatch) {
        tokens.push(<span key={key++} style={{ color: 'var(--text-tertiary)', fontStyle: 'italic' }}>{commentMatch[1]}</span>);
        remaining = '';
      } else if (stringMatch) {
        tokens.push(<span key={key++} style={{ color: '#c3e88d' }}>{stringMatch[1]}</span>);
        remaining = remaining.slice(stringMatch[1].length);
      } else if (annotMatch) {
        tokens.push(<span key={key++} style={{ color: '#c792ea' }}>{annotMatch[1]}</span>);
        remaining = remaining.slice(annotMatch[1].length);
      } else if (numMatch) {
        tokens.push(<span key={key++} style={{ color: '#f78c6c' }}>{numMatch[1]}</span>);
        remaining = remaining.slice(numMatch[1].length);
      } else if (keywordMatch) {
        tokens.push(<span key={key++} style={{ color: '#c792ea' }}>{keywordMatch[1]}</span>);
        remaining = remaining.slice(keywordMatch[1].length);
      } else {
        tokens.push(<span key={key++}>{remaining[0]}</span>);
        remaining = remaining.slice(1);
      }
    }

    return <div key={i}>{tokens.length > 0 ? tokens : <br />}</div>;
  });
}

export function OutputPanel() {
  const {
    output, diagnostics, isCompiling, platform, stats,
    outputTab, setOutputTab, setPlatform,
    setHtml, setCss, setJs, setActiveTab,
  } = usePlaygroundStore();
  const notify = useNotificationsStore((s) => s.notify);
  const [copied, setCopied] = useState(false);

  const errors = diagnostics.filter(d => d.severity === 'error');
  const warnings = diagnostics.filter(d => d.severity === 'warning');
  const infos = diagnostics.filter(d => d.severity === 'info' || d.severity === 'hint' || d.severity === 'note');

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(output);
      setCopied(true);
      notify('Copied to clipboard', 'success', 1500);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      notify('Failed to copy', 'error', 2000);
    }
  }, [output, notify]);

  const handleDownload = useCallback(() => {
    const ext = platform === 'flutter' ? 'dart' : platform === 'compose' ? 'kt' : 'swift';
    const blob = new Blob([output], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `output.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
    notify('File downloaded', 'success', 1500);
  }, [output, platform, notify]);

  const handleLoadSample = useCallback(() => {
    const SAMPLE_HTML = `<div class="container">\n  <div class="card">\n    <h2>Hello World</h2>\n    <p>This is a sample component.</p>\n    <button>Click Me</button>\n  </div>\n</div>`;
    const SAMPLE_CSS = `.container {\n  display: flex;\n  padding: 24px;\n  background: #f5f5f5;\n}\n.card {\n  background: white;\n  border-radius: 12px;\n  padding: 24px;\n  box-shadow: 0 2px 8px rgba(0,0,0,0.1);\n}`;
    const SAMPLE_JS = `// Card interactions\ndocument.querySelectorAll('.card button').forEach(btn => {\n  btn.addEventListener('click', () => {\n    console.log('Card clicked!');\n  });\n});`;
    setHtml(SAMPLE_HTML);
    setCss(SAMPLE_CSS);
    setJs(SAMPLE_JS);
    notify('Sample loaded! Compiling...', 'info', 2000);
    document.dispatchEvent(new CustomEvent('compile-trigger'));
  }, [setHtml, setCss, setJs, notify]);

  const handleUpload = useCallback(() => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.html,.htm,.css,.js,.mjs';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = (ev) => {
        const text = ev.target?.result as string;
        if (file.name.endsWith('.html') || file.name.endsWith('.htm')) {
          setHtml(text);
          setActiveTab('html');
        } else if (file.name.endsWith('.css')) {
          setCss(text);
          setActiveTab('css');
        } else {
          setJs(text);
          setActiveTab('js');
        }
        notify(`Loaded ${file.name}`, 'success', 1500);
      };
      reader.readAsText(file);
    };
    input.click();
  }, [setHtml, setCss, setJs, setActiveTab, notify]);

  const hasContent = isCompiling || !!output || errors.length > 0 || warnings.length > 0 || infos.length > 0;

  return (
    <div style={{
      flex: 1,
      display: 'flex',
      flexDirection: 'column',
      overflow: 'hidden',
      background: 'var(--bg-surface)',
    }}>
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: 'var(--space-2) var(--space-3)',
        borderBottom: '1px solid var(--border-subtle)',
        background: 'var(--bg-surface)',
        flexShrink: 0,
      }}>
        <div style={{
          display: 'flex',
          gap: 1,
          background: 'var(--bg-elevated)',
          borderRadius: 'var(--radius-sm)',
          padding: 2,
        }}>
          {PLATFORM_TABS.map((p) => {
            const isActive = platform === p.id;
            const PlatformIcon = p.icon;
            return (
              <button
                key={p.id}
                onClick={() => setPlatform(p.id)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 5,
                  border: 'none',
                  background: isActive ? 'var(--bg-active)' : 'transparent',
                  color: isActive ? p.color : 'var(--text-tertiary)',
                  fontSize: 11,
                  fontWeight: 600,
                  padding: '3px 8px',
                  borderRadius: 5,
                  cursor: 'pointer',
                  transition: 'all 120ms var(--ease-out)',
                  textTransform: 'uppercase',
                  letterSpacing: '0.03em',
                  fontFamily: 'inherit',
                }}
              >
                <PlatformIcon size={12} />
                {p.label}
              </button>
            );
          })}
        </div>

        <div style={{ display: 'flex', gap: 2 }}>
          {hasContent && (
            <>
              <Tooltip content={copied ? 'Copied!' : 'Copy to clipboard'}>
                <IconButton onClick={handleCopy} aria-label="Copy output" style={copied ? { color: 'var(--success)' } : undefined}>
                  {copied ? <Icon.Check size={13} /> : <Icon.Copy size={13} />}
                </IconButton>
              </Tooltip>
              <Tooltip content="Download file">
                <IconButton onClick={handleDownload} aria-label="Download">
                  <Icon.Download size={13} />
                </IconButton>
              </Tooltip>
              <Tooltip content="Format code">
                <IconButton onClick={() => {}} aria-label="Format">
                  <Icon.Format size={13} />
                </IconButton>
              </Tooltip>
              <Tooltip content="Open in fullscreen">
                <IconButton onClick={() => {}} aria-label="Fullscreen">
                  <Icon.Fullscreen size={13} />
                </IconButton>
              </Tooltip>
              <Tooltip content="Open in playground">
                <IconButton onClick={() => {}} aria-label="Open in playground">
                  <Icon.External size={13} />
                </IconButton>
              </Tooltip>
            </>
          )}
        </div>
      </div>

      {isCompiling ? (
        <LoadingState />
      ) : !hasContent ? (
        <EmptyState onLoadSample={handleLoadSample} onUpload={handleUpload} />
      ) : outputTab === 'diagnostics' && diagnostics.length > 0 ? (
        <div style={{ flex: 1, overflow: 'auto' }}>
          <ErrorState diagnostics={diagnostics} />
        </div>
      ) : outputTab === 'ast' ? (
        <div style={{
          flex: 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: 'var(--text-tertiary)',
          fontSize: 12,
          padding: 'var(--space-4)',
        }}>
          AST view available in development mode
        </div>
      ) : (
        <div style={{
          flex: 1,
          overflow: 'auto',
          position: 'relative',
        }}>
          <div style={{
            padding: 'var(--space-3) var(--space-4)',
            fontSize: 12,
            lineHeight: 1.6,
            fontFamily: 'var(--font-mono)',
            whiteSpace: 'pre-wrap',
            wordBreak: 'break-word',
            minHeight: '100%',
          }}>
            {output ? highlightCode(output) : (
              <span style={{ color: 'var(--text-tertiary)', opacity: 0.5 }}>
                No output generated
              </span>
            )}
          </div>
        </div>
      )}

      {stats && (
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          padding: 'var(--space-2) var(--space-3)',
          borderTop: '1px solid var(--border-subtle)',
          background: 'var(--bg-elevated)',
          flexShrink: 0,
          fontSize: 10,
          color: 'var(--text-tertiary)',
        }}>
          {stats.timeMs > 0 && (
            <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
              <Icon.Clock size={11} />
              {stats.timeMs}ms
            </span>
          )}
          <span>{stats.nodesParsed} nodes</span>
          <span>{stats.cssRules} CSS rules</span>
          <span>{stats.irNodes} IR nodes</span>
          {stats.parseTimeMs > 0 && <span>Parse: {stats.parseTimeMs}ms</span>}
          {stats.genTimeMs > 0 && <span>Gen: {stats.genTimeMs}ms</span>}
        </div>
      )}

      <div style={{
        display: 'flex',
        gap: 1,
        background: 'var(--bg-elevated)',
        borderRadius: 'var(--radius-sm)',
        padding: 2,
        margin: 'var(--space-2)',
        flexShrink: 0,
      }}>
        {OUTPUT_TABS.map((tab) => {
          const isActive = outputTab === tab.id;
          let badge: string | number | undefined;
          if (tab.id === 'diagnostics') {
            const total = errors.length + warnings.length;
            if (total > 0) badge = total;
          }
          return (
            <button
              key={tab.id}
              onClick={() => setOutputTab(tab.id as OutputTab)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 5,
                border: 'none',
                background: isActive ? 'var(--bg-active)' : 'transparent',
                color: isActive ? 'var(--text-primary)' : 'var(--text-tertiary)',
                fontSize: 11,
                fontWeight: 500,
                padding: '3px 8px',
                borderRadius: 5,
                cursor: 'pointer',
                transition: 'all 120ms var(--ease-out)',
                textTransform: 'uppercase',
                letterSpacing: '0.03em',
                fontFamily: 'inherit',
                flex: 1,
                justifyContent: 'center',
              }}
            >
              {tab.label}
              {badge !== undefined && (
                <Badge variant={errors.length > 0 ? 'error' : 'warning'} size="sm">
                  {badge}
                </Badge>
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
}

function IconButton({ children, onClick, 'aria-label': ariaLabel, style }: {
  children: React.ReactNode;
  onClick: () => void;
  'aria-label'?: string;
  style?: React.CSSProperties;
}) {
  return (
    <button
      onClick={onClick}
      aria-label={ariaLabel}
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: 26,
        height: 26,
        border: 'none',
        background: 'transparent',
        color: 'var(--text-tertiary)',
        borderRadius: 4,
        cursor: 'pointer',
        transition: 'all 120ms var(--ease-out)',
        ...style,
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
      {children}
    </button>
  );
}
