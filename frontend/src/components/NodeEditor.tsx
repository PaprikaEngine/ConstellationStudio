import React, { useCallback } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  ReactFlowProvider,
} from 'reactflow';
import 'reactflow/dist/style.css';

import { ConstellationNode } from './ConstellationNode';
import { ConstellationEdge } from './ConstellationEdge';
import { NodePalette } from './NodePalette';
import { useNodeStore } from '../stores/useNodeStore';
import { useTheme, getThemeColors } from '../contexts/ThemeContext';

const nodeTypes = {
  constellation: ConstellationNode,
};

const edgeTypes = {
  constellation: ConstellationEdge,
};

interface NodeEditorProps {
  onNodeSelect?: (nodeId: string | null) => void;
}

export const NodeEditor: React.FC<NodeEditorProps> = ({ onNodeSelect }) => {
  const {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    onConnect,
    setViewport,
  } = useNodeStore();
  
  const { isDark } = useTheme();
  const colors = getThemeColors(isDark);

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback(
    async (event: React.DragEvent) => {
      event.preventDefault();

      const reactFlowBounds = (event.target as Element)
        .closest('.react-flow')
        ?.getBoundingClientRect();

      if (!reactFlowBounds) return;

      const type = event.dataTransfer.getData('application/reactflow');
      if (!type) return;

      const position = {
        x: event.clientX - reactFlowBounds.left,
        y: event.clientY - reactFlowBounds.top,
      };

      // Parse the node type and create appropriate node
      const [category, subtype] = type.split('-');
      const nodeStore = useNodeStore.getState();
      
      try {
        switch (category) {
          case 'input':
            await nodeStore.createInputNode(subtype, position);
            break;
          case 'output':
            await nodeStore.createOutputNode(subtype, position);
            break;
          case 'effect':
            await nodeStore.createEffectNode(subtype, position);
            break;
          case 'audio':
            await nodeStore.createAudioNode(subtype, position);
            break;
          case 'control':
            await nodeStore.createControlNode(subtype, position);
            break;
          case 'tally':
            await nodeStore.createTallyNode(subtype, position);
            break;
          default:
            console.warn('Unknown node category:', category);
        }
      } catch (error) {
        console.error('Failed to create node:', error);
        // TODO: Show user-friendly error message
      }
    },
    []
  );

  const onSelectionChange = useCallback(
    ({ nodes }: { nodes: any[] }) => {
      const selectedNodeId = nodes.length > 0 ? nodes[0].id : null;
      onNodeSelect?.(selectedNodeId);
    },
    [onNodeSelect]
  );

  return (
    <div style={{ height: '100vh', display: 'flex' }}>
      <NodePalette />
      <div style={{ flex: 1 }}>
        <ReactFlowProvider>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onDrop={onDrop}
            onDragOver={onDragOver}
            onSelectionChange={onSelectionChange}
            nodeTypes={nodeTypes}
            edgeTypes={edgeTypes}
            onViewportChange={(viewport) => setViewport(viewport)}
            fitView
          >
            <Background color={colors.canvasDot} gap={20} size={2} />
            <Controls />
            <MiniMap />
          </ReactFlow>
        </ReactFlowProvider>
      </div>
    </div>
  );
};