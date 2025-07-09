// Project save/load utilities

import { Node, Edge, Viewport } from 'reactflow';
import { ProjectConfiguration, ProjectNode, ProjectEdge, ProjectSaveOptions, ProjectLoadResult } from '../types/project';
import { NodeType } from '../types';

const PROJECT_VERSION = '1.0.0';

export class ProjectManager {
  // Convert React Flow nodes/edges to project format
  static exportConfiguration(
    nodes: Node[],
    edges: Edge[],
    viewport?: Viewport,
    options: ProjectSaveOptions = {}
  ): ProjectConfiguration {
    const now = new Date().toISOString();
    
    const projectNodes: ProjectNode[] = nodes.map(node => ({
      id: node.id,
      type: node.type || 'constellation',
      position: node.position,
      data: {
        nodeType: node.data.nodeType,
        label: node.data.label,
        inputTypes: node.data.inputTypes || [],
        outputTypes: node.data.outputTypes || [],
        parameters: node.data.parameters || {},
      },
    }));

    const projectEdges: ProjectEdge[] = edges.map(edge => ({
      id: edge.id,
      source: edge.source,
      target: edge.target,
      sourceHandle: edge.sourceHandle,
      targetHandle: edge.targetHandle,
      type: edge.type || 'constellation',
      data: edge.data,
    }));

    const configuration: ProjectConfiguration = {
      version: PROJECT_VERSION,
      name: options.name || `Project_${new Date().toISOString().split('T')[0]}`,
      description: options.description || 'Constellation Studio project configuration',
      createdAt: now,
      modifiedAt: now,
      nodes: projectNodes,
      edges: projectEdges,
      settings: {
        canvasPosition: viewport ? {
          x: viewport.x,
          y: viewport.y,
          zoom: viewport.zoom,
        } : undefined,
      },
      metadata: {
        nodeCount: nodes.length,
        edgeCount: edges.length,
        exportedBy: 'Constellation Studio',
        ...options.metadata,
      },
    };

    return configuration;
  }

  // Convert project format back to React Flow nodes/edges
  static importConfiguration(config: ProjectConfiguration): ProjectLoadResult {
    try {
      // Validate configuration
      const validation = this.validateConfiguration(config);
      if (!validation.isValid) {
        return {
          success: false,
          error: `Invalid configuration: ${validation.errors.join(', ')}`,
        };
      }

      const warnings: string[] = [];

      // Convert nodes
      const nodes: Node[] = config.nodes.map(projectNode => {
        const node: Node = {
          id: projectNode.id,
          type: projectNode.type,
          position: projectNode.position,
          data: {
            nodeType: projectNode.data.nodeType,
            label: projectNode.data.label,
            inputTypes: projectNode.data.inputTypes,
            outputTypes: projectNode.data.outputTypes,
            parameters: projectNode.data.parameters || {},
          },
        };

        return node;
      });

      // Convert edges
      const edges: Edge[] = config.edges.map(projectEdge => {
        const edge: Edge = {
          id: projectEdge.id,
          source: projectEdge.source,
          target: projectEdge.target,
          sourceHandle: projectEdge.sourceHandle,
          targetHandle: projectEdge.targetHandle,
          type: projectEdge.type,
          data: projectEdge.data,
        };

        return edge;
      });

      // Check for missing node references in edges
      const nodeIds = new Set(nodes.map(n => n.id));
      edges.forEach(edge => {
        if (!nodeIds.has(edge.source)) {
          warnings.push(`Edge ${edge.id} references missing source node: ${edge.source}`);
        }
        if (!nodeIds.has(edge.target)) {
          warnings.push(`Edge ${edge.id} references missing target node: ${edge.target}`);
        }
      });

      return {
        success: true,
        configuration: {
          ...config,
          nodes: nodes as any,
          edges: edges as any,
        },
        warnings: warnings.length > 0 ? warnings : undefined,
      };
    } catch (error) {
      return {
        success: false,
        error: `Failed to import configuration: ${error instanceof Error ? error.message : 'Unknown error'}`,
      };
    }
  }

