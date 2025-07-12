import { useEffect, useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { NodeEditor } from './components/NodeEditor';
import { ParameterPanel } from './components/ParameterPanel';
import { NotificationSystem, useNotifications } from './components/NotificationSystem';
import { SettingsPanel } from './components/SettingsPanel';
import { ProjectPanel } from './components/ProjectPanel';
import { PreviewMonitorPanel } from './components/PreviewMonitorPanel';
import { ThemeProvider, useTheme, getThemeStyles } from './contexts/ThemeContext';
import { useNodeStore } from './stores/useNodeStore';
import './App.css';

const queryClient = new QueryClient();

function AppContent() {
  const { isDark } = useTheme();
  const styles = getThemeStyles(isDark);
  const { 
    isConnected, 
    connectionError, 
    engineRunning, 
    fps, 
    frameCount,
    connectToBackend, 
    startEngine,
    stopEngine 
  } = useNodeStore();

  const [isConnecting, setIsConnecting] = useState(false);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [showParameterPanel, setShowParameterPanel] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showProject, setShowProject] = useState(false);
  const [showPreviewMonitor, setShowPreviewMonitor] = useState(false);
  
  // Notification system
  const notifications = useNotifications();

  // Auto-connect on app start
  useEffect(() => {
    const autoConnect = async () => {
      if (!isConnected && !connectionError && !isConnecting) {
        setIsConnecting(true);
        try {
          await connectToBackend();
        } catch (error) {
          console.log('Auto-connect failed, manual connection required');
        } finally {
          setIsConnecting(false);
        }
      }
    };

    autoConnect();
  }, [isConnected, connectionError, connectToBackend]);

  const handleConnect = async () => {
    setIsConnecting(true);
    try {
      await connectToBackend();
      notifications.success('Connected', 'Successfully connected to backend server');
    } catch (error) {
      notifications.error('Connection Failed', 'Failed to connect to backend server');
    } finally {
      setIsConnecting(false);
    }
  };

  const handleEngineToggle = async () => {
    try {
      if (engineRunning) {
        await stopEngine();
        notifications.info('Engine Stopped', 'Video processing engine has been stopped');
      } else {
        await startEngine();
        notifications.success('Engine Started', 'Video processing engine is now running');
      }
    } catch (error) {
      notifications.error('Engine Error', `Failed to ${engineRunning ? 'stop' : 'start'} engine`);
    }
  };

  return (
    <QueryClientProvider client={queryClient}>
      <div className="App">
        <header style={{
          background: styles.headerGradient,
          color: 'white',
          padding: '1rem',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          boxShadow: styles.shadowLight,
          borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
        }}>
          <h1 style={{ 
            margin: 0, 
            fontSize: '1.5rem', 
            fontWeight: '600',
            letterSpacing: '0.5px',
            textShadow: '0 1px 3px rgba(0, 0, 0, 0.3)',
          }}>
            Constellation Studio
          </h1>
          
          <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
            {/* Connection Status */}
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <div style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                background: isConnected ? '#27ae60' : connectionError ? '#e74c3c' : '#f39c12',
                boxShadow: '0 0 6px rgba(255, 255, 255, 0.5)',
                border: '1px solid rgba(255, 255, 255, 0.3)',
              }} />
              <span style={{ fontSize: '0.875rem' }}>
                {isConnecting ? 'Connecting...' : 
                 isConnected ? 'Connected' : 
                 connectionError ? 'Disconnected' : 'Not Connected'}
              </span>
            </div>

            {/* Engine Status */}
            {isConnected && (
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <button
                  onClick={handleEngineToggle}
                  style={{
                    background: engineRunning ? 'linear-gradient(135deg, #e74c3c 0%, #c0392b 100%)' : 'linear-gradient(135deg, #27ae60 0%, #229954 100%)',
                    color: 'white',
                    border: 'none',
                    padding: '0.25rem 0.75rem',
                    borderRadius: '4px',
                    fontSize: '0.75rem',
                    cursor: 'pointer',
                  }}
                >
                  {engineRunning ? 'Stop' : 'Start'} Engine
                </button>
                {engineRunning && (
                  <span style={{ fontSize: '0.75rem', opacity: 0.8 }}>
                    {fps.toFixed(1)} FPS | {frameCount} frames
                  </span>
                )}
              </div>
            )}

            {/* Connect Button */}
            {!isConnected && (
              <button
                onClick={handleConnect}
                disabled={isConnecting}
                style={{
                  background: 'linear-gradient(135deg, #3498db 0%, #2980b9 100%)',
                  color: 'white',
                  border: 'none',
                  padding: '0.5rem 1rem',
                  borderRadius: '4px',
                  fontSize: '0.875rem',
                  cursor: isConnecting ? 'not-allowed' : 'pointer',
                  opacity: isConnecting ? 0.6 : 1,
                }}
              >
                {isConnecting ? 'Connecting...' : 'Connect'}
              </button>
            )}

            <div style={{ display: 'flex', gap: '0.75rem' }}>
              <button
                onClick={() => setShowProject(true)}
                style={{
                  background: 'rgba(255, 255, 255, 0.2)',
                  color: 'white',
                  border: '1px solid rgba(255, 255, 255, 0.3)',
                  borderRadius: '8px',
                  padding: '0.5rem 0.75rem',
                  cursor: 'pointer',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  transition: 'all 0.2s ease',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.5rem',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.3)';
                  e.currentTarget.style.transform = 'translateY(-1px)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.2)';
                  e.currentTarget.style.transform = 'translateY(0)';
                }}
                title="Project Manager"
              >
                📁 Project
              </button>
              
              <button
                onClick={() => setShowPreviewMonitor(true)}
                style={{
                  background: 'rgba(255, 255, 255, 0.2)',
                  color: 'white',
                  border: '1px solid rgba(255, 255, 255, 0.3)',
                  borderRadius: '8px',
                  padding: '0.5rem 0.75rem',
                  cursor: 'pointer',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  transition: 'all 0.2s ease',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.5rem',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.3)';
                  e.currentTarget.style.transform = 'translateY(-1px)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.2)';
                  e.currentTarget.style.transform = 'translateY(0)';
                }}
                title="Preview & Monitor"
              >
                📹 Preview
              </button>
              
              <button
                onClick={() => setShowSettings(true)}
                style={{
                  background: 'rgba(255, 255, 255, 0.2)',
                  color: 'white',
                  border: '1px solid rgba(255, 255, 255, 0.3)',
                  borderRadius: '8px',
                  padding: '0.5rem 0.75rem',
                  cursor: 'pointer',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  transition: 'all 0.2s ease',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.5rem',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.3)';
                  e.currentTarget.style.transform = 'translateY(-1px)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.2)';
                  e.currentTarget.style.transform = 'translateY(0)';
                }}
                title="Open Settings"
              >
                ⚙️ Settings
              </button>
            </div>

            <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
              <span style={{ 
                background: 'linear-gradient(135deg, #3498db 0%, #2980b9 100%)', 
                padding: '0.3rem 0.8rem', 
                borderRadius: '16px',
                fontSize: '0.75rem',
                fontWeight: '500',
                boxShadow: '0 2px 6px rgba(52, 152, 219, 0.3)',
                border: '1px solid rgba(255, 255, 255, 0.2)',
              }}>
                Phase 1b: Core Features
              </span>
              <span style={{ 
                background: 'linear-gradient(135deg, #27ae60 0%, #229954 100%)', 
                padding: '0.3rem 0.8rem', 
                borderRadius: '16px',
                fontSize: '0.75rem',
                fontWeight: '500',
                boxShadow: '0 2px 6px rgba(39, 174, 96, 0.3)',
                border: '1px solid rgba(255, 255, 255, 0.2)',
              }}>
                Dev Mode
              </span>
            </div>
          </div>
        </header>
        
        <main style={{ height: 'calc(100vh - 80px)', display: 'flex' }}>
          <div style={{ flex: 1 }}>
            <NodeEditor onNodeSelect={setSelectedNodeId} />
          </div>
          
          {showParameterPanel && (
            <ParameterPanel 
              selectedNodeId={selectedNodeId}
              onClose={() => {
                setShowParameterPanel(false);
                setSelectedNodeId(null);
              }}
            />
          )}
        </main>
        
        {/* Parameter panel toggle button */}
        <button
          onClick={() => {
            // Only toggle if we have a valid selected node
            if (selectedNodeId) {
              setShowParameterPanel(!showParameterPanel);
            } else {
              // Show info that no node is selected
              notifications.info('No Node Selected', 'Please select a node first to view its parameters');
            }
          }}
          style={{
            position: 'fixed',
            bottom: '1rem',
            right: '1rem',
            background: styles.buttonPrimary,
            color: 'white',
            border: 'none',
            borderRadius: '50%',
            width: '56px',
            height: '56px',
            cursor: 'pointer',
            boxShadow: '0 6px 20px rgba(102, 126, 234, 0.4)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '1.25rem',
            transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
            transform: 'scale(1)',
          }}
          title="Toggle Parameter Panel"
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = 'scale(1.1)';
            e.currentTarget.style.boxShadow = '0 8px 25px rgba(102, 126, 234, 0.5)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = 'scale(1)';
            e.currentTarget.style.boxShadow = '0 6px 20px rgba(102, 126, 234, 0.4)';
          }}
        >
          🎛️
        </button>
        
        {/* Project Panel */}
        <ProjectPanel 
          isOpen={showProject}
          onClose={() => setShowProject(false)}
        />
        
        {/* Preview & Monitor Panel */}
        <PreviewMonitorPanel 
          isOpen={showPreviewMonitor}
          onClose={() => setShowPreviewMonitor(false)}
        />
        
        {/* Settings Panel */}
        <SettingsPanel 
          isOpen={showSettings}
          onClose={() => setShowSettings(false)}
        />
        
        {/* Notification System */}
        <NotificationSystem 
          notifications={notifications.notifications}
          onDismiss={notifications.removeNotification}
        />
      </div>
    </QueryClientProvider>
  );
}

function App() {
  return (
    <ThemeProvider>
      <AppContent />
    </ThemeProvider>
  );
}

export default App;