class Store {
  constructor(initial = {}) {
    this._state = { ...initial };
    this._listeners = {};
  }

  get(key) {
    return this._state[key];
  }

  set(key, value) {
    const prev = this._state[key];
    if (prev === value) return;
    this._state[key] = value;
    this._emit(key, value, prev);
  }

  on(key, fn) {
    if (!this._listeners[key]) this._listeners[key] = new Set();
    this._listeners[key].add(fn);
    return () => this._listeners[key].delete(fn);
  }

  _emit(key, value, prev) {
    const fns = this._listeners[key];
    if (fns) fns.forEach(fn => fn(value, prev));
  }

  state() {
    return { ...this._state };
  }
}

export const store = new Store({
  html: '',
  css: '',
  target: 'flutter',
  code: '',
  status: 'idle',
  pipelineStage: -1,
  stats: null,
  error: null,
  backendOnline: true,
  panelRatio: 0.5,
  editorTab: 'html',
  notifications: [],
});
