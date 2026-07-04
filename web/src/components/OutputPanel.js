import { icon } from '../utils/icons.js';
import { store } from '../state/store.js';
import { TARGETS } from '../constants.js';
import { notify } from './Notifications.js';

export function createOutputPanel() {
  const el = document.createElement('div');
  el.className = 'split-panel right';
  el.setAttribute('aria-label', 'Generated code output');

  /* ----- Toolbar ----- */
  const toolbar = document.createElement('div');
  toolbar.className = 'panel-header';

  const tabContainer = document.createElement('div');
  tabContainer.className = 'panel-tabs';
  tabContainer.setAttribute('role', 'tablist');

  const tabEls = {};
  let activeTab = store.get('target') || 'flutter';

  TARGETS.forEach(t => {
    const btn = document.createElement('button');
    btn.className = `panel-tab${t.id === activeTab ? ' active' : ''}`;
    btn.dataset.target = t.id;
    btn.setAttribute('role', 'tab');
    btn.setAttribute('aria-selected', String(t.id === activeTab));
    btn.textContent = t.label;
    tabContainer.appendChild(btn);
    tabEls[t.id] = btn;

    btn.addEventListener('click', () => {
      activeTab = t.id;
      Object.values(tabEls).forEach(b => {
        b.classList.remove('active');
        b.setAttribute('aria-selected', 'false');
      });
      btn.classList.add('active');
      btn.setAttribute('aria-selected', 'true');
      store.set('target', t.id);
      updateOutputLabel();
    });
  });

  const actions = document.createElement('div');
  actions.className = 'panel-actions';

  const actionBtns = [
    { id: 'copy', icon: 'copy', label: 'Copy', shortcut: 'Ctrl+C' },
    { id: 'download', icon: 'download', label: 'Download', shortcut: 'Ctrl+S' },
    { id: 'format', icon: 'format', label: 'Format', shortcut: '' },
    { type: 'separator' },
    { id: 'fullscreen', icon: 'fullscreen', label: 'Expand', shortcut: '' },
    { id: 'playground', icon: 'playground', label: 'Playground', shortcut: '' },
  ];

  const actionEls = {};

  actionBtns.forEach(btn => {
    if (btn.type === 'separator') {
      const sep = document.createElement('div');
      sep.className = 'panel-action-btn separator';
      actions.appendChild(sep);
      return;
    }

    const b = document.createElement('button');
    b.className = 'panel-action-btn';
    if (btn.id === 'fullscreen' || btn.id === 'playground') {
      b.classList.add('icon-only');
    }
    b.dataset.action = btn.id;
    b.innerHTML = icon(btn.icon) + (btn.id !== 'fullscreen' && btn.id !== 'playground' ? `<span>${btn.label}</span>` : '');
    b.setAttribute('aria-label', `${btn.label}${btn.shortcut ? ` (${btn.shortcut})` : ''}`);
    actions.appendChild(b);
    actionEls[btn.id] = b;
  });

  toolbar.appendChild(tabContainer);
  toolbar.appendChild(actions);

  /* ----- Code output area ----- */
  const codeContainer = document.createElement('div');
  codeContainer.className = 'editor-container';

  const codeEl = document.createElement('pre');
  codeEl.className = 'output-code';
  codeEl.setAttribute('aria-label', 'Generated code output');

  codeContainer.appendChild(codeEl);

  el.appendChild(toolbar);
  el.appendChild(codeContainer);

  /* ----- Empty state ----- */
  const emptyState = document.createElement('div');
  emptyState.className = 'empty-state';

  const ill = document.createElement('div');
  ill.className = 'empty-state-illustration';
  ill.innerHTML = `<svg viewBox="0 0 120 120" fill="none" stroke="currentColor" stroke-width="1">
    <rect x="20" y="20" width="80" height="80" rx="8" stroke-dasharray="4 4"/>
    <path d="M35 45l10 10-10 10M55 55h30" stroke-dasharray="2 2" stroke-linecap="round"/>
    <path d="M35 75l10 10-10 10M55 85h30" stroke-dasharray="2 2" stroke-linecap="round"/>
  </svg>`;

  const title = document.createElement('h2');
  title.textContent = 'Ready to compile';
  title.id = 'output-empty-title';

  const subtitle = document.createElement('p');
  subtitle.textContent = 'Enter HTML and CSS on the left, then hit Compile to generate native UI code.';
  subtitle.id = 'output-empty-subtitle';

  emptyState.appendChild(ill);
  emptyState.appendChild(title);
  emptyState.appendChild(subtitle);

  codeContainer.appendChild(emptyState);

  /* ----- Loading overlay ----- */
  const loadingOverlay = document.createElement('div');
  loadingOverlay.className = 'loading-overlay';

  const loadingContent = document.createElement('div');
  loadingContent.className = 'loading-overlay-content';

  const loadingMsg = document.createElement('div');
  loadingMsg.className = 'loading-message';

  const spinner = document.createElement('div');
  spinner.className = 'spinner';

  const loadingText = document.createElement('span');
  loadingText.id = 'loading-status-text';
  loadingText.textContent = 'Processing…';

  loadingMsg.appendChild(spinner);
  loadingMsg.appendChild(loadingText);

  const progressContainer = document.createElement('div');
  progressContainer.className = 'loading-progress';

  const progressBar = document.createElement('div');
  progressBar.className = 'loading-progress-bar';
  progressBar.id = 'loading-progress-bar';

  progressContainer.appendChild(progressBar);

  const statusMsg = document.createElement('div');
  statusMsg.className = 'loading-status';
  statusMsg.id = 'loading-status-msg';
  statusMsg.textContent = 'Parsing HTML document…';

  loadingContent.appendChild(loadingMsg);
  loadingContent.appendChild(progressContainer);
  loadingContent.appendChild(statusMsg);
  loadingOverlay.appendChild(loadingContent);
  codeContainer.appendChild(loadingOverlay);

  /* ----- Completion overlay ----- */
  const completionOverlay = document.createElement('div');
  completionOverlay.className = 'completion-overlay';

  const check = document.createElement('div');
  check.className = 'completion-check';
  check.innerHTML = icon('check');

  completionOverlay.appendChild(check);
  codeContainer.appendChild(completionOverlay);

  /* ----- Functions ----- */
  function updateOutputLabel() {
    const target = TARGETS.find(t => t.id === store.get('target'));
    if (target) {
      toolbar.querySelectorAll('.panel-tab').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.target === target.id);
      });
    }
  }

  function showCode(code) {
    emptyState.style.display = 'none';
    codeEl.textContent = code;
    codeEl.style.display = '';
  }

  function showEmpty() {
    codeEl.textContent = '';
    codeEl.style.display = 'none';
    emptyState.style.display = '';
    loadingOverlay.classList.remove('visible');
    completionOverlay.classList.remove('visible');
  }

  function showLoading() {
    emptyState.style.display = 'none';
    codeEl.style.display = 'none';
    loadingOverlay.classList.add('visible');
    completionOverlay.classList.remove('visible');
  }

  function hideLoading() {
    loadingOverlay.classList.remove('visible');
  }

  function showCompletion() {
    completionOverlay.classList.add('visible');
    setTimeout(() => completionOverlay.classList.remove('visible'), 1200);
  }

  function updateLoadingStatus(stage, message) {
    const pct = ((stage) / (5)) * 100;
    progressBar.style.width = `${Math.min(pct, 100)}%`;
    statusMsg.textContent = message;
    loadingText.textContent = message;
  }

  function updateLoadingProgress(pct) {
    progressBar.style.width = `${Math.min(pct, 100)}%`;
  }

  /* ----- Action handlers ----- */
  actionEls.copy?.addEventListener('click', async () => {
    const code = store.get('code');
    if (!code) {
      notify('Nothing to copy — compile something first', 'info');
      return;
    }
    try {
      await navigator.clipboard.writeText(code);
      const btn = actionEls.copy;
      btn.innerHTML = icon('check') + '<span>Copied</span>';
      btn.classList.add('copy-success');
      setTimeout(() => {
        btn.innerHTML = icon('copy') + '<span>Copy</span>';
        btn.classList.remove('copy-success');
      }, 2000);
      notify('Code copied to clipboard', 'success', 2000);
    } catch {
      notify('Failed to copy to clipboard', 'error');
    }
  });

  actionEls.download?.addEventListener('click', () => {
    const code = store.get('code');
    const target = store.get('target');
    if (!code) {
      notify('Nothing to download — compile something first', 'info');
      return;
    }
    const ext = TARGETS.find(t => t.id === target)?.ext || 'txt';
    const blob = new Blob([code], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `GeneratedView.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
    notify('File downloaded', 'success', 2000);
  });

  actionEls.fullscreen?.addEventListener('click', () => {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
    } else {
      document.exitFullscreen();
    }
  });

  actionEls.playground?.addEventListener('click', () => {
    const code = store.get('code');
    const target = store.get('target');
    if (!code) { notify('Nothing to open — compile something first', 'info'); return; }
    const ext = TARGETS.find(t => t.id === target)?.ext || 'txt';
    const blob = new Blob([code], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    window.open(url, '_blank');
    setTimeout(() => URL.revokeObjectURL(url), 60000);
    notify('Code opened in new tab', 'success', 2000);
  });

  showEmpty();

  return {
    el,
    showCode,
    showEmpty,
    showLoading,
    hideLoading,
    showCompletion,
    updateLoadingStatus,
    updateLoadingProgress,
    getOutputEl: () => codeEl,
  };
}
