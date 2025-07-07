// Project and configuration types for save/load functionality

export interface ProjectNode {
  id: string;
  type: string; // constellation
  position: { x: number; y: number };
  data: {
    nodeType: any;
    label: string;
    inputTypes: string[];
    outputTypes: string[];
    parameters?: Record<string, any>;
  };
}

export interface ProjectEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string | null;
  targetHandle?: string | null;
  type: string; // constellation
  data?: {
    connectionType: string;
  };
}

export interface ProjectConfiguration {
  version: string;
  name: string;
  description?: string;
  createdAt: string;
  modifiedAt: string;
  nodes: ProjectNode[];
  edges: ProjectEdge[];
  settings?: {
    canvasPosition?: { x: number; y: number; zoom: number };
    theme?: string;
  };
  metadata?: {
    author?: string;
    tags?: string[];
    [key: string]: any;
  };
}

export interface ProjectSaveOptions {
  name?: string;
  description?: string;
  metadata?: Record<string, any>;
}

export interface ProjectLoadResult {
  success: boolean;
  configuration?: ProjectConfiguration & {
    nodes: any[]; // React Flow Node[]
    edges: any[]; // React Flow Edge[]
  };
  error?: string;
  warnings?: string[];
}