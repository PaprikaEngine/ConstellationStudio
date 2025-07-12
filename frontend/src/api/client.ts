// API Client for Constellation Studio Backend Communication

import axios, { AxiosInstance } from 'axios';
import type { 
  NodeType, 
  NodeConfig, 
  ConnectionType, 
  EngineEvent
} from '../types';

// Configuration
const API_BASE_URL = (import.meta as any).env?.VITE_API_URL || 'http://localhost:3001';
const WS_BASE_URL = (import.meta as any).env?.VITE_WS_URL || 'ws://localhost:3001';

// Request/Response Types
export interface CreateNodeRequest {
  node_type: NodeType;
  config: NodeConfig;
}

export interface CreateConnectionRequest {
  source_id: string;
  target_id: string;
  connection_type: ConnectionType;
}

export interface SetParametersRequest {
  parameters: Record<string, any>;
}

export interface ApiEngineStatus {
  running: boolean;
  fps: number;
  frame_count: number;
  node_count: number;
  connection_count?: number;
}

export interface VideoFrameMetadata {
  type: 'video_frame';
  node_id: string;
  width: number;
  height: number;
  format: 'jpeg' | 'png' | 'rgba8' | 'rgb8';
  timestamp: number;
  frame_number: number;
}

export interface VideoFrame {
  metadata: VideoFrameMetadata;
  data: ArrayBuffer;
}

// API Client Class
export class ConstellationApiClient {
  private api: AxiosInstance;
  private wsConnection: WebSocket | null = null;
  private eventListeners: ((event: EngineEvent) => void)[] = [];
  private videoFrameListeners: ((frame: VideoFrame) => void)[] = [];
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private pendingFrameMetadata: VideoFrameMetadata | null = null;

