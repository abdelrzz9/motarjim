import './styles/index.css';
import { icon } from './utils/icons.js';
import { store } from './state/store.js';
import { convertCode, healthCheck } from './utils/api.js';
import { onShortcut } from './utils/keyboard.js';
import { PIPELINE_STAGES, STATUS_MESSAGES, TARGETS } from './constants.js';
import { createPipeline } from './components/Pipeline.js';
import { createEditorPanel } from './components/EditorPanel.js';
import { createOutputPanel } from './components/OutputPanel.js';
import { createStatusBar } from './components/StatusBar.js';
import { createCommandPalette } from './components/CommandPalette.js';
import { notify } from './components/Notifications.js';

/* ==========================================================================
   APP INIT
   ========================================================================== */
const app = document.getElementById('app');
app.innerHTML = '';

/* ----- Layout shells ----- */
const appMain = document.createElement('div');
appMain.className = 'app-main';

const appContent = document.createElement('div');
appContent.className = 'app-content';

/* ==========================================================================
   TOP NAV
   ========================================================================== */
const topnav = document.createElement('div');
topnav.className = 'topnav';
topnav.setAttribute('role', 'navigation');
topnav.setAttribute('aria-label', 'Main navigation');

const brand = document.createElement('div');
brand.className = 'topnav-brand';
brand.innerHTML = `<span class="topnav-brand-icon">${icon('logo')}</span>motarjim<small>Compiler</small>`;

const navDivider = document.createElement('div');
navDivider.className = 'topnav-divider';

/* Platform selector */
const platforms = document.createElement('div');
platforms.className = 'topnav-platforms';

TARGETS.forEach(t => {
  const btn = document.createElement('button');
  btn.className = `platform-btn${t.id === 'flutter' ? ' active' : ''}`;
  btn.dataset.target = t.id;
  btn.setAttribute('aria-pressed', String(t.id === 'flutter'));
  btn.innerHTML = `<span class="dot"></span><span>${t.label}</span>`;
  btn.addEventListener('click', () => {
    platforms.querySelectorAll('.platform-btn').forEach(b => {
      b.classList.remove('active');
      b.setAttribute('aria-pressed', 'false');
    });
    btn.classList.add('active');
    btn.setAttribute('aria-pressed', 'true');
    store.set('target', t.id);
  });
  platforms.appendChild(btn);
});

const spacer = document.createElement('div');
spacer.className = 'topnav-spacer';

/* Settings & Theme */
const settingsBtn = document.createElement('button');
settingsBtn.className = 'topnav-btn';
settingsBtn.innerHTML = icon('settings');
settingsBtn.setAttribute('aria-label', 'Settings');
settingsBtn.setAttribute('title', 'Settings');

/* Theme toggle */
function getTheme() {
  return window.localStorage.getItem('motarjim:theme') || 'dark';
}

function setTheme(theme) {
  document.documentElement.classList.toggle('light', theme === 'light');
  document.documentElement.classList.toggle('dark', theme === 'dark');
  window.localStorage.setItem('motarjim:theme', theme);
}

setTheme(getTheme());

const themeBtn = document.createElement('button');
themeBtn.className = 'topnav-btn';
themeBtn.innerHTML = icon('theme');
themeBtn.setAttribute('aria-label', 'Toggle theme');
themeBtn.setAttribute('title', 'Toggle theme');
themeBtn.addEventListener('click', () => {
  const current = getTheme();
  setTheme(current === 'dark' ? 'light' : 'dark');
});

/* Compile button */
const compileBtn = document.createElement('button');
compileBtn.className = 'compile-btn';
compileBtn.id = 'compile-btn';
compileBtn.innerHTML = icon('compile') + '<span>Compile</span>';
compileBtn.setAttribute('aria-label', 'Compile code');
compileBtn.setAttribute('title', 'Compile (Ctrl+Enter)');

topnav.appendChild(brand);
topnav.appendChild(navDivider);
topnav.appendChild(platforms);
topnav.appendChild(spacer);
topnav.appendChild(settingsBtn);
topnav.appendChild(themeBtn);
topnav.appendChild(compileBtn);

/* ==========================================================================
   PIPELINE
   ========================================================================== */
const pipeline = createPipeline();

/* ==========================================================================
   SPLIT PANELS
   ========================================================================== */
const splitPanels = document.createElement('div');
splitPanels.className = 'split-panels';
splitPanels.setAttribute('role', 'region');
splitPanels.setAttribute('aria-label', 'Editor and output panels');

const editorPanel = createEditorPanel();
const outputPanel = createOutputPanel();

/* Resize handle */
const resizeHandle = document.createElement('div');
resizeHandle.className = 'resize-handle';
resizeHandle.setAttribute('role', 'separator');
resizeHandle.setAttribute('aria-label', 'Resize panels');
resizeHandle.setAttribute('tabindex', '0');

