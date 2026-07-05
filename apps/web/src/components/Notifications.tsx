import { useNotificationsStore, type NotificationType } from '../stores/notificationsStore';
import { Icon } from './Icons';

const NOTIFICATION_ICONS: Record<NotificationType, typeof Icon.Info> = {
  success: Icon.Check,
  error: Icon.Error,
  info: Icon.Info,
  warning: Icon.Warning,
};

const NOTIFICATION_COLORS: Record<NotificationType, string> = {
  success: 'var(--success)',
  error: 'var(--error)',
  info: 'var(--info)',
  warning: 'var(--warning)',
};

function NotificationIcon({ type }: { type: NotificationType }) {
  const IconComp = NOTIFICATION_ICONS[type];
  return (
    <span style={{ width: 16, height: 16, flexShrink: 0, color: NOTIFICATION_COLORS[type] }}>
      <IconComp size={16} />
    </span>
  );
}

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
      role="log"
      aria-live="polite"
    >
      {notifications.map((n) => {
        const bg = n.type === 'error' ? 'rgba(239,68,68,0.1)'
          : n.type === 'warning' ? 'rgba(245,166,35,0.1)'
          : n.type === 'success' ? 'rgba(46,211,160,0.1)'
          : 'rgba(84,197,248,0.1)';

        const border = n.type === 'error' ? 'rgba(239,68,68,0.2)'
          : n.type === 'warning' ? 'rgba(245,166,35,0.2)'
          : n.type === 'success' ? 'rgba(46,211,160,0.2)'
          : 'var(--border-default)';

        return (
          <div
            key={n.id}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 'var(--space-3)',
              padding: 'var(--space-3) var(--space-4)',
              background: bg,
              border: `1px solid ${border}`,
              borderRadius: 'var(--radius-md)',
              boxShadow: 'var(--shadow-md)',
              fontSize: 'var(--text-sm)',
              color: 'var(--text-primary)',
              pointerEvents: 'auto',
              animation: 'slide-in-right 200ms var(--ease-out)',
              minWidth: 280,
              maxWidth: 400,
              backdropFilter: 'blur(8px)',
            }}
          >
            <NotificationIcon type={n.type} />
            <span style={{ flex: 1 }}>{n.message}</span>
            <button
              onClick={() => removeNotification(n.id)}
              aria-label="Dismiss notification"
              style={{
                border: 'none',
                background: 'transparent',
                color: 'var(--text-tertiary)',
                cursor: 'pointer',
                padding: 2,
                display: 'flex',
                transition: 'color 120ms',
                borderRadius: 4,
              }}
              onMouseEnter={(e) => { e.currentTarget.style.color = 'var(--text-secondary)'; }}
              onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--text-tertiary)'; }}
            >
              <Icon.Close size={12} />
            </button>
          </div>
        );
      })}
    </div>
  );
}
