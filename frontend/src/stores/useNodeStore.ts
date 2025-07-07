import { create } from 'zustand';
import { Node, Edge, Connection, addEdge, applyNodeChanges, applyEdgeChanges, Viewport } from 'reactflow';
import type { NodeProperties, NodeType, ConnectionType } from '../types';
import { apiClient } from '../api';

interface NodeStoreState {
  nodes: Node[];
  edges: Edge[];
  nodeProperties: Record<string, NodeProperties>;
  viewport: Viewport | undefined;
  
  // Connection state
  isConnected: boolean;
  connectionError: string | null;
  
  // Engine state
  engineRunning: boolean;
  fps: number;
  frameCount: number;
  
  // Actions
  addNode: (nodeType: NodeType, position: { x: number; y: number }) => Promise<void>;
  removeNode: (nodeId: string) => Promise<void>;
  updateNodeData: (nodeId: string, data: any) => void;
  setNodeProperties: (nodeId: string, properties: NodeProperties) => void;
  setNodeParameters: (nodeId: string, parameters: Record<string, any>) => Promise<void>;
  
  onNodesChange: (changes: any[]) => void;
  onEdgesChange: (changes: any[]) => void;
  onConnect: (connection: Connection) => Promise<void>;
  
  // Node creation helpers
  createInputNode: (inputType: string, position: { x: number; y: number }) => Promise<void>;
  createOutputNode: (outputType: string, position: { x: number; y: number }) => Promise<void>;
  createEffectNode: (effectType: string, position: { x: number; y: number }) => Promise<void>;
  createAudioNode: (audioType: string, position: { x: number; y: number }) => Promise<void>;
  createControlNode: (controlType: string, position: { x: number; y: number }) => Promise<void>;
  createTallyNode: (tallyType: string, position: { x: number; y: number }) => Promise<void>;
  
  // API Integration
  connectToBackend: () => Promise<void>;
  disconnectFromBackend: () => void;
  startEngine: () => Promise<void>;
  stopEngine: () => Promise<void>;
  refreshEngineStatus: () => Promise<void>;
  
  // WebSocket event handling
  handleEngineEvent: (event: any) => void;
  
  // Project management
  setNodesAndEdges: (nodes: Node[], edges: Edge[]) => void;
  setViewport: (viewport: Viewport) => void;
}

