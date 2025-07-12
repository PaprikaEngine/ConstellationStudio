export type NodeType = 
  | { Input: 'Camera' | 'ScreenCapture' | 'WindowCapture' | 'VideoFile' | 'TestPattern' }
  | { Output: 'VirtualWebcam' | 'Preview' | 'Viewer' }
  | { Effect: 'ColorCorrection' | 'Blur' | 'Sharpen' | 'Transform' | 'Composite' }
  | { Audio: 'Input' | 'Mixer' | 'Effect' | 'Output' }
  | { Control: 'LFO' | 'Timeline' | 'MathController' | 'MidiController' | 'OscController' | 'ParameterController' | 'AnimationController' }
  | { Tally: 'Generator' | 'Monitor' | 'Logic' | 'Router' };

// Updated to match Issue #12 new connection system
export type ConnectionType = 'RenderData' | 'Audio' | 'Control' | 'Tally';

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

export interface AudioLevel {
  peak_left: number;
  peak_right: number;
  rms_left: number;
  rms_right: number;
  db_peak_left: number;
  db_peak_right: number;
  db_rms_left: number;
  db_rms_right: number;
  is_clipping: boolean;
  timestamp: number;
}

export interface EngineEvent {
  NodeAdded?: { id: string; nodeType: NodeType };
  NodeRemoved?: { id: string };
  NodeConnected?: { sourceId: string; targetId: string; connectionType: ConnectionType };
  NodeDisconnected?: { sourceId: string; targetId: string };
  ParameterChanged?: { nodeId: string; parameter: string; value: any };
  FrameProcessed?: { timestamp: number };
  Error?: { message: string };
  AudioLevel?: {
    node_id: string;
    peak_left: number;
    peak_right: number;
    rms_left: number;
    rms_right: number;
    db_peak_left: number;
    db_peak_right: number;
    db_rms_left: number;
    db_rms_right: number;
    is_clipping: boolean;
    timestamp: number;
  };
}

export interface EngineStatus {
  running: boolean;
  fps: number;
  frameCount: number;
  nodeCount: number;
}