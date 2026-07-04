const _handlers = new Map();

export function onShortcut(key, fn) {
  _handlers.set(key, fn);
}

document.addEventListener('keydown', (e) => {
  const parts = [];
  if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
  if (e.shiftKey) parts.push('Shift');
  if (e.altKey) parts.push('Alt');

  const key = e.key === ' ' ? 'Space' : e.key;
  if (key.length === 1) parts.push(key.toUpperCase());
  else parts.push(key);

  const combo = parts.join('+');

  const handler = _handlers.get(combo);
  if (handler) {
    e.preventDefault();
    handler(e);
  }
});

export function shortcuts() {
  return {
    'Ctrl+Enter': 'Compile',
    'Ctrl+C': 'Copy output',
    'Ctrl+S': 'Download output',
    'Ctrl+Shift+F': 'Format code',
    'Ctrl+K': 'Command palette',
    'Ctrl+1': 'Flutter target',
    'Ctrl+2': 'Compose target',
    'Ctrl+3': 'SwiftUI target',
  };
}
