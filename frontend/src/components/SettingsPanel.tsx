import React, { useState, useEffect } from 'react';
import { Settings, X, Monitor, Sun, Moon } from 'lucide-react';
import { useTheme, getThemeColors, getThemeStyles, Theme } from '../contexts/ThemeContext';

interface SettingsPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export const SettingsPanel: React.FC<SettingsPanelProps> = ({ isOpen, onClose }) => {
  const { theme, setTheme, isDark, effectiveTheme } = useTheme();
  const colors = getThemeColors(isDark);
  const styles = getThemeStyles(isDark);

  // Handle Escape key
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && isOpen) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown);
      return () => document.removeEventListener('keydown', handleKeyDown);
    }
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const themeOptions = [
    { value: 'system' as Theme, label: 'System', icon: <Monitor size={16} />, description: 'Follow system preference' },
    { value: 'light' as Theme, label: 'Light', icon: <Sun size={16} />, description: 'Light theme' },
    { value: 'dark' as Theme, label: 'Dark', icon: <Moon size={16} />, description: 'Dark theme' },
  ];

  return (
    <>
      {/* Backdrop */}
      <div
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0, 0, 0, 0.5)',
          zIndex: 9998,
          backdropFilter: 'blur(4px)',
        }}
        onClick={onClose}
      />

      {/* Settings Panel */}
      <div
        style={{
          position: 'fixed',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          width: '480px',
          maxWidth: '90vw',
          maxHeight: '80vh',
          background: colors.background,
          borderRadius: '16px',
          boxShadow: styles.shadowHeavy,
          border: `2px solid ${colors.borderLight}`,
          zIndex: 9999,
          overflow: 'hidden',
        }}
      >
        {/* Header */}
        <div
          style={{
            background: styles.headerGradient,
            color: '#ffffff',
            padding: '1.5rem',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
            <Settings size={24} />
            <h2 style={{
              margin: 0,
              fontSize: '1.25rem',
              fontWeight: '600',
              letterSpacing: '0.3px',
            }}>
              Settings
            </h2>
          </div>
          
          <button
            onClick={onClose}
            style={{
              background: 'rgba(255, 255, 255, 0.2)',
              border: '1px solid rgba(255, 255, 255, 0.3)',
              borderRadius: '8px',
              color: '#ffffff',
              cursor: 'pointer',
              padding: '0.5rem',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.2s ease',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'rgba(255, 255, 255, 0.3)';
              e.currentTarget.style.transform = 'scale(1.05)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'rgba(255, 255, 255, 0.2)';
              e.currentTarget.style.transform = 'scale(1)';
            }}
            title="Close Settings"
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div style={{ padding: '2rem' }}>
          {/* Theme Section */}
          <div style={{ marginBottom: '2rem' }}>
            <h3 style={{
              margin: '0 0 1rem 0',
              fontSize: '1rem',
              fontWeight: '600',
              color: colors.text,
              borderBottom: `2px solid ${colors.borderLight}`,
              paddingBottom: '0.5rem',
            }}>
              Appearance
            </h3>
            
            <div style={{ marginBottom: '1rem' }}>
              <label style={{
                display: 'block',
                fontSize: '0.875rem',
                fontWeight: '500',
                color: colors.text,
                marginBottom: '0.75rem',
              }}>
                Theme
              </label>
              
              <div style={{
                display: 'flex',
                flexDirection: 'column',
                gap: '0.5rem',
              }}>
                {themeOptions.map((option) => (
                  <label
                    key={option.value}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '0.75rem',
                      padding: '0.75rem 1rem',
                      background: theme === option.value ? colors.surfaceHover : colors.surface,
                      border: `2px solid ${theme === option.value ? colors.primary : colors.border}`,
                      borderRadius: '8px',
                      cursor: 'pointer',
                      transition: 'all 0.2s cubic-bezier(0.4, 0, 0.2, 1)',
                      position: 'relative',
                    }}
                    onMouseEnter={(e) => {
                      if (theme !== option.value) {
                        e.currentTarget.style.borderColor = colors.primary;
                        e.currentTarget.style.background = colors.surfaceHover;
                      }
                    }}
                    onMouseLeave={(e) => {
                      if (theme !== option.value) {
                        e.currentTarget.style.borderColor = colors.border;
                        e.currentTarget.style.background = colors.surface;
                      }
                    }}
                  >
                    <input
                      type="radio"
                      name="theme"
                      value={option.value}
                      checked={theme === option.value}
                      onChange={() => setTheme(option.value)}
                      style={{
                        margin: 0,
                        accentColor: colors.primary,
                        width: '16px',
                        height: '16px',
                      }}
                    />
                    
                    <div style={{
                      color: theme === option.value ? colors.primary : colors.textSecondary,
                      transition: 'color 0.2s ease',
                    }}>
                      {option.icon}
                    </div>
                    
                    <div style={{ flex: 1 }}>
                      <div style={{
                        fontSize: '0.875rem',
                        fontWeight: '500',
                        color: colors.text,
                        marginBottom: '0.125rem',
                      }}>
                        {option.label}
                        {option.value === 'system' && (
                          <span style={{
                            marginLeft: '0.5rem',
                            fontSize: '0.75rem',
                            color: colors.textMuted,
                            fontWeight: '400',
                          }}>
                            (Currently: {effectiveTheme})
                          </span>
                        )}
                      </div>
                      <div style={{
                        fontSize: '0.75rem',
                        color: colors.textMuted,
                      }}>
                        {option.description}
                      </div>
                    </div>
                    
                    {theme === option.value && (
                      <div style={{
                        position: 'absolute',
                        top: '0.5rem',
                        right: '0.5rem',
                        width: '8px',
                        height: '8px',
                        background: colors.primary,
                        borderRadius: '50%',
                        boxShadow: `0 0 0 2px ${colors.background}`,
                      }} />
                    )}
                  </label>
                ))}
              </div>
            </div>
          </div>

          {/* Version Info */}
          <div style={{
            background: colors.backgroundSecondary,
            borderRadius: '8px',
            padding: '1rem',
            border: `1px solid ${colors.borderLight}`,
          }}>
            <h4 style={{
              margin: '0 0 0.5rem 0',
              fontSize: '0.875rem',
              fontWeight: '600',
              color: colors.text,
            }}>
              About
            </h4>
            <div style={{
              fontSize: '0.75rem',
              color: colors.textMuted,
              lineHeight: '1.4',
            }}>
              <div>Constellation Studio v0.1.0</div>
              <div>Node-based video processing platform</div>
              <div style={{ marginTop: '0.5rem' }}>
                Built with Rust + Vulkan + React + TypeScript
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
};