  // Validate project configuration
  static validateConfiguration(config: any): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!config) {
      errors.push('Configuration is null or undefined');
      return { isValid: false, errors };
    }

    if (!config.version) {
      errors.push('Missing version field');
    }

    if (!config.name) {
      errors.push('Missing name field');
    }

    if (!Array.isArray(config.nodes)) {
      errors.push('Nodes field must be an array');
    } else {
      config.nodes.forEach((node: any, index: number) => {
        if (!node.id) errors.push(`Node ${index} missing id`);
        if (!node.position) errors.push(`Node ${index} missing position`);
        if (!node.data) errors.push(`Node ${index} missing data`);
      });
    }

    if (!Array.isArray(config.edges)) {
      errors.push('Edges field must be an array');
    } else {
      config.edges.forEach((edge: any, index: number) => {
        if (!edge.id) errors.push(`Edge ${index} missing id`);
        if (!edge.source) errors.push(`Edge ${index} missing source`);
        if (!edge.target) errors.push(`Edge ${index} missing target`);
      });
    }

    return { isValid: errors.length === 0, errors };
  }

  // Save configuration to file
  static saveToFile(config: ProjectConfiguration, filename?: string): void {
    const blob = new Blob([JSON.stringify(config, null, 2)], {
      type: 'application/json',
    });

    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename || `${config.name.replace(/\s+/g, '_')}.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }

  // Load configuration from file
  static loadFromFile(): Promise<ProjectLoadResult> {
    return new Promise((resolve) => {
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.json,application/json';
      
      input.onchange = async (event) => {
        const file = (event.target as HTMLInputElement).files?.[0];
        if (!file) {
          resolve({
            success: false,
            error: 'No file selected',
          });
          return;
        }

        try {
          const text = await file.text();
          const config = JSON.parse(text);
          const result = this.importConfiguration(config);
          resolve(result);
        } catch (error) {
          resolve({
            success: false,
            error: `Failed to read file: ${error instanceof Error ? error.message : 'Unknown error'}`,
          });
        }
      };

      input.oncancel = () => {
        resolve({
          success: false,
          error: 'File selection cancelled',
        });
      };

      document.body.appendChild(input);
      input.click();
      document.body.removeChild(input);
    });
  }

  // Save configuration to localStorage
  static saveToLocalStorage(config: ProjectConfiguration, key: string = 'constellation-project'): boolean {
    try {
      localStorage.setItem(key, JSON.stringify(config));
      return true;
    } catch (error) {
      console.error('Failed to save to localStorage:', error);
      return false;
    }
  }

  // Load configuration from localStorage
  static loadFromLocalStorage(key: string = 'constellation-project'): ProjectLoadResult {
    try {
      const stored = localStorage.getItem(key);
      if (!stored) {
        return {
          success: false,
          error: 'No saved project found',
        };
      }

      const config = JSON.parse(stored);
      return this.importConfiguration(config);
    } catch (error) {
      return {
        success: false,
        error: `Failed to load from localStorage: ${error instanceof Error ? error.message : 'Unknown error'}`,
      };
    }
  }

  // Generate sample configuration
  static generateSampleConfiguration(): ProjectConfiguration {
    const now = new Date().toISOString();
    
    return {
      version: PROJECT_VERSION,
      name: 'Sample Project',
      description: 'A sample Constellation Studio project with basic video processing nodes',
      createdAt: now,
      modifiedAt: now,
      nodes: [
        {
          id: 'input-1',
          type: 'constellation',
          position: { x: 100, y: 100 },
          data: {
            nodeType: { Input: 'Camera' },
            label: 'Camera Input',
            inputTypes: [],
            outputTypes: ['RenderData', 'Audio'],
            parameters: {
              deviceId: { value: 'default', type: 'string' },
              resolution: { value: '1920x1080', type: 'string' },
            },
          },
        },
        {
          id: 'effect-1',
          type: 'constellation',
          position: { x: 350, y: 100 },
          data: {
            nodeType: { Effect: 'ColorCorrection' },
            label: 'Color Correction Effect',
            inputTypes: ['RenderData'],
            outputTypes: ['RenderData'],
            parameters: {
              brightness: { value: 1.0, type: 'float' },
              contrast: { value: 1.0, type: 'float' },
              saturation: { value: 1.0, type: 'float' },
            },
          },
        },
        {
          id: 'output-1',
          type: 'constellation',
          position: { x: 600, y: 100 },
          data: {
            nodeType: { Output: 'VirtualWebcam' },
            label: 'Virtual Webcam Output',
            inputTypes: ['RenderData', 'Audio'],
            outputTypes: [],
            parameters: {
              quality: { value: 'high', type: 'string' },
            },
          },
        },
      ],
      edges: [
        {
          id: 'edge-1',
          source: 'input-1',
          target: 'effect-1',
          sourceHandle: 'output-renderdata-0',
          targetHandle: 'input-renderdata-0',
          type: 'constellation',
          data: {
            connectionType: 'RenderData',
          },
        },
        {
          id: 'edge-2',
          source: 'effect-1',
          target: 'output-1',
          sourceHandle: 'output-renderdata-0',
          targetHandle: 'input-renderdata-0',
          type: 'constellation',
          data: {
            connectionType: 'RenderData',
          },
        },
        {
          id: 'edge-3',
          source: 'input-1',
          target: 'output-1',
          sourceHandle: 'output-audio-0',
          targetHandle: 'input-audio-0',
          type: 'constellation',
          data: {
            connectionType: 'Audio',
          },
        },
      ],
      settings: {
        canvasPosition: { x: 0, y: 0, zoom: 1 },
      },
      metadata: {
        nodeCount: 3,
        edgeCount: 3,
        exportedBy: 'Constellation Studio',
        author: 'Sample',
        tags: ['sample', 'camera', 'color-correction'],
      },
    };
  }
}