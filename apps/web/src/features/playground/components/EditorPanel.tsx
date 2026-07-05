import { useRef, useCallback, useEffect } from 'react';
import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { Icon } from '../../../components/Icons';
import { Tooltip } from '../../../design-system';
import { useNotificationsStore } from '../../../stores/notificationsStore';

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

function hsv(tag: string, color: string, key: number) {
  return <span key={key} style={{ color }}>{tag}</span>;
}

function highlightHTML(code: string): React.ReactNode[] {
  const lines = code.split('\n');
  return lines.map((line, i) => {
    const tokens: React.ReactNode[] = [];
    let remaining = line;
    let key = 0;
    while (remaining.length > 0) {
      const tag = remaining.match(/^(<\/?[\w-]+)/);
      const attr = remaining.match(/^(\s+[\w-]+=)/);
      const str = remaining.match(/^("[^"]*"|'[^']*')/);
      const comment = remaining.match(/^(<!--[\s\S]*?-->)/);

      if (comment) {
        tokens.push(hsv(comment[1], 'var(--text-tertiary)', key++));
        remaining = remaining.slice(comment[1].length);
      } else if (tag) {
        tokens.push(hsv(tag[1], '#c792ea', key++));
        remaining = remaining.slice(tag[1].length);
      } else if (attr) {
        tokens.push(hsv(attr[1], '#82aaff', key++));
        remaining = remaining.slice(attr[1].length);
      } else if (str) {
        tokens.push(hsv(str[1], '#c3e88d', key++));
        remaining = remaining.slice(str[1].length);
      } else {
        tokens.push(hsv(remaining[0], 'var(--text-primary)', key++));
        remaining = remaining.slice(1);
      }
    }
    return <div key={i}>{tokens}</div>;
  });
}

function highlightCSS(code: string): React.ReactNode[] {
  const lines = code.split('\n');
  return lines.map((line, i) => {
    const tokens: React.ReactNode[] = [];
    let remaining = line;
    let key = 0;
    while (remaining.length > 0) {
      const prop = remaining.match(/^([\w-]+)(?=\s*:)/);
      const val = remaining.match(/^(:[\s\S]*?;)/);
      const sel = remaining.match(/^([.#]?[\w-]+(?=\s*\{))/);
      const comment = remaining.match(/^(\/\*[\s\S]*?\*\/)/);

      if (comment) {
        tokens.push(hsv(comment[1], 'var(--text-tertiary)', key++));
        remaining = remaining.slice(comment[1].length);
      } else if (prop) {
        tokens.push(hsv(prop[1], '#82aaff', key++));
        remaining = remaining.slice(prop[1].length);
      } else if (val) {
        const v = val[1];
        tokens.push(hsv(v[0], 'var(--text-tertiary)', key++));
        const c = v.slice(1).match(/(#[0-9a-fA-F]+|rgba?\([^)]+\))/);
        if (c) {
          const idx = v.slice(1).indexOf(c[1]);
          tokens.push(hsv(v.slice(1, idx + 1), 'var(--text-primary)', key++));
          tokens.push(hsv(c[1], '#c3e88d', key++));
          tokens.push(hsv(v.slice(1).slice(idx + c[1].length), 'var(--text-primary)', key++));
        } else {
          tokens.push(hsv(v.slice(1), '#f78c6c', key++));
        }
      } else if (sel) {
        tokens.push(hsv(sel[1], '#c792ea', key++));
        remaining = remaining.slice(sel[1].length);
      } else {
        tokens.push(hsv(remaining[0], 'var(--text-primary)', key++));
        remaining = remaining.slice(1);
      }
    }
    return <div key={i}>{tokens}</div>;
  });
}

const JS_KEYWORDS = new Set([
  'async', 'await', 'break', 'case', 'catch', 'class', 'const', 'continue',
  'debugger', 'default', 'delete', 'do', 'else', 'export', 'extends', 'finally',
  'for', 'function', 'if', 'import', 'in', 'instanceof', 'let', 'new', 'of',
  'return', 'static', 'super', 'switch', 'this', 'throw', 'try', 'typeof',
  'var', 'void', 'while', 'with', 'yield', 'from', 'as', 'enum', 'implements',
  'interface', 'package', 'private', 'protected', 'public',
]);

function highlightJS(code: string): React.ReactNode[] {
  const lines = code.split('\n');
  return lines.map((line, i) => {
    const tokens: React.ReactNode[] = [];
    let remaining = line;
    let key = 0;
    while (remaining.length > 0) {
      const comment = remaining.match(/^(\/\/.*)/);
      const multiComment = remaining.match(/^(\/\*[\s\S]*?\*\/)/);
      const str = remaining.match(/^("[^"]*"|'[^']*'|`[^`]*`)/);
      const num = remaining.match(/^(\b\d+\.?\d*\b)/);
      const word = remaining.match(/^([$\w]+)/);

      if (comment) {
        tokens.push(hsv(comment[1], 'var(--text-tertiary)', key++));
        remaining = remaining.slice(comment[1].length);
      } else if (multiComment) {
        tokens.push(hsv(multiComment[1], 'var(--text-tertiary)', key++));
        remaining = remaining.slice(multiComment[1].length);
      } else if (str) {
        tokens.push(hsv(str[1], '#c3e88d', key++));
        remaining = remaining.slice(str[1].length);
      } else if (num) {
        tokens.push(hsv(num[1], '#f78c6c', key++));
        remaining = remaining.slice(num[1].length);
      } else if (word) {
        const w = word[1];
        if (JS_KEYWORDS.has(w)) {
          tokens.push(hsv(w, '#c792ea', key++));
        } else if (w[0] === w[0]?.toUpperCase() && w[0] !== w[0]?.toLowerCase()) {
          tokens.push(hsv(w, '#82aaff', key++));
        } else {
          tokens.push(hsv(w, 'var(--text-primary)', key++));
        }
        remaining = remaining.slice(w.length);
      } else {
        tokens.push(hsv(remaining[0], 'var(--text-primary)', key++));
        remaining = remaining.slice(1);
      }
    }
    return <div key={i}>{tokens}</div>;
  });
}

function CodeEditor({
  value,
  onChange,
  language,
  placeholder,
}: {
  value: string;
  onChange: (val: string) => void;
  language: 'html' | 'css' | 'js';
  placeholder?: string;
}) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const syncRef = useRef<HTMLDivElement>(null);
  const lineCount = value.split('\n').length;

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [value]);

  const handleScroll = useCallback(() => {
    if (syncRef.current && textareaRef.current) {
      syncRef.current.scrollTop = textareaRef.current.scrollTop;
    }
  }, []);

  const getLang = () => {
    switch (language) {
      case 'html': return 'HTML';
      case 'css': return 'CSS';
      case 'js': return 'JavaScript';
    }
  };

  const highlight = () => {
    if (!value) return null;
    switch (language) {
      case 'html': return highlightHTML(value);
      case 'css': return highlightCSS(value);
      case 'js': return highlightJS(value);
    }
  };

  return (
    <div style={{
      position: 'relative',
      flex: 1,
      overflow: 'hidden',
      background: 'var(--bg-base)',
    }}>
      <div style={{ display: 'flex', height: '100%', position: 'relative' }}>
        <div style={{
          width: 44,
          flexShrink: 0,
          padding: '12px 0',
          textAlign: 'right',
          color: 'var(--text-tertiary)',
          fontSize: 11,
          lineHeight: 'var(--editor-line-height)',
          fontFamily: 'var(--font-mono)',
          userSelect: 'none',
          borderRight: '1px solid var(--border-subtle)',
          background: 'var(--bg-base)',
          paddingRight: 10,
          opacity: 0.4,
          overflow: 'hidden',
        }}>
          {Array.from({ length: Math.max(1, lineCount) }).map((_, i) => (
            <div key={i}>{i + 1}</div>
          ))}
        </div>

        <div style={{ flex: 1, position: 'relative', overflow: 'hidden' }}>
          <div
            ref={syncRef}
            style={{
              padding: '12px 16px',
              fontSize: 12,
              lineHeight: 'var(--editor-line-height)',
              fontFamily: 'var(--font-mono)',
              color: 'var(--text-primary)',
              pointerEvents: 'none',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              minHeight: '100%',
              overflow: 'auto',
              position: 'absolute',
              inset: 0,
            }}>
            {highlight()}
            {!value && (
              <span style={{ color: 'var(--text-tertiary)', opacity: 0.4 }}>
                {placeholder || `Enter ${getLang()} code...`}
              </span>
            )}
          </div>
          <textarea
            ref={textareaRef}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onScroll={handleScroll}
            spellCheck={false}
            aria-label={`${getLang()} editor`}
            style={{
              position: 'absolute',
              inset: 0,
              width: '100%',
              height: '100%',
              border: 'none',
              background: 'transparent',
              color: 'transparent',
              caretColor: 'var(--accent)',
              resize: 'none',
              outline: 'none',
              padding: '12px 16px',
              fontSize: 12,
              lineHeight: 'var(--editor-line-height)',
              fontFamily: 'var(--font-mono)',
              overflow: 'auto',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
            }}
          />
        </div>
      </div>
    </div>
  );
}

export function EditorPanel() {
  const { html, css, js, activeTab, setHtml, setCss, setJs, setActiveTab } = usePlaygroundStore();
  const notify = useNotificationsStore((s) => s.notify);
  const fileInputRef = useRef<HTMLInputElement>(null);

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
            <IconButton onClick={() => {}} aria-label="Format">
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
            placeholder={'<div>\n  Enter your HTML here...\n</div>'}
          />
        )}
        {activeTab === 'css' && (
          <CodeEditor
            value={css}
            onChange={setCss}
            language="css"
            placeholder={'.container {\n  /* Enter your CSS here */\n}'}
          />
        )}
        {activeTab === 'js' && (
          <CodeEditor
            value={js}
            onChange={setJs}
            language="js"
            placeholder={'// Enter your JavaScript here\nconsole.log("hello");'}
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
