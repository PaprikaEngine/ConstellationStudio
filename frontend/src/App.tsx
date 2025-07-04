import React from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { NodeEditor } from './components/NodeEditor';
import './App.css';

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <div className="App">
        <header style={{
          background: '#2c3e50',
          color: 'white',
          padding: '1rem',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}>
          <h1 style={{ margin: 0, fontSize: '1.5rem' }}>
            Constellation Studio
          </h1>
          <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
            <span style={{ 
              background: '#27ae60', 
              padding: '0.25rem 0.75rem', 
              borderRadius: '12px',
              fontSize: '0.875rem',
            }}>
              Phase 1 - Local 2D Processing
            </span>
            <div style={{ fontSize: '0.875rem', opacity: 0.8 }}>
              Rust + Ash Vulkan Backend
            </div>
          </div>
        </header>
        
        <main style={{ height: 'calc(100vh - 80px)' }}>
          <NodeEditor />
        </main>
        
        <div style={{
          position: 'fixed',
          bottom: '1rem',
          right: '1rem',
          background: 'rgba(0,0,0,0.8)',
          color: 'white',
          padding: '0.5rem 1rem',
          borderRadius: '8px',
          fontSize: '0.75rem',
        }}>
          Real-time Video Processing Engine Ready
        </div>
      </div>
    </QueryClientProvider>
  );
}

export default App;