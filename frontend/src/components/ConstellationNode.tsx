import React from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { ConnectionType } from '../types';

interface ConstellationNodeData {
  nodeType: any;
  label: string;
  inputTypes: ConnectionType[];
  outputTypes: ConnectionType[];
}

export const ConstellationNode: React.FC<NodeProps<ConstellationNodeData>> = ({
  data,
  selected,
}) => {
  const getHandleColor = (connectionType: ConnectionType) => {
    switch (connectionType) {
      case 'Video':
        return '#ff6b6b';
      case 'Audio':
        return '#4ecdc4';
      case 'Tally':
        return '#ffe66d';
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
          top: `${30 + index * 20}px`,
          width: 12,
          height: 12,
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
          top: `${30 + index * 20}px`,
          width: 12,
          height: 12,
        }}
      />
    ));
  };

  const nodeHeight = Math.max(60, 30 + Math.max(data.inputTypes.length, data.outputTypes.length) * 20);

  return (
    <div
      style={{
        background: selected ? '#e3f2fd' : '#f5f5f5',
        border: selected ? '2px solid #2196f3' : '1px solid #ccc',
        borderRadius: '8px',
        padding: '10px',
        minWidth: '150px',
        height: `${nodeHeight}px`,
        position: 'relative',
        boxShadow: selected ? '0 4px 8px rgba(0,0,0,0.2)' : '0 2px 4px rgba(0,0,0,0.1)',
      }}
    >
      {renderInputHandles()}
      
      <div style={{ 
        fontSize: '14px', 
        fontWeight: 'bold', 
        textAlign: 'center',
        marginBottom: '5px',
      }}>
        {data.label}
      </div>
      
      <div style={{ fontSize: '12px', color: '#666' }}>
        {data.inputTypes.length > 0 && (
          <div>In: {data.inputTypes.join(', ')}</div>
        )}
        {data.outputTypes.length > 0 && (
          <div>Out: {data.outputTypes.join(', ')}</div>
        )}
      </div>
      
      {renderOutputHandles()}
    </div>
  );
};