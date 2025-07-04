import { create } from 'zustand';
import { Node, Edge, Connection, addEdge, applyNodeChanges, applyEdgeChanges } from 'reactflow';
import type { NodeProperties, NodeType, ConnectionType } from '../types';

interface NodeStoreState {
  nodes: Node[];
  edges: Edge[];
  nodeProperties: Record<string, NodeProperties>;
  
  // Actions
  addNode: (nodeType: NodeType, position: { x: number; y: number }) => void;
  removeNode: (nodeId: string) => void;
  updateNodeData: (nodeId: string, data: any) => void;
  setNodeProperties: (nodeId: string, properties: NodeProperties) => void;
  
  onNodesChange: (changes: any[]) => void;
  onEdgesChange: (changes: any[]) => void;
  onConnect: (connection: Connection) => void;
  
  // Node creation helpers
  createInputNode: (inputType: string, position: { x: number; y: number }) => void;
  createOutputNode: (outputType: string, position: { x: number; y: number }) => void;
  createEffectNode: (effectType: string, position: { x: number; y: number }) => void;
}

export const useNodeStore = create<NodeStoreState>((set, get) => ({
  nodes: [],
  edges: [],
  nodeProperties: {},

  addNode: (nodeType, position) => {
    const newNode: Node = {
      id: `node-${Date.now()}`,
      type: 'constellation',
      position,
      data: {
        nodeType,
        label: getNodeLabel(nodeType),
        inputTypes: getInputTypes(nodeType),
        outputTypes: getOutputTypes(nodeType),
      },
    };

    set((state) => ({
      nodes: [...state.nodes, newNode],
    }));
  },

  removeNode: (nodeId) => {
    set((state) => ({
      nodes: state.nodes.filter((node) => node.id !== nodeId),
      edges: state.edges.filter((edge) => edge.source !== nodeId && edge.target !== nodeId),
      nodeProperties: Object.fromEntries(
        Object.entries(state.nodeProperties).filter(([id]) => id !== nodeId)
      ),
    }));
  },

  updateNodeData: (nodeId, data) => {
    set((state) => ({
      nodes: state.nodes.map((node) =>
        node.id === nodeId ? { ...node, data: { ...node.data, ...data } } : node
      ),
    }));
  },

  setNodeProperties: (nodeId, properties) => {
    set((state) => ({
      nodeProperties: {
        ...state.nodeProperties,
        [nodeId]: properties,
      },
    }));
  },

  onNodesChange: (changes) => {
    set((state) => ({
      nodes: applyNodeChanges(changes, state.nodes),
    }));
  },

  onEdgesChange: (changes) => {
    set((state) => ({
      edges: applyEdgeChanges(changes, state.edges),
    }));
  },

  onConnect: (connection) => {
    const newEdge: Edge = {
      id: `edge-${connection.source}-${connection.target}`,
      source: connection.source!,
      target: connection.target!,
      sourceHandle: connection.sourceHandle,
      targetHandle: connection.targetHandle,
      type: 'constellation',
      data: {
        connectionType: getConnectionTypeFromHandles(
          connection.sourceHandle,
          connection.targetHandle
        ),
      },
    };

    set((state) => ({
      edges: addEdge(newEdge, state.edges),
    }));
  },

  createInputNode: (inputType, position) => {
    get().addNode({ Input: inputType } as NodeType, position);
  },

  createOutputNode: (outputType, position) => {
    get().addNode({ Output: outputType } as NodeType, position);
  },

  createEffectNode: (effectType, position) => {
    get().addNode({ Effect: effectType } as NodeType, position);
  },
}));

function getNodeLabel(nodeType: NodeType): string {
  if ('Input' in nodeType) return `${nodeType.Input} Input`;
  if ('Output' in nodeType) return `${nodeType.Output} Output`;
  if ('Effect' in nodeType) return `${nodeType.Effect} Effect`;
  if ('Audio' in nodeType) return `${nodeType.Audio} Audio`;
  if ('Tally' in nodeType) return `${nodeType.Tally} Tally`;
  return 'Unknown Node';
}

function getInputTypes(nodeType: NodeType): ConnectionType[] {
  if ('Input' in nodeType) return [];
  if ('Output' in nodeType) return ['Video', 'Audio'];
  if ('Effect' in nodeType) return ['Video'];
  if ('Audio' in nodeType) return ['Audio'];
  if ('Tally' in nodeType) return ['Tally'];
  return [];
}

function getOutputTypes(nodeType: NodeType): ConnectionType[] {
  if ('Input' in nodeType) {
    const inputType = nodeType.Input;
    if (inputType === 'Camera' || inputType === 'VideoFile') return ['Video', 'Audio'];
    return ['Video'];
  }
  if ('Output' in nodeType) return [];
  if ('Effect' in nodeType) return ['Video'];
  if ('Audio' in nodeType) return ['Audio'];
  if ('Tally' in nodeType) return ['Tally'];
  return [];
}

function getConnectionTypeFromHandles(
  sourceHandle: string | null,
  targetHandle: string | null
): ConnectionType {
  if (sourceHandle?.includes('video') || targetHandle?.includes('video')) return 'Video';
  if (sourceHandle?.includes('audio') || targetHandle?.includes('audio')) return 'Audio';
  if (sourceHandle?.includes('tally') || targetHandle?.includes('tally')) return 'Tally';
  return 'Video';
}