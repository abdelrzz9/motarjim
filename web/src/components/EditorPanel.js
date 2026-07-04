import { icon } from '../utils/icons.js';
import { store } from '../state/store.js';
import { SAMPLE_HTML, SAMPLE_CSS } from '../constants.js';

export function createEditorPanel() {
  const el = document.createElement('div');
  el.className = 'split-panel left';
  el.setAttribute('aria-label', 'Input editor');

  let activeTab = 'html';

  const htmlContent = store.get('html') || '';
  const cssContent = store.get('css') || '';

  /* ----- Toolbar ----- */
  const toolbar = document.createElement('div');
  toolbar.className = 'panel-header';

  const tabs = document.createElement('div');
  tabs.className = 'panel-tabs';
  tabs.setAttribute('role', 'tablist');

  const htmlTab = document.createElement('button');
  htmlTab.className = 'panel-tab active';
  htmlTab.setAttribute('role', 'tab');
  htmlTab.setAttribute('aria-selected', 'true');
  htmlTab.textContent = 'HTML';

  const cssTab = document.createElement('button');
  cssTab.className = 'panel-tab';
  cssTab.setAttribute('role', 'tab');
  cssTab.setAttribute('aria-selected', 'false');
  cssTab.textContent = 'CSS';

  tabs.appendChild(htmlTab);
  tabs.appendChild(cssTab);

  const actions = document.createElement('div');
  actions.className = 'panel-actions';

  const actionBtns = [
    { id: 'paste', icon: 'paste', label: 'Paste', shortcut: '' },
    { id: 'format', icon: 'format', label: 'Format', shortcut: 'Ctrl+Shift+F' },
    { id: 'upload', icon: 'upload', label: 'Upload', shortcut: '' },
    { id: 'clear', icon: 'clear', label: 'Clear', shortcut: '' },
    { type: 'separator' },
    { id: 'sample', icon: 'sample', label: 'Sample', shortcut: '', primary: true },
  ];

  actionBtns.forEach(btn => {
    if (btn.type === 'separator') {
      const sep = document.createElement('div');
      sep.className = 'panel-action-btn separator';
      actions.appendChild(sep);
      return;
    }

    const b = document.createElement('button');
    b.className = `panel-action-btn${btn.primary ? ' btn-primary' : ''}`;
    b.dataset.action = btn.id;
    b.innerHTML = icon(btn.icon) + `<span>${btn.label}</span>`;
    b.setAttribute('aria-label', `${btn.label}${btn.shortcut ? ` (${btn.shortcut})` : ''}`);
    actions.appendChild(b);
  });

  toolbar.appendChild(tabs);
  toolbar.appendChild(actions);

  /* ----- Editor area ----- */
  const editorContainer = document.createElement('div');
  editorContainer.className = 'editor-container';

  const wrapper = document.createElement('div');
  wrapper.className = 'editor-wrapper';

  const gutter = document.createElement('div');
  gutter.className = 'editor-gutter';
  gutter.setAttribute('aria-hidden', 'true');

  const inputArea = document.createElement('div');
  inputArea.className = 'editor-input-area';

  const htmlTextarea = document.createElement('textarea');
  htmlTextarea.className = 'editor-textarea';
  htmlTextarea.id = 'html-input';
  htmlTextarea.spellcheck = false;
  htmlTextarea.placeholder = '<section class="hero">\n  <h1>Hello</h1>\n</section>';
  htmlTextarea.value = htmlContent;
  htmlTextarea.setAttribute('aria-label', 'HTML input editor');

  const cssTextarea = document.createElement('textarea');
  cssTextarea.className = 'editor-textarea';
  cssTextarea.id = 'css-input';
  cssTextarea.spellcheck = false;
  cssTextarea.placeholder = '.hero {\n  padding: 2rem;\n}';
  cssTextarea.value = cssContent;
  cssTextarea.style.display = 'none';
  cssTextarea.setAttribute('aria-label', 'CSS input editor');

  inputArea.appendChild(htmlTextarea);
  inputArea.appendChild(cssTextarea);
  wrapper.appendChild(gutter);
  wrapper.appendChild(inputArea);
  editorContainer.appendChild(wrapper);

  el.appendChild(toolbar);
  el.appendChild(editorContainer);

  /* ----- Empty state overlay ----- */
  const emptyState = document.createElement('div');
  emptyState.className = 'empty-state';
  emptyState.style.display = 'none';
  emptyState.setAttribute('aria-label', 'Empty input. Paste HTML and CSS to get started.');

  const ill = document.createElement('div');
  ill.className = 'empty-state-illustration';
  ill.innerHTML = `<svg viewBox="0 0 120 120" fill="none" stroke="currentColor" stroke-width="1">
    <rect x="15" y="25" width="90" height="70" rx="8" stroke-dasharray="4 4"/>
    <path d="M30 45h60M30 55h40M30 65h50M30 75h30" stroke-dasharray="2 2"/>
    <circle cx="60" cy="18" r="6" stroke-dasharray="2 2"/>
    <circle cx="30" cy="18" r="4" stroke-dasharray="2 2"/>
    <circle cx="90" cy="18" r="5" stroke-dasharray="2 2"/>
  </svg>`;

  const title = document.createElement('h2');
  title.textContent = 'HTML/CSS → Native UI';

  const subtitle = document.createElement('p');
  subtitle.textContent = 'Paste your HTML and CSS, or load a sample project to get started.';

  const actionsContainer = document.createElement('div');
  actionsContainer.className = 'empty-state-actions';

  const sampleBtn = document.createElement('button');
  sampleBtn.className = 'btn btn-primary';
  sampleBtn.innerHTML = icon('sample') + 'Load Sample';

  const uploadBtn = document.createElement('button');
  uploadBtn.className = 'btn btn-secondary';
  uploadBtn.innerHTML = icon('upload') + 'Open File';

  actionsContainer.appendChild(sampleBtn);
  actionsContainer.appendChild(uploadBtn);

  emptyState.appendChild(ill);
  emptyState.appendChild(title);
  emptyState.appendChild(subtitle);
  emptyState.appendChild(actionsContainer);

  el.appendChild(emptyState);

  /* ----- Line numbers ----- */
  function updateGutter() {
    const ta = activeTab === 'html' ? htmlTextarea : cssTextarea;
    const lines = ta.value.split('\n').length;
    gutter.textContent = Array.from({ length: lines }, (_, i) => i + 1).join('\n');
  }

  function syncScroll() {
    gutter.scrollTop = inputArea.scrollTop;
  }

  htmlTextarea.addEventListener('input', () => {
    updateGutter();
    store.set('html', htmlTextarea.value);
  });

  cssTextarea.addEventListener('input', () => {
    updateGutter();
    store.set('css', cssTextarea.value);
  });

  htmlTextarea.addEventListener('scroll', syncScroll);
  cssTextarea.addEventListener('scroll', syncScroll);

  /* ----- Tab key handling ----- */
  function handleTabKey(e) {
    if (e.key === 'Tab') {
      e.preventDefault();
      const ta = e.target;
      const start = ta.selectionStart;
      const end = ta.selectionEnd;
      ta.value = ta.value.substring(0, start) + '  ' + ta.value.substring(end);
      ta.selectionStart = ta.selectionEnd = start + 2;
      ta.dispatchEvent(new Event('input'));
    }
  }

  htmlTextarea.addEventListener('keydown', handleTabKey);
  cssTextarea.addEventListener('keydown', handleTabKey);

  /* ----- Tab switching ----- */
  function switchTab(tab) {
    activeTab = tab;
    htmlTab.classList.toggle('active', tab === 'html');
    cssTab.classList.toggle('active', tab === 'css');
    htmlTab.setAttribute('aria-selected', tab === 'html');
    cssTab.setAttribute('aria-selected', tab === 'css');
    htmlTextarea.style.display = tab === 'html' ? '' : 'none';
    cssTextarea.style.display = tab === 'css' ? '' : 'none';
    store.set('editorTab', tab);
    setTimeout(updateGutter, 0);
  }

  htmlTab.addEventListener('click', () => switchTab('html'));
  cssTab.addEventListener('click', () => switchTab('css'));

  /* ----- Action buttons ----- */
  actions.querySelectorAll('[data-action]').forEach(btn => {
    btn.addEventListener('click', () => {
      const action = btn.dataset.action;
      switch (action) {
        case 'paste':
          navigator.clipboard.readText().then(text => {
            const ta = activeTab === 'html' ? htmlTextarea : cssTextarea;
            ta.focus();
            const start = ta.selectionStart;
            const end = ta.selectionEnd;
            ta.value = ta.value.substring(0, start) + text + ta.value.substring(end);
            ta.dispatchEvent(new Event('input'));
          });
          break;
        case 'format':
          // Basic formatting - could integrate prettier
          break;
        case 'upload':
          uploadBtn.click();
          break;
        case 'clear':
          if (activeTab === 'html') {
            htmlTextarea.value = '';
            htmlTextarea.dispatchEvent(new Event('input'));
          } else {
            cssTextarea.value = '';
            cssTextarea.dispatchEvent(new Event('input'));
          }
          break;
        case 'sample':
          htmlTextarea.value = SAMPLE_HTML;
          cssTextarea.value = SAMPLE_CSS;
          htmlTextarea.dispatchEvent(new Event('input'));
          cssTextarea.dispatchEvent(new Event('input'));
          switchTab('html');
          break;
      }
    });
  });

  /* ----- Upload file ----- */
  const fileInput = document.createElement('input');
  fileInput.type = 'file';
  fileInput.accept = '.html,.css,.htm';
  fileInput.style.display = 'none';
  fileInput.setAttribute('aria-hidden', 'true');

  uploadBtn.addEventListener('click', () => fileInput.click());

  fileInput.addEventListener('change', () => {
    const file = fileInput.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (e) => {
      const content = e.target.result;
      if (file.name.endsWith('.css')) {
        cssTextarea.value = content;
        cssTextarea.dispatchEvent(new Event('input'));
        switchTab('css');
      } else {
        htmlTextarea.value = content;
        htmlTextarea.dispatchEvent(new Event('input'));
        switchTab('html');
      }
    };
    reader.readAsText(file);
    fileInput.value = '';
  });

  el.appendChild(fileInput);

  /* ----- Toggle empty state ----- */
  function checkEmpty() {
    const hasHtml = htmlTextarea.value.trim().length > 0;
    const hasCss = cssTextarea.value.trim().length > 0;
    const isEmpty = !hasHtml && !hasCss;

    editorContainer.style.display = isEmpty ? 'none' : '';
    emptyState.style.display = isEmpty ? '' : 'none';

    if (isEmpty) {
      toolbar.style.borderBottom = '1px solid var(--border-default)';
    }
  }

  store.on('html', checkEmpty);
  store.on('css', checkEmpty);

  updateGutter();
  checkEmpty();

  return {
    el,
    getHTML: () => htmlTextarea.value,
    getCSS: () => cssTextarea.value,
    setHTML: (v) => { htmlTextarea.value = v; htmlTextarea.dispatchEvent(new Event('input')); },
    setCSS: (v) => { cssTextarea.value = v; cssTextarea.dispatchEvent(new Event('input')); },
    focus: () => htmlTextarea.focus(),
  };
}
