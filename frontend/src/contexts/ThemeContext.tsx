import React, { createContext, useContext, useState, useEffect } from 'react';

export type Theme = 'light' | 'dark' | 'system';

interface ThemeContextType {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  isDark: boolean;
  effectiveTheme: 'light' | 'dark';
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

interface ThemeProviderProps {
  children: React.ReactNode;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  const [theme, setThemeState] = useState<Theme>(() => {
    // Load theme from localStorage or default to system
    const savedTheme = localStorage.getItem('constellation-theme') as Theme;
    return savedTheme || 'system';
  });

  const [systemDark, setSystemDark] = useState(() => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  });

  // Determine effective theme
  const effectiveTheme: 'light' | 'dark' = theme === 'system' ? (systemDark ? 'dark' : 'light') : theme;
  const isDark = effectiveTheme === 'dark';

  const setTheme = (newTheme: Theme) => {
    setThemeState(newTheme);
    localStorage.setItem('constellation-theme', newTheme);
  };

  useEffect(() => {
    // Listen for system theme changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (e: MediaQueryListEvent) => {
      setSystemDark(e.matches);
    };

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, []);

  useEffect(() => {
    // Apply effective theme to document body
    document.body.setAttribute('data-theme', effectiveTheme);
    document.body.className = `theme-${effectiveTheme}`;
  }, [effectiveTheme]);

  return (
    <ThemeContext.Provider value={{ theme, setTheme, isDark, effectiveTheme }}>
      {children}
    </ThemeContext.Provider>
  );
};

// Theme colors and styles
export const getThemeColors = (isDark: boolean) => ({
  // Primary colors
  primary: isDark ? '#667eea' : '#667eea',
  primaryDark: isDark ? '#5a67d8' : '#764ba2',
  
  // Background colors
  background: isDark ? '#1a202c' : '#ffffff',
  backgroundSecondary: isDark ? '#2d3748' : '#f8f9fa',
  backgroundTertiary: isDark ? '#4a5568' : '#e9ecef',
  
  // Surface colors
  surface: isDark ? '#2d3748' : '#ffffff',
  surfaceHover: isDark ? '#4a5568' : '#f8f9fa',
  
  // Text colors
  text: isDark ? '#f7fafc' : '#2c3e50',
  textSecondary: isDark ? '#cbd5e0' : '#6c757d',
  textMuted: isDark ? '#a0aec0' : '#8e9aaf',
  
  // Border colors
  border: isDark ? '#4a5568' : '#d1d9e0',
  borderLight: isDark ? '#2d3748' : '#e0e6ed',
  
  // Node colors
  nodeBackground: isDark ? '#2d3748' : '#ffffff',
  nodeBackgroundSelected: isDark ? 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)' : 'linear-gradient(135deg, #e3f2fd 0%, #f3e5f5 100%)',
  nodeBorder: isDark ? '#4a5568' : '#d1d9e0',
  nodeBorderSelected: isDark ? '#667eea' : '#2196f3',
  
  // Canvas colors
  canvasBackground: isDark ? 'linear-gradient(135deg, #1a202c 0%, #2d3748 100%)' : 'linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%)',
  canvasDot: isDark ? '#4a5568' : '#d1d5db',
  
  // Notification colors
  success: isDark ? '#48bb78' : '#28a745',
  error: isDark ? '#f56565' : '#dc3545',
  warning: isDark ? '#ed8936' : '#ffc107',
  info: isDark ? '#4299e1' : '#17a2b8',
  
  // Status colors
  connected: '#48bb78',
  disconnected: '#f56565',
  connecting: '#ed8936',
});

export const getThemeStyles = (isDark: boolean) => {
  const colors = getThemeColors(isDark);
  
  return {
    // Header gradient
    headerGradient: isDark 
      ? 'linear-gradient(135deg, #2d3748 0%, #4a5568 100%)'
      : 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    
    // Button gradients
    buttonPrimary: isDark
      ? 'linear-gradient(135deg, #667eea 0%, #5a67d8 100%)'
      : 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    
    buttonSuccess: isDark
      ? 'linear-gradient(135deg, #48bb78 0%, #38a169 100%)'
      : 'linear-gradient(135deg, #28a745 0%, #20c997 100%)',
    
    buttonDanger: isDark
      ? 'linear-gradient(135deg, #f56565 0%, #e53e3e 100%)'
      : 'linear-gradient(135deg, #e74c3c 0%, #c0392b 100%)',
    
    // Shadow styles
    shadowLight: isDark
      ? '0 2px 8px rgba(0, 0, 0, 0.3)'
      : '0 2px 8px rgba(0, 0, 0, 0.1)',
    
    shadowMedium: isDark
      ? '0 4px 16px rgba(0, 0, 0, 0.4)'
      : '0 4px 16px rgba(0, 0, 0, 0.15)',
    
    shadowHeavy: isDark
      ? '0 8px 32px rgba(0, 0, 0, 0.5)'
      : '0 8px 24px rgba(33, 150, 243, 0.3)',
  };
};