export const useNodeStore = create<NodeStoreState>((set, get) => ({
  nodes: [],
  edges: [],
  nodeProperties: {},
  viewport: undefined,
  
  // Connection state
  isConnected: false,
  connectionError: null,
  
  // Engine state
  engineRunning: false,
  fps: 0,
  frameCount: 0,

  addNode: async (nodeType, position) => {
    try {
      // Create node via API
      const nodeId = await apiClient.createNode(nodeType, { parameters: {} });
      
      const newNode: Node = {
        id: nodeId,
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
      
      console.log(`✅ Node created: ${nodeId}`);
    } catch (error) {
      console.error('❌ Failed to create node:', error);
      throw error;
    }
  },

  removeNode: async (nodeId) => {
    try {
      // Delete node via API
      await apiClient.deleteNode(nodeId);
      
      set((state) => ({
        nodes: state.nodes.filter((node) => node.id !== nodeId),
        edges: state.edges.filter((edge) => edge.source !== nodeId && edge.target !== nodeId),
        nodeProperties: Object.fromEntries(
          Object.entries(state.nodeProperties).filter(([id]) => id !== nodeId)
        ),
      }));
      
      console.log(`✅ Node deleted: ${nodeId}`);
    } catch (error) {
      console.error('❌ Failed to delete node:', error);
      throw error;
    }
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

  setNodeParameters: async (nodeId, parameters) => {
    try {
      await apiClient.setNodeParameters(nodeId, parameters);
      console.log(`✅ Parameters set for node: ${nodeId}`, parameters);
    } catch (error) {
      console.error('❌ Failed to set node parameters:', error);
      throw error;
    }
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

  onConnect: async (connection) => {
    try {
      const connectionType = getConnectionTypeFromHandles(
        connection.sourceHandle,
        connection.targetHandle
      );
      
      // Create connection via API
      await apiClient.createConnection(
        connection.source!,
        connection.target!,
        connectionType
      );

      const newEdge: Edge = {
        id: `edge-${connection.source}-${connection.target}`,
        source: connection.source!,
        target: connection.target!,
        sourceHandle: connection.sourceHandle,
        targetHandle: connection.targetHandle,
        type: 'constellation',
        data: {
          connectionType,
        },
      };

      set((state) => ({
        edges: addEdge(newEdge, state.edges),
      }));
      
      console.log(`✅ Connection created: ${connection.source} -> ${connection.target} (${connectionType})`);
    } catch (error) {
      console.error('❌ Failed to create connection:', error);
      throw error;
    }
  },

  createInputNode: async (inputType, position) => {
    await get().addNode({ Input: inputType } as NodeType, position);
  },

  createOutputNode: async (outputType, position) => {
    await get().addNode({ Output: outputType } as NodeType, position);
  },

  createEffectNode: async (effectType, position) => {
    await get().addNode({ Effect: effectType } as NodeType, position);
  },

  createAudioNode: async (audioType, position) => {
    await get().addNode({ Audio: audioType } as NodeType, position);
  },

  createControlNode: async (controlType, position) => {
    await get().addNode({ Control: controlType } as NodeType, position);
  },

  createTallyNode: async (tallyType, position) => {
    await get().addNode({ Tally: tallyType } as NodeType, position);
  },

  // API Integration
  connectToBackend: async () => {
    try {
      // Test connection with health check
      const isHealthy = await apiClient.healthCheck();
      if (!isHealthy) {
        throw new Error('Backend health check failed');
      }

      // Connect WebSocket
      await apiClient.connectWebSocket();

      // Set up event listener
      apiClient.addEventListener((event) => {
        get().handleEngineEvent(event);
      });

      set(() => ({
        isConnected: true,
        connectionError: null,
      }));

      // Initial status refresh
      await get().refreshEngineStatus();
      
      console.log('✅ Connected to backend');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      set(() => ({
        isConnected: false,
        connectionError: errorMessage,
      }));
      console.error('❌ Failed to connect to backend:', error);
      throw error;
    }
  },

  disconnectFromBackend: () => {
    apiClient.disconnectWebSocket();
    set(() => ({
      isConnected: false,
      connectionError: null,
    }));
    console.log('🔌 Disconnected from backend');
  },

  startEngine: async () => {
    try {
      await apiClient.startEngine();
      await get().refreshEngineStatus();
      console.log('✅ Engine started');
    } catch (error) {
      console.error('❌ Failed to start engine:', error);
      throw error;
    }
  },

  stopEngine: async () => {
    try {
      await apiClient.stopEngine();
      await get().refreshEngineStatus();
      console.log('✅ Engine stopped');
    } catch (error) {
      console.error('❌ Failed to stop engine:', error);
      throw error;
    }
  },

  refreshEngineStatus: async () => {
    try {
      const status = await apiClient.getEngineStatus();
      set(() => ({
        engineRunning: status.running,
        fps: status.fps,
        frameCount: status.frame_count,
      }));
    } catch (error) {
      console.error('❌ Failed to refresh engine status:', error);
    }
  },

  // Project management
  setNodesAndEdges: (newNodes, newEdges) => {
    set(() => ({
      nodes: newNodes,
      edges: newEdges,
    }));
  },

  setViewport: (newViewport) => {
    set(() => ({
      viewport: newViewport,
    }));
  },
  
  // WebSocket event handling
  handleEngineEvent: (event) => {
    console.log('📨 Engine event:', event);
    
    switch (event.type || Object.keys(event)[0]) {
      case 'NodeAdded':
        // Handle node added event
        // Note: Node is already added via API, so we might just need to sync
        break;
      case 'NodeRemoved':
        // Handle node removed event
        break;
      case 'NodeConnected':
        // Handle connection created event
        break;
      case 'ParameterChanged':
        // Handle parameter change event
        break;
      case 'FrameProcessed':
        if (event.FrameProcessed) {
          set((state) => ({
            frameCount: state.frameCount + 1,
          }));
        }
        break;
      case 'EngineStarted':
        set(() => ({ engineRunning: true }));
        break;
      case 'EngineStopped':
        set(() => ({ engineRunning: false }));
        break;
      case 'Error':
        console.error('🚨 Engine error:', event.Error?.message);
        break;
      default:
        console.log('🔍 Unknown event type:', event);
    }
  },
}));

function getNodeLabel(nodeType: NodeType): string {
  if ('Input' in nodeType) return `${nodeType.Input} Input`;
  if ('Output' in nodeType) return `${nodeType.Output} Output`;
  if ('Effect' in nodeType) return `${nodeType.Effect} Effect`;
  if ('Audio' in nodeType) return `${nodeType.Audio} Audio`;
  if ('Control' in nodeType) return `${nodeType.Control} Control`;
  if ('Tally' in nodeType) return `${nodeType.Tally} Tally`;
  return 'Unknown Node';
}

function getInputTypes(nodeType: NodeType): ConnectionType[] {
  if ('Input' in nodeType) return [];
  if ('Output' in nodeType) return ['RenderData', 'Audio'];
  if ('Effect' in nodeType) return ['RenderData'];
  if ('Audio' in nodeType) return ['Audio'];
  if ('Control' in nodeType) return [];
  if ('Tally' in nodeType) return ['Tally'];
  return [];
}

function getOutputTypes(nodeType: NodeType): ConnectionType[] {
  if ('Input' in nodeType) {
    const inputType = nodeType.Input;
    if (inputType === 'Camera' || inputType === 'VideoFile') return ['RenderData', 'Audio'];
    return ['RenderData'];
  }
  if ('Output' in nodeType) return [];
  if ('Effect' in nodeType) return ['RenderData'];
  if ('Audio' in nodeType) return ['Audio'];
  if ('Control' in nodeType) return ['Control'];
  if ('Tally' in nodeType) return ['Tally'];
  return [];
}

function getConnectionTypeFromHandles(
  sourceHandle: string | null,
  targetHandle: string | null
): ConnectionType {
  if (sourceHandle?.includes('renderdata') || targetHandle?.includes('renderdata')) return 'RenderData';
  if (sourceHandle?.includes('audio') || targetHandle?.includes('audio')) return 'Audio';
  if (sourceHandle?.includes('control') || targetHandle?.includes('control')) return 'Control';
  if (sourceHandle?.includes('tally') || targetHandle?.includes('tally')) return 'Tally';
  return 'RenderData';
}