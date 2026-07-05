import { useRef, useCallback } from 'react';
import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { Icon } from '../../../components/Icons';
import { Tooltip } from '../../../design-system';
import { useNotificationsStore } from '../../../stores/notificationsStore';
import Editor, { type OnMount } from '@monaco-editor/react';
import { useTheme } from '../../../hooks/useTheme';

const SAMPLES: Record<string, { html: string; css: string; js: string }> = {
  'Card Grid': {
    html: `<div class="container">\n  <div class="card">\n    <h2>Card Title</h2>\n    <p>This is a sample card component with some content.</p>\n    <button>Learn More</button>\n  </div>\n  <div class="card">\n    <h2>Another Card</h2>\n    <p>More content here to demonstrate the layout.</p>\n    <button>Get Started</button>\n  </div>\n</div>`,
    css: `.container {\n  display: flex;\n  gap: 16px;\n  padding: 16px;\n}\n.card {\n  background: white;\n  border-radius: 12px;\n  padding: 24px;\n  box-shadow: 0 2px 8px rgba(0,0,0,0.1);\n}\nh2 { margin: 0 0 8px; color: #111; }\np { color: #666; margin: 0 0 16px; line-height: 1.5; }\nbutton {\n  background: #6c5ce7;\n  color: white;\n  border: none;\n  padding: 8px 16px;\n  border-radius: 6px;\n  cursor: pointer;\n}`,
    js: `// Card interactions\ndocument.querySelectorAll('.card button').forEach(btn => {\n  btn.addEventListener('click', () => {\n    alert('Card clicked!');\n  });\n});`,
  },
  'Navigation Bar': {
    html: `<nav class="navbar">\n  <div class="logo">Acme Corp</div>\n  <div class="links">\n    <a href="#">Home</a>\n    <a href="#">Products</a>\n    <a href="#">About</a>\n    <a href="#">Contact</a>\n  </div>\n  <button class="cta">Sign Up</button>\n</nav>`,
    css: `.navbar {\n  display: flex;\n  align-items: center;\n  padding: 12px 24px;\n  background: #1a1a2e;\n  border-radius: 12px;\n}\n.logo {\n  font-weight: bold;\n  font-size: 18px;\n  color: #fff;\n  margin-right: 32px;\n}\n.links { display: flex; gap: 16px; flex: 1; }\n.links a {\n  color: #8888aa;\n  text-decoration: none;\n  font-size: 14px;\n}\n.cta {\n  background: #6c5ce7;\n  color: white;\n  border: none;\n  padding: 8px 20px;\n  border-radius: 8px;\n  cursor: pointer;\n}`,
    js: `// Mobile menu toggle\nconst menuBtn = document.querySelector('.menu-toggle');\nconst navLinks = document.querySelector('.links');\nmenuBtn?.addEventListener('click', () => {\n  navLinks?.classList.toggle('open');\n});`,
  },
};

const LANG_MAP: Record<string, string> = {
  html: 'html',
  css: 'css',
  js: 'javascript',
};

function CodeEditor({
  value,
  onChange,
  language,
  onMount,
}: {
  value: string;
  onChange: (val: string) => void;
  language: 'html' | 'css' | 'js';
  placeholder?: string;
  onMount?: (language: string, editor: Parameters<OnMount>[0]) => void;
}) {
  const { theme } = useTheme();

  const handleMount: OnMount = (editor) => {
    onMount?.(language, editor);
  };

  return (
    <Editor
      key={language}
      defaultLanguage={LANG_MAP[language]}
      language={LANG_MAP[language]}
      value={value}
      onChange={(val) => onChange(val ?? '')}
      theme={theme === 'dark' ? 'vs-dark' : 'vs'}
      onMount={handleMount}
      options={{
        minimap: { enabled: false },
        fontSize: 13,
        fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", ui-monospace, monospace',
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        automaticLayout: true,
        tabSize: 2,
        wordWrap: 'off',
        padding: { top: 12, bottom: 12 },
        glyphMargin: false,
        folding: true,
        lineDecorationsWidth: 8,
        lineNumbersMinChars: 3,
        renderLineHighlight: 'line',
        cursorBlinking: 'smooth',
        cursorSmoothCaretAnimation: 'on',
        smoothScrolling: true,
        bracketPairColorization: { enabled: true },
        autoClosingBrackets: 'always',
        autoClosingQuotes: 'always',
        autoClosingDelete: 'always',
        autoIndent: 'full',
        formatOnPaste: true,
        formatOnType: true,
        suggestOnTriggerCharacters: true,
        acceptSuggestionOnEnter: 'on',
        quickSuggestions: true,
        parameterHints: { enabled: true },
        hover: { enabled: true },
        inlayHints: { enabled: 'on' },
        colorDecorators: true,
        selectionHighlight: true,
        occurrencesHighlight: 'singleFile',
        matchBrackets: 'always',
        linkedEditing: true,
      }}
    />
  );
}

