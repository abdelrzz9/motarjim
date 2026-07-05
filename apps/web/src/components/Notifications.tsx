import { useNotificationsStore, type NotificationType } from '../stores/notificationsStore';
import { Icon } from './Icons';

const NOTIFICATION_ICONS: Record<NotificationType, typeof Icon.Info> = {
  success: Icon.Check,
  error: Icon.Error,
  info: Icon.Info,
  warning: Icon.Warning,
};

const NOTIFICATION_CLASSES: Record<NotificationType, string> = {
  success: 'notification-success',
  error: 'notification-error',
  info: 'notification-info',
  warning: 'notification-warning',
};

export default function Notifications() {
  const { notifications, removeNotification } = useNotificationsStore();

  if (notifications.length === 0) return null;

  return (
    <div
      style={{
        position: 'fixed',
        bottom: 'var(--space-6)',
        right: 'var(--space-6)',
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-2)',
        zIndex: 100,
        pointerEvents: 'none',
      }}
    >
      {notifications.map((n) => {
        const IconComp = NOTIFICATION_ICONS[n.type];
        return (
          <div
            key={n.id}
            className={NOTIFICATION_CLASSES[n.type]}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 'var(--space-3)',
              padding: 'var(--space-3) var(--space-4)',
              background: 'var(--bg-elevated)',
              border: '1px solid var(--border-default)',
              borderRadius: 'var(--radius-md)',
              boxShadow: 'var(--shadow-md)',
              fontSize: 'var(--text-sm)',
              color: 'var(--text-primary)',
              pointerEvents: 'auto',
              animation: 'slide-in-right var(--duration-slow) var(--ease-out)',
              minWidth: 280,
              maxWidth: 400,
            }}
          >
            <span style={{ width: 16, height: 16, flexShrink: 0, color: `var(--${n.type === 'error' ? 'error' : n.type === 'warning' ? 'warning' : n.type === 'success' ? 'success' : 'info'})` }}>
              <IconComp size={16} />
            </span>
            <span style={{ flex: 1 }}>{n.message}</span>
            <button
              onClick={() => removeNotification(n.id)}
              style={{
                border: 'none',
                background: 'transparent',
                color: 'var(--text-tertiary)',
                cursor: 'pointer',
                padding: 2,
                display: 'flex',
                transition: 'color var(--duration-fast)',
              }}
            >
              <Icon.Close size={12} />
            </button>
          </div>
        );
      })}
    </div>
  );
}