let isResizing = false;

resizeHandle.addEventListener('mousedown', (e) => {
  isResizing = true;
  document.body.style.cursor = 'col-resize';
  document.body.style.userSelect = 'none';
  const startX = e.clientX;
  const startRatio = store.get('panelRatio') || 0.5;
  const parent = splitPanels;
  const parentRect = parent.getBoundingClientRect();

  function onMove(e) {
    if (!isResizing) return;
    const dx = e.clientX - startX;
    const pct = Math.max(0.3, Math.min(0.7, startRatio + dx / parentRect.width));
    store.set('panelRatio', pct);
    editorPanel.el.style.flex = pct;
    outputPanel.el.style.flex = 1 - pct;
  }

  function onUp() {
    isResizing = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
  }

  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
});

/* Keyboard resize */
resizeHandle.addEventListener('keydown', (e) => {
  const step = 0.02;
  let ratio = store.get('panelRatio') || 0.5;
  if (e.key === 'ArrowLeft') {
    ratio = Math.max(0.3, ratio - step);
  } else if (e.key === 'ArrowRight') {
    ratio = Math.min(0.7, ratio + step);
  } else return;
  store.set('panelRatio', ratio);
  editorPanel.el.style.flex = ratio;
  outputPanel.el.style.flex = 1 - ratio;
});

const initialRatio = store.get('panelRatio') || 0.5;
editorPanel.el.style.flex = initialRatio;
outputPanel.el.style.flex = 1 - initialRatio;

splitPanels.appendChild(editorPanel.el);
splitPanels.appendChild(resizeHandle);
splitPanels.appendChild(outputPanel.el);

/* ==========================================================================
   STATUS BAR
   ========================================================================== */
const statusBar = createStatusBar();

/* ==========================================================================
   COMMAND PALETTE
   ========================================================================== */
const commandPalette = createCommandPalette({ compile: runCompile });

/* ==========================================================================
   ASSEMBLE
   ========================================================================== */
appContent.appendChild(splitPanels);

appMain.appendChild(pipeline.el);
appMain.appendChild(appContent);
appMain.appendChild(statusBar.el);

app.appendChild(topnav);
app.appendChild(appMain);

/* ==========================================================================
   COMPILE LOGIC
   ========================================================================== */
async function runCompile() {
  const html = editorPanel.getHTML().trim();
  if (!html) {
    notify('Paste some HTML first — the engine needs at least one element to compile.', 'warning');
    return;
  }

  const target = store.get('target');
  compileBtn.disabled = true;
  compileBtn.innerHTML = `<div class="spinner" style="width:14px;height:14px;border:2px solid rgba(255,255,255,0.3);border-top-color:white;border-radius:50%;animation:spin .8s linear infinite;"></div><span>Compiling…</span>`;
  store.set('status', 'compiling');
  store.set('error', null);
  store.set('stats', null);
  store.set('code', '');
  outputPanel.showLoading();
  pipeline.reset();

  const stageTimers = [];

  function animatePipeline() {
    return new Promise((resolve) => {
      PIPELINE_STAGES.forEach((stage, i) => {
        const delay = i * 250;
        stageTimers.push(setTimeout(() => {
          pipeline.setStage(i);
          outputPanel.updateLoadingStatus(i, STATUS_MESSAGES[stage.id]);
        }, delay));
      });
      stageTimers.push(setTimeout(resolve, PIPELINE_STAGES.length * 250 + 300));
    });
  }

  try {
    const startTime = performance.now();

    const [apiResult] = await Promise.all([
      convertCode({ html, css: editorPanel.getCSS(), target }),
      animatePipeline(),
    ]);

    const duration = (performance.now() - startTime) / 1000;

    stageTimers.forEach(clearTimeout);
    pipeline.complete();

    store.set('code', apiResult.code);
    outputPanel.showCode(apiResult.code);
    outputPanel.hideLoading();
    outputPanel.showCompletion();

    store.set('stats', {
      ...apiResult.stats,
      duration: apiResult.stats?.duration ?? duration,
      warnings: apiResult.stats?.warnings ?? 0,
    });

    notify(`Generated ${apiResult.stats?.generatedLines ?? 0} lines in ${duration.toFixed(2)}s`, 'success', 3000);
  } catch (err) {
    stageTimers.forEach(clearTimeout);
    pipeline.reset();

    const errorMsg = err.message || 'Conversion failed. Please check your HTML and CSS.';

    store.set('error', errorMsg);
    outputPanel.hideLoading();
    outputPanel.showEmpty();
    outputPanel.getOutputEl().style.display = 'none';

    showErrorCard(errorMsg);
    notify(errorMsg, 'error', 5000);
  } finally {
    compileBtn.disabled = false;
    compileBtn.innerHTML = icon('compile') + '<span>Compile</span>';
    store.set('status', 'idle');
  }
}

