export interface NodeType {
  Input: 'Camera' | 'ScreenCapture' | 'WindowCapture' | 'VideoFile' | 'TestPattern';
  Output: 'VirtualWebcam' | 'Preview';
  Effect: 'ColorCorrection' | 'Blur' | 'Sharpen' | 'Transform' | 'Composite';
  Audio: 'Input' | 'Mixer' | 'Effect' | 'Output';
  Tally: 'Generator' | 'Monitor' | 'Logic' | 'Router';
}

export type ConnectionType = 'Video' | 'Audio' | 'Tally';

export interface NodeConfig {
  parameters: Record<string, any>;
}

export interface NodeProperties {
  id: string;
  name: string;
  nodeType: NodeType;
  inputTypes: ConnectionType[];
  outputTypes: ConnectionType[];
  parameters: Record<string, ParameterDefinition>;
}

export interface ParameterDefinition {
  name: string;
  parameterType: ParameterType;
  defaultValue: any;
  minValue?: any;
  maxValue?: any;
  description: string;
}

export type ParameterType = 
  | 'Float'
  | 'Integer'
  | 'Boolean'
  | 'String'
  | 'Color'
  | 'Vector2'
  | 'Vector3'
  | 'Vector4'
  | { Enum: string[] };

export interface EngineEvent {
  NodeAdded?: { id: string; nodeType: NodeType };
  NodeRemoved?: { id: string };
  NodeConnected?: { sourceId: string; targetId: string; connectionType: ConnectionType };
  NodeDisconnected?: { sourceId: string; targetId: string };
  ParameterChanged?: { nodeId: string; parameter: string; value: any };
  FrameProcessed?: { timestamp: number };
  Error?: { message: string };
}

export interface EngineStatus {
  running: boolean;
  fps: number;
  frameCount: number;
  nodeCount: number;
}