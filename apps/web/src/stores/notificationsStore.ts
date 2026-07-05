import { create } from 'zustand';

export type NotificationType = 'success' | 'error' | 'info' | 'warning';

export interface Notification {
  id: string;
  message: string;
  type: NotificationType;
  duration: number;
}

interface NotificationsStore {
  notifications: Notification[];
  notify: (message: string, type?: NotificationType, duration?: number) => void;
  removeNotification: (id: string) => void;
}

export const useNotificationsStore = create<NotificationsStore>((set, get) => ({
  notifications: [],
  notify: (message, type = 'info', duration = 3000) => {
    const id = Math.random().toString(36).slice(2);
    set((s) => ({ notifications: [...s.notifications, { id, message, type, duration }] }));
    if (duration > 0) {
      setTimeout(() => get().removeNotification(id), duration);
    }
  },
  removeNotification: (id) => {
    set((s) => ({ notifications: s.notifications.filter((n) => n.id !== id) }));
  },
}));
