import { icon } from '../utils/icons.js';

let container;

function getContainer() {
  if (!container) {
    container = document.createElement('div');
    container.className = 'notification-container';
    container.setAttribute('role', 'status');
    container.setAttribute('aria-live', 'polite');
    document.body.appendChild(container);
  }
  return container;
}

export function notify(message, type = 'info', duration = 3000) {
  const c = getContainer();
  const el = document.createElement('div');
  el.className = `notification notification-${type}`;
  el.setAttribute('role', 'alert');

  const iconMap = { success: 'check', error: 'error', info: 'info', warning: 'warning' };

  el.innerHTML = `
    <span class="notification-icon">${icon(iconMap[type] || 'info')}</span>
    <span class="notification-message">${message}</span>
    <button class="notification-close" aria-label="Dismiss">${icon('close')}</button>
  `;

  el.querySelector('.notification-close').addEventListener('click', () => remove(el));
  c.appendChild(el);

  const timer = setTimeout(() => remove(el), duration);

  function remove(el) {
    clearTimeout(timer);
    el.classList.add('removing');
    setTimeout(() => el.remove(), 200);
  }

  return el;
}
