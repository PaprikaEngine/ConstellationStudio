import React, { useCallback } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Edge,
  ReactFlowProvider,
} from 'reactflow';
import 'reactflow/dist/style.css';

import { ConstellationNode } from './ConstellationNode';
import { ConstellationEdge } from './ConstellationEdge';
import { NodePalette } from './NodePalette';
import { useNodeStore } from '../stores/useNodeStore';

const nodeTypes = {
  constellation: ConstellationNode,
};

const edgeTypes = {
  constellation: ConstellationEdge,
};

export const NodeEditor: React.FC = () => {
  const {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    onConnect,
  } = useNodeStore();

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
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
      
      switch (category) {
        case 'input':
          nodeStore.createInputNode(subtype, position);
          break;
        case 'output':
          nodeStore.createOutputNode(subtype, position);
          break;
        case 'effect':
          nodeStore.createEffectNode(subtype, position);
          break;
      }
    },
    []
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
            nodeTypes={nodeTypes}
            edgeTypes={edgeTypes}
            fitView
          >
            <Background />
            <Controls />
            <MiniMap />
          </ReactFlow>
        </ReactFlowProvider>
      </div>
    </div>
  );
};