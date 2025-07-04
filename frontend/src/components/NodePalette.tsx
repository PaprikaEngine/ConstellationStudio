import React from 'react';
import { Monitor, Mic, Camera, FileVideo, TestTube, Tv, Eye, Palette, Blur, Sparkles, Move, Layers } from 'lucide-react';

interface NodePaletteItem {
  type: string;
  label: string;
  icon: React.ReactNode;
  category: string;
}

const nodeItems: NodePaletteItem[] = [
  // Input Nodes
  { type: 'input-Camera', label: 'Camera', icon: <Camera size={16} />, category: 'Input' },
  { type: 'input-ScreenCapture', label: 'Screen Capture', icon: <Monitor size={16} />, category: 'Input' },
  { type: 'input-WindowCapture', label: 'Window Capture', icon: <Monitor size={16} />, category: 'Input' },
  { type: 'input-VideoFile', label: 'Video File', icon: <FileVideo size={16} />, category: 'Input' },
  { type: 'input-TestPattern', label: 'Test Pattern', icon: <TestTube size={16} />, category: 'Input' },
  
  // Output Nodes
  { type: 'output-VirtualWebcam', label: 'Virtual Webcam', icon: <Tv size={16} />, category: 'Output' },
  { type: 'output-Preview', label: 'Preview', icon: <Eye size={16} />, category: 'Output' },
  
  // Effect Nodes
  { type: 'effect-ColorCorrection', label: 'Color Correction', icon: <Palette size={16} />, category: 'Effects' },
  { type: 'effect-Blur', label: 'Blur', icon: <Blur size={16} />, category: 'Effects' },
  { type: 'effect-Sharpen', label: 'Sharpen', icon: <Sparkles size={16} />, category: 'Effects' },
  { type: 'effect-Transform', label: 'Transform', icon: <Move size={16} />, category: 'Effects' },
  { type: 'effect-Composite', label: 'Composite', icon: <Layers size={16} />, category: 'Effects' },
  
  // Audio Nodes
  { type: 'audio-Input', label: 'Audio Input', icon: <Mic size={16} />, category: 'Audio' },
];

export const NodePalette: React.FC = () => {
  const onDragStart = (event: React.DragEvent, nodeType: string) => {
    event.dataTransfer.setData('application/reactflow', nodeType);
    event.dataTransfer.effectAllowed = 'move';
  };

  const groupedItems = nodeItems.reduce((acc, item) => {
    if (!acc[item.category]) {
      acc[item.category] = [];
    }
    acc[item.category].push(item);
    return acc;
  }, {} as Record<string, NodePaletteItem[]>);

  return (
    <div style={{
      width: '250px',
      background: '#f8f9fa',
      borderRight: '1px solid #dee2e6',
      padding: '16px',
      overflowY: 'auto',
    }}>
      <h3 style={{ margin: '0 0 16px 0', fontSize: '18px', fontWeight: 'bold' }}>
        Node Palette
      </h3>
      
      {Object.entries(groupedItems).map(([category, items]) => (
        <div key={category} style={{ marginBottom: '24px' }}>
          <h4 style={{ 
            margin: '0 0 8px 0', 
            fontSize: '14px', 
            fontWeight: '600',
            color: '#495057',
            textTransform: 'uppercase',
            letterSpacing: '0.5px',
          }}>
            {category}
          </h4>
          
          {items.map((item) => (
            <div
              key={item.type}
              draggable
              onDragStart={(event) => onDragStart(event, item.type)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                padding: '8px 12px',
                margin: '4px 0',
                background: '#ffffff',
                border: '1px solid #dee2e6',
                borderRadius: '4px',
                cursor: 'grab',
                fontSize: '14px',
                transition: 'all 0.2s ease',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = '#e9ecef';
                e.currentTarget.style.transform = 'translateX(4px)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = '#ffffff';
                e.currentTarget.style.transform = 'translateX(0)';
              }}
            >
              {item.icon}
              <span>{item.label}</span>
            </div>
          ))}
        </div>
      ))}
      
      <div style={{
        marginTop: '32px',
        padding: '16px',
        background: '#e3f2fd',
        borderRadius: '8px',
        fontSize: '12px',
        color: '#1565c0',
      }}>
        <strong>How to use:</strong><br />
        Drag nodes from this palette onto the canvas to create your video processing pipeline.
        Connect nodes by dragging from output handles to input handles.
      </div>
    </div>
  );
};