import { icon } from '../utils/icons.js';
import { TARGETS } from '../constants.js';
import { notify } from './Notifications.js';
import { store } from '../state/store.js';

export function createCommandPalette({ compile }) {
  const overlay = document.createElement('div');
  overlay.className = 'command-palette-overlay';
  overlay.setAttribute('role', 'dialog');
  overlay.setAttribute('aria-label', 'Command palette');
  overlay.setAttribute('tabindex', '-1');

  const palette = document.createElement('div');
  palette.className = 'command-palette';

  const inputContainer = document.createElement('div');
  inputContainer.className = 'command-palette-input';

  inputContainer.innerHTML = icon('command');

  const input = document.createElement('input');
  input.type = 'text';
  input.placeholder = 'Search commands…';
  input.setAttribute('aria-label', 'Search commands');
  inputContainer.appendChild(input);
  palette.appendChild(inputContainer);

  const results = document.createElement('div');
  results.className = 'command-palette-results';
  results.setAttribute('role', 'listbox');
  palette.appendChild(results);

  overlay.appendChild(palette);
  document.body.appendChild(overlay);

  let isOpen = false;
  let selectedIndex = -1;

  const commands = [
    { id: 'compile', label: 'Compile code', shortcut: 'Ctrl+Enter', icon: 'play', action: () => compile() },
    { id: 'copy', label: 'Copy output', shortcut: 'Ctrl+C', icon: 'copy', action: () => document.querySelector('[data-action="copy"]')?.click() },
    { id: 'download', label: 'Download output', shortcut: 'Ctrl+S', icon: 'download', action: () => document.querySelector('[data-action="download"]')?.click() },
    { id: 'format', label: 'Format code', shortcut: 'Ctrl+Shift+F', icon: 'format', action: () => {} },
    { id: 'flutter', label: 'Switch to Flutter', shortcut: 'Ctrl+1', icon: 'arrowRight', action: () => store.set('target', 'flutter') },
    { id: 'compose', label: 'Switch to Compose', shortcut: 'Ctrl+2', icon: 'arrowRight', action: () => store.set('target', 'compose') },
    { id: 'swiftui', label: 'Switch to SwiftUI', shortcut: 'Ctrl+3', icon: 'arrowRight', action: () => store.set('target', 'swiftui') },
    { id: 'sample', label: 'Load sample project', shortcut: '', icon: 'sample', action: () => document.querySelector('[data-action="sample"]')?.click() },
    { id: 'shortcuts', label: 'Keyboard shortcuts', shortcut: '?', icon: 'keyboard', action: () => showShortcutsModal() },
  ];

  function render(query = '') {
    const q = query.toLowerCase().trim();
    const filtered = q ? commands.filter(c =>
      c.label.toLowerCase().includes(q) || c.id.toLowerCase().includes(q)
    ) : commands;

    selectedIndex = -1;
    results.innerHTML = '';

    if (filtered.length === 0) {
      results.innerHTML = '<div class="command-palette-empty">No commands found</div>';
      return;
    }

    filtered.forEach((cmd, i) => {
      const item = document.createElement('div');
      item.className = 'command-palette-item';
      item.dataset.index = i;
      item.setAttribute('role', 'option');
      item.setAttribute('tabindex', '-1');

      const iconSpan = document.createElement('span');
      iconSpan.className = 'command-palette-item-icon';
      iconSpan.innerHTML = icon(cmd.icon);

      const label = document.createElement('span');
      label.className = 'command-palette-item-label';
      label.textContent = cmd.label;

      item.appendChild(iconSpan);
      item.appendChild(label);

      if (cmd.shortcut) {
        const shortcut = document.createElement('span');
        shortcut.className = 'command-palette-item-shortcut';
        shortcut.textContent = cmd.shortcut;
        item.appendChild(shortcut);
      }

      item.addEventListener('click', () => {
        cmd.action();
        close();
      });

      item.addEventListener('mouseenter', () => {
        document.querySelectorAll('.command-palette-item').forEach(el => el.classList.remove('selected'));
        item.classList.add('selected');
        selectedIndex = i;
      });

      results.appendChild(item);
    });
  }

  function open() {
    isOpen = true;
    overlay.classList.add('visible');
    input.value = '';
    input.focus();
    render();
  }

  function close() {
    isOpen = false;
    overlay.classList.remove('visible');
    input.blur();
  }

  function toggle() {
    isOpen ? close() : open();
  }

  /* ----- Keyboard navigation ----- */
  input.addEventListener('keydown', (e) => {
    const items = results.querySelectorAll('.command-palette-item');

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
        updateSelection(items);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        updateSelection(items);
        break;
      case 'Enter':
        e.preventDefault();
        if (selectedIndex >= 0 && items[selectedIndex]) {
          items[selectedIndex].click();
        }
        break;
      case 'Escape':
        e.preventDefault();
        close();
        break;
    }
  });

  function updateSelection(items) {
    items.forEach((el, i) => {
      el.classList.toggle('selected', i === selectedIndex);
      if (i === selectedIndex) el.scrollIntoView({ block: 'nearest' });
    });
  }

  input.addEventListener('input', () => render(input.value));

  overlay.addEventListener('click', (e) => {
    if (e.target === overlay) close();
  });

  /* ----- Shortcuts modal ----- */
  let shortcutsModal = null;

  function showShortcutsModal() {
    if (shortcutsModal) {
      shortcutsModal.remove();
      shortcutsModal = null;
      return;
    }

    const mo = document.createElement('div');
    mo.className = 'modal-overlay visible';

    const m = document.createElement('div');
    m.className = 'modal';

    m.innerHTML = `
      <div class="modal-header">
        <span class="modal-title">Keyboard Shortcuts</span>
        <button class="modal-close" aria-label="Close">${icon('close')}</button>
      </div>
      <div class="modal-body">
        <table>
          <thead><tr><th>Shortcut</th><th>Action</th></tr></thead>
          <tbody>
            <tr><td><kbd>Ctrl</kbd> + <kbd>Enter</kbd></td><td>Compile code</td></tr>
            <tr><td><kbd>Ctrl</kbd> + <kbd>C</kbd></td><td>Copy output</td></tr>
            <tr><td><kbd>Ctrl</kbd> + <kbd>S</kbd></td><td>Download output</td></tr>
            <tr><td><kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>F</kbd></td><td>Format code</td></tr>
            <tr><td><kbd>Ctrl</kbd> + <kbd>K</kbd></td><td>Command palette</td></tr>
            <tr><td><kbd>Ctrl</kbd> + <kbd>1</kbd></td><td>Flutter target</td></tr>
            <tr><td><kbd>Ctrl</kbd> + <kbd>2</kbd></td><td>Compose target</td></tr>
            <tr><td><kbd>Ctrl</kbd> + <kbd>3</kbd></td><td>SwiftUI target</td></tr>
            <tr><td><kbd>?</kbd></td><td>Toggle this modal</td></tr>
          </tbody>
        </table>
      </div>
    `;

    mo.appendChild(m);
    document.body.appendChild(mo);
    shortcutsModal = mo;

    const close = () => {
      mo.remove();
      shortcutsModal = null;
    };

    mo.querySelector('.modal-close').addEventListener('click', close);
    mo.addEventListener('click', (e) => { if (e.target === mo) close(); });

    document.addEventListener('keydown', function handler(e) {
      if (e.key === 'Escape') { close(); document.removeEventListener('keydown', handler); }
    });
  }

  render();

  return { toggle, open, close, el: overlay };
}
