import type { LogEntry } from './types';

const MAX_LOG_ENTRIES = 500;
const logBuffer: LogEntry[] = [];
let logEnabled = true;

function getTimestamp(): string {
  return new Date().toISOString();
}

export const logger = {
  enable() {
    logEnabled = true;
  },

  disable() {
    logEnabled = false;
  },

  debug(module: string, message: string, data?: unknown) {
    if (!logEnabled) return;
    const entry: LogEntry = {
      timestamp: getTimestamp(),
      level: 'debug',
      module,
      message,
      data,
    };
    appendLog(entry);
  },

  info(module: string, message: string, data?: unknown) {
    if (!logEnabled) return;
    const entry: LogEntry = {
      timestamp: getTimestamp(),
      level: 'info',
      module,
      message,
      data,
    };
    appendLog(entry);
  },

  warn(module: string, message: string, data?: unknown) {
    if (!logEnabled) return;
    const entry: LogEntry = {
      timestamp: getTimestamp(),
      level: 'warn',
      module,
      message,
      data,
    };
    appendLog(entry);
  },

  error(module: string, message: string, data?: unknown) {
    if (!logEnabled) return;
    const entry: LogEntry = {
      timestamp: getTimestamp(),
      level: 'error',
      module,
      message,
      data,
    };
    appendLog(entry);
    console.error(`[${module}] ${message}`, data);
  },

  timed<T>(module: string, operation: string, fn: () => T): T {
    const start = performance.now();
    try {
      const result = fn();
      const durationMs = performance.now() - start;
      this.info(module, `${operation} completed`, { durationMs: Math.round(durationMs) });
      return result;
    } catch (err) {
      const durationMs = performance.now() - start;
      this.error(module, `${operation} failed`, { durationMs: Math.round(durationMs), error: String(err) });
      throw err;
    }
  },

  async timedAsync<T>(module: string, operation: string, fn: () => Promise<T>): Promise<T> {
    const start = performance.now();
    try {
      const result = await fn();
      const durationMs = performance.now() - start;
      this.info(module, `${operation} completed`, { durationMs: Math.round(durationMs) });
      return result;
    } catch (err) {
      const durationMs = performance.now() - start;
      this.error(module, `${operation} failed`, { durationMs: Math.round(durationMs), error: String(err) });
      throw err;
    }
  },

  getLogs(): LogEntry[] {
    return [...logBuffer];
  },

  clearLogs() {
    logBuffer.length = 0;
  },
};

function appendLog(entry: LogEntry) {
  logBuffer.push(entry);
  if (logBuffer.length > MAX_LOG_ENTRIES) {
    logBuffer.shift();
  }
  if (import.meta.env.DEV) {
    const icon = entry.level === 'error' ? '✕' : entry.level === 'warn' ? '⚠' : entry.level === 'info' ? 'ℹ' : '→';
    console.debug(`${icon} [${entry.module}] ${entry.message}`, entry.data || '');
  }
}
