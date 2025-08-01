import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useNodeStore } from '../stores/useNodeStore';
import { apiClient, type VideoFrame } from '../api/client';

interface VideoPreviewProps {
  nodeId: string;
  width?: number;
  height?: number;
  showStats?: boolean;
  title?: string;
}

interface VideoStats {
  fps: number;
  resolution: string;
  bitrate: string;
  latency: number;
  frameDrops: number;
}

export const VideoPreview: React.FC<VideoPreviewProps> = ({
  nodeId,
  width = 640,
  height = 360,
  showStats = true,
  title = "Video Preview"
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { apiClient } = useNodeStore();
  const [videoStats, setVideoStats] = useState<VideoStats>({
    fps: 30,
    resolution: "1920x1080",
    bitrate: "5.2 Mbps",
    latency: 33.3,
    frameDrops: 0
  });
  const [isPlaying, setIsPlaying] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const animationFrameRef = useRef<number>();
  const frameCountRef = useRef<number>(0);
  const lastFrameTimeRef = useRef<number>(Date.now());
  const unsubscribeVideoFrameRef = useRef<(() => void) | null>(null);

  // Initialize preview canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    canvas.width = width;
    canvas.height = height;

    // Draw placeholder frame
    drawPlaceholderFrame(ctx, width, height);
  }, [width, height]);

  // Start video preview
  const startPreview = useCallback(async () => {
    try {
      setError(null);
      
      // Connect to WebSocket if not already connected
      if (!apiClient.isConnected()) {
        await apiClient.connectWebSocket();
      }
      
      // Subscribe to video frames for this node
      unsubscribeVideoFrameRef.current = apiClient.addVideoFrameListener((frame: VideoFrame) => {
        if (frame.metadata.node_id === nodeId) {
          drawVideoFrame(frame);
          updateFrameStats();
        }
      });
      
      // Start preview streaming
      apiClient.startVideoPreview(nodeId);
      
      setIsPlaying(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  }, [nodeId, drawVideoFrame, updateFrameStats]);

  // Stop video preview
  const stopPreview = useCallback(async () => {
    try {
      // Stop preview streaming
      apiClient.stopVideoPreview(nodeId);
      
      // Unsubscribe from video frames
      if (unsubscribeVideoFrameRef.current) {
        unsubscribeVideoFrameRef.current();
        unsubscribeVideoFrameRef.current = null;
      }
      
      setIsPlaying(false);
      
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      
      // Draw placeholder frame
      const canvas = canvasRef.current;
      if (canvas) {
        const ctx = canvas.getContext('2d');
        if (ctx) {
          drawPlaceholderFrame(ctx, width, height);
        }
      }
    } catch (err) {
      console.error('Failed to stop preview:', err);
    }
  }, [nodeId, width, height]);

  // Draw video frame from WebSocket
  const drawVideoFrame = useCallback((frame: VideoFrame) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    // For now, since we're receiving mock JPEG data, draw a test pattern with frame info
    // In production, this would decode the actual JPEG data
    const { metadata } = frame;
    
    // Clear canvas
    ctx.clearRect(0, 0, width, height);
    
    // Draw animated background based on frame number
    const hue = (metadata.frame_number * 2) % 360;
    const gradient = ctx.createLinearGradient(0, 0, width, height);
    gradient.addColorStop(0, `hsl(${hue}, 70%, 40%)`);
    gradient.addColorStop(1, `hsl(${(hue + 120) % 360}, 70%, 20%)`);
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, width, height);
    
    // Draw frame info
    ctx.fillStyle = '#fff';
    ctx.font = '16px monospace';
    ctx.textAlign = 'center';
    ctx.fillText(`Live Video Stream`, width / 2, height / 2 - 20);
    ctx.fillText(`Frame: ${metadata.frame_number}`, width / 2, height / 2);
    ctx.fillText(`${metadata.width}x${metadata.height}`, width / 2, height / 2 + 20);
    
    // Draw timestamp
    ctx.font = '12px monospace';
    ctx.textAlign = 'left';
    ctx.fillText(`TS: ${metadata.timestamp}`, 10, 25);
    ctx.fillText(`Node: ${metadata.node_id.substring(0, 8)}...`, 10, 45);
    
    // Update resolution in stats
    setVideoStats(prev => ({
      ...prev,
      resolution: `${metadata.width}x${metadata.height}`
    }));
  }, [width, height]);
  
  // Update frame statistics
  const updateFrameStats = useCallback(() => {
    const now = Date.now();
    frameCountRef.current += 1;
    
    // Calculate FPS every second
    if (now - lastFrameTimeRef.current >= 1000) {
      const fps = frameCountRef.current;
      setVideoStats(prev => ({
        ...prev,
        fps,
        latency: Math.round(Math.random() * 10) + 25, // Simulated latency
      }));
      frameCountRef.current = 0;
      lastFrameTimeRef.current = now;
    }
  }, []);

  // Draw placeholder frame
  const drawPlaceholderFrame = (ctx: CanvasRenderingContext2D, w: number, h: number) => {
    // Background
    ctx.fillStyle = '#2a2a2a';
    ctx.fillRect(0, 0, w, h);

    // Center text
    ctx.fillStyle = '#888';
    ctx.font = '16px Arial';
    ctx.textAlign = 'center';
    ctx.fillText('No Video Signal', w / 2, h / 2 - 10);
    ctx.fillText('Click Play to Start Preview', w / 2, h / 2 + 20);

    // Border
    ctx.strokeStyle = '#555';
    ctx.lineWidth = 2;
    ctx.strokeRect(1, 1, w - 2, h - 2);
  };

  // Draw test pattern
  const drawTestPattern = (ctx: CanvasRenderingContext2D, w: number, h: number) => {
    const time = Date.now() * 0.001;
    
    // Animated gradient background
    const gradient = ctx.createLinearGradient(0, 0, w, h);
    gradient.addColorStop(0, `hsl(${(time * 20) % 360}, 70%, 50%)`);
    gradient.addColorStop(1, `hsl(${(time * 20 + 180) % 360}, 70%, 30%)`);
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, w, h);

    // Moving circles
    ctx.fillStyle = 'rgba(255, 255, 255, 0.3)';
    for (let i = 0; i < 5; i++) {
      const x = (Math.sin(time + i) * 0.3 + 0.5) * w;
      const y = (Math.cos(time * 0.7 + i) * 0.3 + 0.5) * h;
      const radius = 20 + Math.sin(time * 2 + i) * 10;
      
      ctx.beginPath();
      ctx.arc(x, y, radius, 0, Math.PI * 2);
      ctx.fill();
    }

    // Frame counter
    ctx.fillStyle = '#fff';
    ctx.font = '12px monospace';
    ctx.textAlign = 'left';
    ctx.fillText(`Frame: ${Math.floor(time * 30) % 10000}`, 10, 20);
    ctx.fillText(`Time: ${time.toFixed(2)}s`, 10, 35);
  };

  // Toggle preview
  const togglePreview = useCallback(() => {
    if (isPlaying) {
      stopPreview();
    } else {
      startPreview();
    }
  }, [isPlaying, stopPreview, startPreview]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      if (unsubscribeVideoFrameRef.current) {
        unsubscribeVideoFrameRef.current();
      }
      if (isPlaying) {
        stopPreview();
      }
    };
  }, [isPlaying, stopPreview]);

  return (
    <div className="video-preview-container">
      {/* Preview Header */}
      <div className="preview-header">
        <h3 className="preview-title">{title}</h3>
        <div className="preview-controls">
          <button 
            onClick={togglePreview}
            className={`preview-btn ${isPlaying ? 'stop' : 'play'}`}
          >
            {isPlaying ? '⏸️ Stop' : '▶️ Play'}
          </button>
        </div>
      </div>

      {/* Video Canvas */}
      <div className="video-canvas-container">
        <canvas
          ref={canvasRef}
          className="video-canvas"
          style={{
            width: '100%',
            height: 'auto',
            maxWidth: width,
            maxHeight: height,
            backgroundColor: '#000'
          }}
        />
        
        {/* Error Overlay */}
        {error && (
          <div className="error-overlay">
            <span className="error-text">❌ {error}</span>
          </div>
        )}
      </div>

      {/* Video Stats */}
      {showStats && (
        <div className="video-stats">
          <div className="stats-grid">
            <div className="stat-item">
              <span className="stat-label">FPS:</span>
              <span className="stat-value">{videoStats.fps}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Resolution:</span>
              <span className="stat-value">{videoStats.resolution}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Bitrate:</span>
              <span className="stat-value">{videoStats.bitrate}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Latency:</span>
              <span className="stat-value">{videoStats.latency}ms</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Drops:</span>
              <span className="stat-value">{videoStats.frameDrops}</span>
            </div>
          </div>
        </div>
      )}

      <style jsx>{`
        .video-preview-container {
          border: 1px solid #333;
          border-radius: 8px;
          background: #1a1a1a;
          overflow: hidden;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
        }

        .preview-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          background: #2a2a2a;
          border-bottom: 1px solid #333;
        }

        .preview-title {
          color: #fff;
          font-size: 14px;
          font-weight: 600;
          margin: 0;
        }

        .preview-controls {
          display: flex;
          gap: 8px;
        }

        .preview-btn {
          padding: 6px 12px;
          border: 1px solid #555;
          border-radius: 4px;
          background: #333;
          color: #fff;
          font-size: 12px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .preview-btn:hover {
          background: #444;
          border-color: #666;
        }

        .preview-btn.play {
          background: #0f7b0f;
          border-color: #0f7b0f;
        }

        .preview-btn.stop {
          background: #c41e3a;
          border-color: #c41e3a;
        }

        .video-canvas-container {
          position: relative;
          padding: 16px;
        }

        .video-canvas {
          border: 1px solid #333;
          border-radius: 4px;
          display: block;
        }

        .error-overlay {
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          background: rgba(196, 30, 58, 0.9);
          color: white;
          padding: 8px 16px;
          border-radius: 4px;
          font-size: 14px;
        }

        .video-stats {
          padding: 12px 16px;
          background: #222;
          border-top: 1px solid #333;
        }

        .stats-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
          gap: 8px;
        }

        .stat-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 4px 8px;
          background: #333;
          border-radius: 4px;
          font-size: 12px;
        }

        .stat-label {
          color: #aaa;
          font-weight: 500;
        }

        .stat-value {
          color: #fff;
          font-weight: 600;
          font-family: monospace;
        }
      `}</style>
    </div>
  );
};

export default VideoPreview;