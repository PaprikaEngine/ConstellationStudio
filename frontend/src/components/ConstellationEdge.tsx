import React from 'react';
import { EdgeProps, getBezierPath } from 'reactflow';
import { ConnectionType } from '../types';
import { useTheme } from '../contexts/ThemeContext';

interface ConstellationEdgeData {
  connectionType: ConnectionType;
}

export const ConstellationEdge: React.FC<EdgeProps<ConstellationEdgeData>> = ({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  data,
  selected,
}) => {
  const { isDark } = useTheme();
  const [edgePath] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const getEdgeColor = (connectionType: ConnectionType) => {
    const baseColors = {
      'RenderData': '#ff6b6b',
      'Audio': '#4ecdc4', 
      'Control': '#9b59b6',
      'Tally': '#f39c12',
    };
    
    const color = baseColors[connectionType] || '#95a5a6';
    
    // Slightly adjust colors for dark mode
    if (isDark) {
      switch (connectionType) {
        case 'RenderData':
          return '#ff7979';
        case 'Audio':
          return '#55efc4';
        case 'Control':
          return '#a29bfe';
        case 'Tally':
          return '#fdcb6e';
        default:
          return '#b2bec3';
      }
    }
    
    return color;
  };

  const strokeColor = getEdgeColor(data?.connectionType || 'RenderData');
  const strokeWidth = selected ? 3 : 2;

  return (
    <>
      <path
        id={id}
        style={{
          stroke: strokeColor,
          strokeWidth,
          fill: 'none',
        }}
        className="react-flow__edge-path"
        d={edgePath}
      />
      {data?.connectionType && (
        <text>
          <textPath
            href={`#${id}`}
            style={{
              fontSize: '12px',
              fill: strokeColor,
              fontWeight: 'bold',
            }}
            startOffset="50%"
            textAnchor="middle"
          >
            {data.connectionType}
          </textPath>
        </text>
      )}
    </>
  );
};