export function EditorPanel() {
  const { html, css, js, activeTab, setHtml, setCss, setJs, setActiveTab } = usePlaygroundStore();
  const notify = useNotificationsStore((s) => s.notify);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const editorRefs = useRef<Record<string, Parameters<OnMount>[0] | null>>({});

  const handleEditorMount = useCallback((language: string, editor: Parameters<OnMount>[0]) => {
    editorRefs.current[language] = editor;
  }, []);

  const handleFormat = useCallback(() => {
    const editor = editorRefs.current[LANG_MAP[activeTab]];
    if (editor) {
      editor.getAction('editor.action.formatDocument')?.run();
    }
  }, [activeTab]);

  const handlePasteFromClipboard = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (activeTab === 'html') setHtml(text);
      else if (activeTab === 'css') setCss(text);
      else setJs(text);
      notify('Pasted from clipboard', 'success', 1500);
    } catch {
      notify('Failed to read clipboard', 'error', 2000);
    }
  }, [activeTab, setHtml, setCss, setJs, notify]);

  const handleUpload = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
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
      } else if (file.name.endsWith('.js') || file.name.endsWith('.mjs')) {
        setJs(text);
        setActiveTab('js');
      }
      notify(`Loaded ${file.name}`, 'success', 1500);
    };
    reader.readAsText(file);
    e.target.value = '';
  }, [setHtml, setCss, setJs, setActiveTab, notify]);

  const handleClear = useCallback(() => {
    if (activeTab === 'html') setHtml('');
    else if (activeTab === 'css') setCss('');
    else setJs('');
    notify('Editor cleared', 'info', 1000);
  }, [activeTab, setHtml, setCss, setJs, notify]);

  const handleLoadSample = useCallback(() => {
    const keys = Object.keys(SAMPLES);
    const sample = SAMPLES[keys[Math.floor(Math.random() * keys.length)]];
    setHtml(sample.html);
    setCss(sample.css);
    setJs(sample.js);
    notify('Sample loaded', 'success', 1500);
  }, [setHtml, setCss, setJs, notify]);

  const tabs = [
    { id: 'html' as const, label: 'HTML', icon: Icon.Html },
    { id: 'css' as const, label: 'CSS', icon: Icon.Css },
    { id: 'js' as const, label: 'JS', icon: Icon.Code },
  ];

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
          {tabs.map((tab) => {
            const isActive = activeTab === tab.id;
            const TabIcon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 5,
                  border: 'none',
                  background: isActive ? 'var(--bg-active)' : 'transparent',
                  color: isActive ? 'var(--text-primary)' : 'var(--text-tertiary)',
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
                <TabIcon size={12} />
                {tab.label}
              </button>
            );
          })}
        </div>

        <div style={{ display: 'flex', gap: 2 }}>
          <Tooltip content="Paste from clipboard">
            <IconButton onClick={handlePasteFromClipboard} aria-label="Paste">
              <Icon.Paste size={13} />
            </IconButton>
          </Tooltip>
          <Tooltip content="Format code">
            <IconButton onClick={handleFormat} aria-label="Format">
              <Icon.Format size={13} />
            </IconButton>
          </Tooltip>
          <Tooltip content="Upload file">
            <IconButton onClick={handleUpload} aria-label="Upload file">
              <Icon.Upload size={13} />
            </IconButton>
          </Tooltip>
          <input
            ref={fileInputRef}
            type="file"
            accept=".html,.htm,.css,.js,.mjs"
            style={{ display: 'none' }}
            onChange={handleFileChange}
          />
          <Tooltip content="Clear editor">
            <IconButton onClick={handleClear} aria-label="Clear">
              <Icon.Clear size={13} />
            </IconButton>
          </Tooltip>
          <Tooltip content="Load sample">
            <IconButton onClick={handleLoadSample} aria-label="Load sample">
              <Icon.Folder size={13} />
            </IconButton>
          </Tooltip>
        </div>
      </div>

      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        {activeTab === 'html' && (
          <CodeEditor
            value={html}
            onChange={setHtml}
            language="html"
            onMount={handleEditorMount}
          />
        )}
        {activeTab === 'css' && (
          <CodeEditor
            value={css}
            onChange={setCss}
            language="css"
            onMount={handleEditorMount}
          />
        )}
        {activeTab === 'js' && (
          <CodeEditor
            value={js}
            onChange={setJs}
            language="js"
            onMount={handleEditorMount}
          />
        )}
      </div>
    </div>
  );
}

function IconButton({ children, onClick, 'aria-label': ariaLabel }: {
  children: React.ReactNode;
  onClick: () => void;
  'aria-label'?: string;
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
