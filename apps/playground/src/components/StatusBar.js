import { icon } from '../utils/icons.js';
import { store } from '../state/store.js';

export function createStatusBar() {
  const el = document.createElement('div');
  el.className = 'statusbar';
  el.setAttribute('role', 'status');

  const left = document.createElement('div');
  left.className = 'statusbar-section';

  const items = {};

  const itemDefs = [
    { id: 'time', icon: 'play', label: 'Time', value: '—' },
    { id: 'nodes', icon: 'node', label: 'Nodes', value: '—' },
    { id: 'components', icon: 'search', label: 'Components', value: '—' },
    { id: 'lines', icon: 'format', label: 'Lines', value: '—' },
    { id: 'warnings', icon: 'warning', label: 'Warnings', value: '0' },
  ];

  itemDefs.forEach(def => {
    const item = document.createElement('div');
    item.className = 'statusbar-item';

    const iconSpan = document.createElement('span');
    iconSpan.innerHTML = icon(def.icon);

    const labelSpan = document.createElement('span');
    labelSpan.className = 'label';
    labelSpan.textContent = `${def.label}: `;

    const valueSpan = document.createElement('b');
    valueSpan.id = `stat-${def.id}`;
    valueSpan.textContent = def.value;

    item.appendChild(iconSpan);
    item.appendChild(labelSpan);
    item.appendChild(valueSpan);
    left.appendChild(item);
    items[def.id] = valueSpan;
  });

  /* ----- Right side ----- */
  const right = document.createElement('div');
  right.className = 'statusbar-section';

  const backendItem = document.createElement('div');
  backendItem.className = 'statusbar-item';

  const dot = document.createElement('span');
  dot.className = 'statusbar-dot online';
  dot.id = 'status-dot';

  const backendLabel = document.createElement('span');
  backendLabel.textContent = 'Engine';

  const backendStatus = document.createElement('b');
  backendStatus.id = 'status-text';
  backendStatus.textContent = 'Online';

  backendItem.appendChild(dot);
  backendItem.appendChild(backendLabel);
  backendItem.appendChild(document.createTextNode(' '));
  backendItem.appendChild(backendStatus);
  right.appendChild(backendItem);

  el.appendChild(left);
  el.appendChild(right);

  /* ----- Store listeners ----- */
  store.on('stats', (stats) => {
    if (!stats) {
      Object.values(items).forEach(el => el.textContent = '—');
      return;
    }
    if (stats.duration != null) items.time.textContent = `${stats.duration.toFixed(2)}s`;
    if (stats.htmlNodes != null) items.nodes.textContent = stats.htmlNodes;
    if (stats.componentsDetected != null) items.components.textContent = stats.componentsDetected;
    if (stats.generatedLines != null) items.lines.textContent = stats.generatedLines;
    if (stats.warnings != null) items.warnings.textContent = stats.warnings;
  });

  store.on('backendOnline', (online) => {
    dot.className = `statusbar-dot ${online ? 'online' : 'offline'}`;
    backendStatus.textContent = online ? 'Online' : 'Offline';
  });

  store.on('error', (err) => {
    items.warnings.textContent = err ? '1' : '0';
  });

  return { el };
}
