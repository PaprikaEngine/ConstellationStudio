import React, { useState, useEffect } from 'react';
import { AlertCircle, CheckCircle, Info, X, AlertTriangle } from 'lucide-react';
import { useTheme, getThemeColors } from '../contexts/ThemeContext';

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  duration?: number; // milliseconds, undefined = permanent
  timestamp: number;
}

interface NotificationSystemProps {
  notifications: Notification[];
  onDismiss: (id: string) => void;
}

export const NotificationSystem: React.FC<NotificationSystemProps> = ({
  notifications,
  onDismiss,
}) => {
  const { isDark } = useTheme();
  const colors = getThemeColors(isDark);
  const [visibleNotifications, setVisibleNotifications] = useState<Notification[]>([]);

  useEffect(() => {
    setVisibleNotifications(notifications);

    // Auto-dismiss notifications with duration
    const timers = notifications.map((notification) => {
      if (notification.duration && notification.duration > 0) {
        return setTimeout(() => {
          onDismiss(notification.id);
        }, notification.duration);
      }
      return null;
    });

    // Cleanup function to clear all timers
    return () => {
      timers.forEach((timerId) => timerId && clearTimeout(timerId));
    };
  }, [notifications, onDismiss]);

  const getNotificationIcon = (type: NotificationType) => {
    switch (type) {
      case 'success':
        return <CheckCircle size={20} />;
      case 'error':
        return <AlertCircle size={20} />;
      case 'warning':
        return <AlertTriangle size={20} />;
      case 'info':
        return <Info size={20} />;
      default:
        return <Info size={20} />;
    }
  };

  const getNotificationStyles = (type: NotificationType) => {
    const baseStyles = {
      background: colors.surface,
      border: '1px solid',
      borderRadius: '12px',
      boxShadow: isDark ? '0 4px 16px rgba(0, 0, 0, 0.4)' : '0 4px 12px rgba(0, 0, 0, 0.15)',
      padding: '1rem',
      marginBottom: '0.5rem',
      minWidth: '300px',
      maxWidth: '400px',
      position: 'relative' as const,
      animation: 'slideInRight 0.4s cubic-bezier(0.4, 0, 0.2, 1)',
    };

    const getTypeColors = (type: NotificationType) => {
      if (isDark) {
        switch (type) {
          case 'success':
            return { border: colors.success, color: '#c6f6d5', bg: 'rgba(72, 187, 120, 0.1)' };
          case 'error':
            return { border: colors.error, color: '#fed7d7', bg: 'rgba(245, 101, 101, 0.1)' };
          case 'warning':
            return { border: colors.warning, color: '#feebc8', bg: 'rgba(237, 137, 54, 0.1)' };
          case 'info':
            return { border: colors.info, color: '#bee3f8', bg: 'rgba(66, 153, 225, 0.1)' };
          default:
            return { border: colors.border, color: colors.text, bg: colors.surface };
        }
      } else {
        switch (type) {
          case 'success':
            return { border: '#28a745', color: '#155724', bg: '#d4edda' };
          case 'error':
            return { border: '#dc3545', color: '#721c24', bg: '#f8d7da' };
          case 'warning':
            return { border: '#ffc107', color: '#856404', bg: '#fff3cd' };
          case 'info':
            return { border: '#17a2b8', color: '#0c5460', bg: '#d1ecf1' };
          default:
            return { border: colors.border, color: colors.text, bg: colors.surface };
        }
      }
    };

    const typeColors = getTypeColors(type);
    
    return {
      ...baseStyles,
      borderColor: typeColors.border,
      color: typeColors.color,
      backgroundColor: typeColors.bg,
    };
  };

  if (visibleNotifications.length === 0) {
    return null;
  }

  return (
    <>
      <style>
        {`
          @keyframes slideInRight {
            from {
              transform: translateX(100%);
              opacity: 0;
            }
            to {
              transform: translateX(0);
              opacity: 1;
            }
          }
          
          @keyframes slideOutRight {
            from {
              transform: translateX(0);
              opacity: 1;
            }
            to {
              transform: translateX(100%);
              opacity: 0;
            }
          }
        `}
      </style>
      <div
        style={{
          position: 'fixed',
          top: '1rem',
          right: '1rem',
          zIndex: 9999,
          pointerEvents: 'none',
        }}
      >
        {visibleNotifications.map((notification) => (
          <div
            key={notification.id}
            style={{
              ...getNotificationStyles(notification.type),
              pointerEvents: 'all',
            }}
          >
            <div style={{
              display: 'flex',
              alignItems: 'flex-start',
              gap: '0.75rem',
            }}>
              <div style={{ flexShrink: 0, marginTop: '0.125rem' }}>
                {getNotificationIcon(notification.type)}
              </div>
              
              <div style={{ flex: 1, minWidth: 0 }}>
                <h4 style={{
                  margin: '0 0 0.25rem 0',
                  fontSize: '0.875rem',
                  fontWeight: 'bold',
                  lineHeight: 1.3,
                }}>
                  {notification.title}
                </h4>
                
                <p style={{
                  margin: 0,
                  fontSize: '0.8125rem',
                  lineHeight: 1.4,
                  wordBreak: 'break-word',
                }}>
                  {notification.message}
                </p>
                
                <div style={{
                  marginTop: '0.5rem',
                  fontSize: '0.75rem',
                  opacity: 0.7,
                }}>
                  {new Date(notification.timestamp).toLocaleTimeString()}
                </div>
              </div>
              
              <button
                onClick={() => onDismiss(notification.id)}
                style={{
                  background: 'none',
                  border: 'none',
                  cursor: 'pointer',
                  padding: '0.25rem',
                  borderRadius: '4px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  opacity: 0.7,
                  transition: 'opacity 0.2s ease',
                  flexShrink: 0,
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.opacity = '1';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.opacity = '0.7';
                }}
              >
                <X size={16} />
              </button>
            </div>
            
            {/* Progress bar for timed notifications */}
            {notification.duration && notification.duration > 0 && (
              <div
                style={{
                  position: 'absolute',
                  bottom: 0,
                  left: 0,
                  right: 0,
                  height: '3px',
                  background: 'rgba(0, 0, 0, 0.1)',
                  borderRadius: '0 0 8px 8px',
                  overflow: 'hidden',
                }}
              >
                <div
                  style={{
                    height: '100%',
                    background: 'currentColor',
                    animation: `shrinkWidth ${notification.duration}ms linear`,
                    transformOrigin: 'left',
                  }}
                />
              </div>
            )}
          </div>
        ))}
      </div>
      
      <style>
        {`
          @keyframes shrinkWidth {
            from {
              transform: scaleX(1);
            }
            to {
              transform: scaleX(0);
            }
          }
        `}
      </style>
    </>
  );
};

// Hook for managing notifications
export const useNotifications = () => {
  const [notifications, setNotifications] = useState<Notification[]>([]);

  const addNotification = (
    type: NotificationType,
    title: string,
    message: string,
    duration: number = 5000
  ) => {
    const notification: Notification = {
      id: `notification-${Date.now()}-${Math.random()}`,
      type,
      title,
      message,
      duration,
      timestamp: Date.now(),
    };

    setNotifications(prev => [...prev, notification]);
    return notification.id;
  };

  const removeNotification = (id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id));
  };

  const clearAll = () => {
    setNotifications([]);
  };

  // Convenience methods
  const success = (title: string, message: string, duration?: number) =>
    addNotification('success', title, message, duration);
  
  const error = (title: string, message: string, duration?: number) =>
    addNotification('error', title, message, duration);
  
  const warning = (title: string, message: string, duration?: number) =>
    addNotification('warning', title, message, duration);
  
  const info = (title: string, message: string, duration?: number) =>
    addNotification('info', title, message, duration);

  return {
    notifications,
    addNotification,
    removeNotification,
    clearAll,
    success,
    error,
    warning,
    info,
  };
};