  constructor() {
    this.api = axios.create({
      baseURL: API_BASE_URL,
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Add request/response interceptors for logging
    this.api.interceptors.request.use(
      (config) => {
        console.log(`🚀 API Request: ${config.method?.toUpperCase()} ${config.url}`, config.data);
        return config;
      },
      (error) => {
        console.error('❌ API Request Error:', error);
        return Promise.reject(error);
      }
    );

    this.api.interceptors.response.use(
      (response) => {
        console.log(`✅ API Response: ${response.status} ${response.config.url}`, response.data);
        return response;
      },
      (error) => {
        console.error('❌ API Response Error:', error.response?.status, error.response?.data);
        return Promise.reject(error);
      }
    );
  }

  // Node Management
  async getAllNodes(): Promise<Record<string, string>> {
    const response = await this.api.get<Record<string, string>>('/api/nodes');
    return response.data;
  }

  async createNode(nodeType: NodeType, config: NodeConfig = { parameters: {} }): Promise<string> {
    const request: CreateNodeRequest = {
      node_type: nodeType,
      config,
    };
    const response = await this.api.post<string>('/api/nodes', request);
    return response.data;
  }

  async getNode(nodeId: string): Promise<string> {
    const response = await this.api.get<string>(`/api/nodes/${nodeId}`);
    return response.data;
  }

  async updateNode(nodeId: string, data: any): Promise<void> {
    await this.api.put(`/api/nodes/${nodeId}`, data);
  }

  async deleteNode(nodeId: string): Promise<void> {
    await this.api.delete(`/api/nodes/${nodeId}`);
  }

  async setNodeParameters(nodeId: string, parameters: Record<string, any>): Promise<void> {
    const request: SetParametersRequest = { parameters };
    await this.api.put(`/api/nodes/${nodeId}/parameters`, request);
  }

  // Connection Management
  async createConnection(sourceId: string, targetId: string, connectionType: ConnectionType): Promise<void> {
    const request: CreateConnectionRequest = {
      source_id: sourceId,
      target_id: targetId,
      connection_type: connectionType,
    };
    await this.api.post('/api/connections', request);
  }

  async deleteConnection(sourceId: string, targetId: string): Promise<void> {
    await this.api.delete(`/api/connections/${sourceId}/${targetId}`);
  }

  // Engine Control
  async startEngine(): Promise<void> {
    await this.api.post('/api/engine/start');
  }

  async stopEngine(): Promise<void> {
    await this.api.post('/api/engine/stop');
  }

  async getEngineStatus(): Promise<ApiEngineStatus> {
    const response = await this.api.get<ApiEngineStatus>('/api/engine/status');
    return response.data;
  }

  // WebSocket Connection
  connectWebSocket(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.wsConnection?.readyState === WebSocket.OPEN) {
        resolve();
        return;
      }

      const wsUrl = `${WS_BASE_URL}/ws`;
      console.log(`🔌 Connecting to WebSocket: ${wsUrl}`);

      this.wsConnection = new WebSocket(wsUrl);

      this.wsConnection.onopen = () => {
        console.log('✅ WebSocket connected');
        this.reconnectAttempts = 0;
        resolve();
      };

      this.wsConnection.onmessage = (event) => {
        // Handle binary data (video frames)
        if (event.data instanceof ArrayBuffer) {
          if (this.pendingFrameMetadata) {
            const videoFrame: VideoFrame = {
              metadata: this.pendingFrameMetadata,
              data: event.data
            };
            
            console.log('🎥 Received video frame:', videoFrame.metadata.node_id, videoFrame.metadata.frame_number);
            
            // Notify video frame listeners
            this.videoFrameListeners.forEach(listener => {
              try {
                listener(videoFrame);
              } catch (error) {
                console.error('❌ Error in video frame listener:', error);
              }
            });
            
            this.pendingFrameMetadata = null;
          }
          return;
        }
        
        // Handle text data (events and frame metadata)
        try {
          const message = JSON.parse(event.data);
          
          // Check if this is video frame metadata
          if (message.type === 'video_frame') {
            this.pendingFrameMetadata = message as VideoFrameMetadata;
            return;
          }
          
          // Handle regular engine events
          const engineEvent: EngineEvent = message;
          console.log('📨 WebSocket event:', engineEvent);
          
          // Notify all listeners
          this.eventListeners.forEach(listener => {
            try {
              listener(engineEvent);
            } catch (error) {
              console.error('❌ Error in event listener:', error);
            }
          });
        } catch (error) {
          console.error('❌ Failed to parse WebSocket message:', error);
        }
      };

      this.wsConnection.onclose = (event) => {
        console.log('🔌 WebSocket disconnected:', event.code, event.reason);
        this.wsConnection = null;
        
        // Auto-reconnect
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
          this.reconnectAttempts++;
          const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 10000);
          console.log(`🔄 Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
          setTimeout(() => this.connectWebSocket(), delay);
        }
      };

      this.wsConnection.onerror = (error) => {
        console.error('❌ WebSocket error:', error);
        reject(new Error('WebSocket connection failed'));
      };
    });
  }

  disconnectWebSocket(): void {
    if (this.wsConnection) {
      this.wsConnection.close();
      this.wsConnection = null;
    }
  }

  // Event Listeners
  addEventListener(listener: (event: EngineEvent) => void): () => void {
    this.eventListeners.push(listener);
    
    // Return unsubscribe function
    return () => {
      const index = this.eventListeners.indexOf(listener);
      if (index > -1) {
        this.eventListeners.splice(index, 1);
      }
    };
  }

  // Video Frame Listeners
  addVideoFrameListener(listener: (frame: VideoFrame) => void): () => void {
    this.videoFrameListeners.push(listener);
    
    // Return unsubscribe function
    return () => {
      const index = this.videoFrameListeners.indexOf(listener);
      if (index > -1) {
        this.videoFrameListeners.splice(index, 1);
      }
    };
  }

  // Video Preview Control
  startVideoPreview(nodeId: string): void {
    if (this.wsConnection?.readyState === WebSocket.OPEN) {
      const message = {
        type: 'preview_start',
        node_id: nodeId
      };
      this.wsConnection.send(JSON.stringify(message));
      console.log(`🎥 Started video preview for node ${nodeId}`);
    }
  }

  stopVideoPreview(nodeId: string): void {
    if (this.wsConnection?.readyState === WebSocket.OPEN) {
      const message = {
        type: 'preview_stop',
        node_id: nodeId
      };
      this.wsConnection.send(JSON.stringify(message));
      console.log(`🛑 Stopped video preview for node ${nodeId}`);
    }
  }

  // Utility Methods
  isConnected(): boolean {
    return this.wsConnection?.readyState === WebSocket.OPEN;
  }

  async healthCheck(): Promise<boolean> {
    try {
      await this.getEngineStatus();
      return true;
    } catch (error) {
      console.error('❌ Health check failed:', error);
      return false;
    }
  }
}

// Singleton instance
export const apiClient = new ConstellationApiClient();

// React Hook for easier usage
export const useApiClient = () => {
  return apiClient;
};