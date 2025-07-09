import React from 'react';
import { Monitor, Mic, Camera, FileVideo, TestTube, Tv, Eye, Palette, Sparkles, Move, Layers, Settings, Play, Gamepad2, Wifi, Zap, Radio, Activity, GitBranch, Shuffle } from 'lucide-react';
import { useTheme, getThemeColors, getThemeStyles } from '../contexts/ThemeContext';

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
  { type: 'output-Viewer', label: 'Viewer', icon: <Monitor size={16} />, category: 'Output' },
  
  // Effect Nodes
  { type: 'effect-ColorCorrection', label: 'Color Correction', icon: <Palette size={16} />, category: 'Effects' },
  { type: 'effect-Blur', label: 'Blur', icon: <Zap size={16} />, category: 'Effects' },
  { type: 'effect-Sharpen', label: 'Sharpen', icon: <Sparkles size={16} />, category: 'Effects' },
  { type: 'effect-Transform', label: 'Transform', icon: <Move size={16} />, category: 'Effects' },
  { type: 'effect-Composite', label: 'Composite', icon: <Layers size={16} />, category: 'Effects' },
  
  // Audio Nodes
  { type: 'audio-Input', label: 'Audio Input', icon: <Mic size={16} />, category: 'Audio' },
  { type: 'audio-Mixer', label: 'Audio Mixer', icon: <Settings size={16} />, category: 'Audio' },
  { type: 'audio-Effect', label: 'Audio Effect', icon: <Sparkles size={16} />, category: 'Audio' },
  { type: 'audio-Output', label: 'Audio Output', icon: <Mic size={16} />, category: 'Audio' },
  
  // Control Nodes
  { type: 'control-ParameterController', label: 'Parameter Controller', icon: <Settings size={16} />, category: 'Control' },
  { type: 'control-AnimationController', label: 'Animation Controller', icon: <Play size={16} />, category: 'Control' },
  { type: 'control-MidiController', label: 'MIDI Controller', icon: <Gamepad2 size={16} />, category: 'Control' },
  { type: 'control-OscController', label: 'OSC Controller', icon: <Wifi size={16} />, category: 'Control' },
  
  // Tally Nodes
  { type: 'tally-Generator', label: 'Tally Generator', icon: <Radio size={16} />, category: 'Tally' },
  { type: 'tally-Monitor', label: 'Tally Monitor', icon: <Activity size={16} />, category: 'Tally' },
  { type: 'tally-Logic', label: 'Tally Logic', icon: <GitBranch size={16} />, category: 'Tally' },
  { type: 'tally-Router', label: 'Tally Router', icon: <Shuffle size={16} />, category: 'Tally' },
];

export const NodePalette: React.FC = () => {
  const { isDark } = useTheme();
  const colors = getThemeColors(isDark);
  const styles = getThemeStyles(isDark);

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
      background: colors.background,
      borderRight: `2px solid ${colors.borderLight}`,
      padding: '16px',
      overflowY: 'auto',
      height: '100vh',
      boxShadow: styles.shadowLight,
    }}>
      <h3 style={{ 
        margin: '0 0 16px 0', 
        fontSize: '18px', 
        fontWeight: 'bold',
        color: colors.text,
        borderBottom: `2px solid ${isDark ? colors.border : '#e3f2fd'}`,
        paddingBottom: '8px',
      }}>
        Node Palette
      </h3>
      
      {Object.entries(groupedItems).map(([category, items]) => (
        <div key={category} style={{ marginBottom: '24px' }}>
          <h4 style={{ 
            margin: '0 0 12px 0', 
            fontSize: '13px', 
            fontWeight: '700',
            color: colors.text,
            textTransform: 'uppercase',
            letterSpacing: '0.8px',
            background: colors.backgroundSecondary,
            padding: '6px 12px',
            borderRadius: '4px',
            border: `1px solid ${colors.borderLight}`,
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
                gap: '10px',
                padding: '10px 14px',
                margin: '6px 0',
                background: colors.surface,
                border: `1px solid ${colors.border}`,
                borderRadius: '6px',
                cursor: 'grab',
                fontSize: '14px',
                fontWeight: '500',
                color: colors.text,
                transition: 'all 0.2s cubic-bezier(0.4, 0, 0.2, 1)',
                boxShadow: styles.shadowLight,
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = colors.surfaceHover;
                e.currentTarget.style.transform = 'translateX(4px)';
                e.currentTarget.style.borderColor = colors.primary;
                e.currentTarget.style.boxShadow = styles.shadowMedium;
                e.currentTarget.style.color = isDark ? colors.primary : '#1565c0';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = colors.surface;
                e.currentTarget.style.transform = 'translateX(0)';
                e.currentTarget.style.borderColor = colors.border;
                e.currentTarget.style.boxShadow = styles.shadowLight;
                e.currentTarget.style.color = colors.text;
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
        background: isDark 
          ? 'linear-gradient(135deg, #2d3748 0%, #4a5568 100%)'
          : 'linear-gradient(135deg, #e3f2fd 0%, #f3e5f5 100%)',
        borderRadius: '8px',
        fontSize: '12px',
        color: isDark ? colors.primary : '#1565c0',
        border: `1px solid ${isDark ? colors.border : '#bbdefb'}`,
        boxShadow: styles.shadowLight,
      }}>
        <strong>How to use:</strong><br />
        Drag nodes from this palette onto the canvas to create your video processing pipeline.
        Connect nodes by dragging from output handles to input handles.
      </div>
    </div>
  );
};