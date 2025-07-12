import React from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { ConnectionType } from '../types';
import { useTheme, getThemeColors, getThemeStyles } from '../contexts/ThemeContext';
import AudioLevelMeter from './AudioLevelMeter';

interface ConstellationNodeData {
  nodeType: any;
  label: string;
  inputTypes: ConnectionType[];
  outputTypes: ConnectionType[];
}

export const ConstellationNode: React.FC<NodeProps<ConstellationNodeData>> = ({
  id,
  data,
  selected,
}) => {
  const { isDark } = useTheme();
  const colors = getThemeColors(isDark);
  const styles = getThemeStyles(isDark);
  const getHandleColor = (connectionType: ConnectionType) => {
    switch (connectionType) {
      case 'RenderData':
        return '#ff6b6b';
      case 'Audio':
        return '#4ecdc4';
      case 'Control':
        return '#9b59b6';
      case 'Tally':
        return '#f39c12';
      default:
        return '#95a5a6';
    }
  };

  const renderInputHandles = () => {
    return data.inputTypes.map((type, index) => (
      <Handle
        key={`input-${type}-${index}`}
        type="target"
        position={Position.Left}
        id={`input-${type.toLowerCase()}-${index}`}
        style={{
          background: getHandleColor(type),
          top: `${35 + index * 22}px`,
          width: 14,
          height: 14,
          border: '2px solid #ffffff',
          boxShadow: '0 2px 6px rgba(0, 0, 0, 0.2)',
        }}
      />
    ));
  };

  const renderOutputHandles = () => {
    return data.outputTypes.map((type, index) => (
      <Handle
        key={`output-${type}-${index}`}
        type="source"
        position={Position.Right}
        id={`output-${type.toLowerCase()}-${index}`}
        style={{
          background: getHandleColor(type),
          top: `${35 + index * 22}px`,
          width: 14,
          height: 14,
          border: '2px solid #ffffff',
          boxShadow: '0 2px 6px rgba(0, 0, 0, 0.2)',
        }}
      />
    ));
  };

  // Check if this is an audio node
  const isAudioNode = data.outputTypes.includes('Audio') || 
                     (data.nodeType && typeof data.nodeType === 'object' && 'Audio' in data.nodeType);
  
  const nodeHeight = Math.max(
    isAudioNode ? 110 : 70, // Extra height for audio meter
    35 + Math.max(data.inputTypes.length, data.outputTypes.length) * 22
  );

  return (
    <div
      style={{
        background: selected ? colors.nodeBackgroundSelected : colors.nodeBackground,
        border: selected ? `2px solid ${colors.nodeBorderSelected}` : `1px solid ${colors.nodeBorder}`,
        borderRadius: '10px',
        padding: '12px',
        minWidth: '160px',
        height: `${nodeHeight}px`,
        position: 'relative',
        boxShadow: selected ? styles.shadowHeavy : styles.shadowLight,
        transition: 'all 0.2s cubic-bezier(0.4, 0, 0.2, 1)',
      }}
    >
      {renderInputHandles()}
      
      <div style={{ 
        fontSize: '14px', 
        fontWeight: '600', 
        textAlign: 'center',
        marginBottom: '8px',
        color: selected ? (isDark ? '#ffffff' : '#1565c0') : colors.text,
        letterSpacing: '0.3px',
      }}>
        {data.label}
      </div>
      
      <div style={{ 
        fontSize: '11px', 
        color: selected ? (isDark ? '#cbd5e0' : '#5e72e4') : colors.textSecondary,
        lineHeight: '1.4',
        fontWeight: '500',
      }}>
        {data.inputTypes.length > 0 && (
          <div>In: {data.inputTypes.join(', ')}</div>
        )}
        {data.outputTypes.length > 0 && (
          <div>Out: {data.outputTypes.join(', ')}</div>
        )}
      </div>
      
      {/* Audio Level Meter for Audio Nodes */}
      {isAudioNode && (
        <div style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          marginTop: '8px',
          padding: '4px',
          borderRadius: '4px',
          backgroundColor: 'rgba(0, 0, 0, 0.1)',
        }}>
          <AudioLevelMeter
            nodeId={id}
            width={30}
            height={35}
            mode="mono"
            showLabels={false}
            showValues={true}
            className="node-audio-meter"
          />
        </div>
      )}
      
      {renderOutputHandles()}
    </div>
  );
};