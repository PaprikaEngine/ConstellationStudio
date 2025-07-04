import React from 'react';
import { EdgeProps, getBezierPath } from 'reactflow';
import { ConnectionType } from '../types';

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
  const [edgePath] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const getEdgeColor = (connectionType: ConnectionType) => {
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

  const strokeColor = getEdgeColor(data?.connectionType || 'Video');
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