/* ----- Error card ----- */
function showErrorCard(message) {
  const existing = document.querySelector('.error-card');
  if (existing) existing.remove();

  const card = document.createElement('div');
  card.className = 'error-card';
  card.setAttribute('role', 'alert');

  const lineMatch = message.match(/line (\d+)/i);
  const lineInfo = lineMatch ? lineMatch[1] : null;

  const expanded = window.localStorage.getItem('motarjim:errorExpanded') === 'true';

  card.innerHTML = `
    <div class="error-card-header">
      <div class="error-card-icon">${icon('error')}</div>
      <div class="error-card-body">
        <div class="error-card-title">Compilation Error</div>
        <div class="error-card-description">${escapeHtml(message)}</div>
        <div class="error-card-meta">
          ${lineInfo ? `<span>Line: <b>${lineInfo}</b></span>` : ''}
          <span>Check your HTML syntax and CSS selectors</span>
        </div>
        <div class="error-card-actions">
          <button class="btn btn-sm btn-ghost" data-action="copy-error">${icon('copy')} Copy Error</button>
          <button class="btn btn-sm btn-ghost" data-action="toggle-details">${icon('expand')} Details</button>
        </div>
        <div class="error-card-details ${expanded ? 'visible' : ''}">${escapeHtml(message)}\n\nTry checking:\n• HTML tags are properly closed\n• CSS selectors match existing elements\n• No syntax errors in your code</div>
      </div>
    </div>
  `;

  const container = outputPanel.getOutputEl().parentElement;
  container.appendChild(card);

  card.querySelector('[data-action="copy-error"]').addEventListener('click', () => {
    navigator.clipboard.writeText(message);
    notify('Error copied to clipboard', 'success', 2000);
  });

  const detailsEl = card.querySelector('.error-card-details');
  const toggleBtn = card.querySelector('[data-action="toggle-details"]');
  toggleBtn.addEventListener('click', () => {
    const isVisible = detailsEl.classList.toggle('visible');
    toggleBtn.innerHTML = icon(isVisible ? 'collapse' : 'expand') + (isVisible ? ' Hide' : ' Details');
    window.localStorage.setItem('motarjim:errorExpanded', String(isVisible));
  });
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

/* ----- Compile events ----- */
compileBtn.addEventListener('click', runCompile);

store.on('target', (target) => {
  const btn = platforms.querySelector(`[data-target="${target}"]`);
  if (btn) {
    platforms.querySelectorAll('.platform-btn').forEach(b => {
      b.classList.remove('active');
      b.setAttribute('aria-pressed', 'false');
    });
    btn.classList.add('active');
    btn.setAttribute('aria-pressed', 'true');
  }
});

/* ==========================================================================
   AUTO-SAVE DRAFTS
   ========================================================================== */
function saveDraft() {
  try {
    const draft = {
      html: editorPanel.getHTML(),
      css: editorPanel.getCSS(),
      savedAt: Date.now(),
    };
    window.localStorage.setItem('motarjim:draft', JSON.stringify(draft));
  } catch { /* storage full */ }
}

let saveTimer = null;
store.on('html', () => {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(saveDraft, 1000);
});
store.on('css', () => {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(saveDraft, 1000);
});

function restoreDraft() {
  try {
    const raw = window.localStorage.getItem('motarjim:draft');
    if (raw) {
      const draft = JSON.parse(raw);
      if (draft.html) editorPanel.setHTML(draft.html);
      if (draft.css) editorPanel.setCSS(draft.css);
    }
  } catch { /* ignore */ }
}
restoreDraft();

/* ==========================================================================
   KEYBOARD SHORTCUTS
   ========================================================================== */
onShortcut('Ctrl+Enter', runCompile);

onShortcut('Ctrl+K', () => commandPalette.toggle());

onShortcut('?', () => commandPalette.toggle());

/* Target switchers */
onShortcut('Ctrl+1', () => store.set('target', 'flutter'));
onShortcut('Ctrl+2', () => store.set('target', 'compose'));
onShortcut('Ctrl+3', () => store.set('target', 'swiftui'));

/* ==========================================================================
   BACKEND HEALTH CHECK
   ========================================================================== */
async function checkBackend() {
  const online = await healthCheck();
  store.set('backendOnline', online);
}

checkBackend();
setInterval(checkBackend, 30000);

/* ==========================================================================
   KEYBOARD SHORTCUT HELP (Ctrl+/)
   ========================================================================== */
document.addEventListener('keydown', (e) => {
  if (e.key === '?' && !e.ctrlKey && !e.metaKey && !e.shiftKey && !e.altKey) {
    const active = document.activeElement;
    if (active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA')) return;
    commandPalette.toggle();
  }
});
