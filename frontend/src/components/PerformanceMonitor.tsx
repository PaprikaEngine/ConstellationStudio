import React, { useEffect, useState, useRef } from 'react';
import { useNodeStore } from '../stores/useNodeStore';

interface PerformanceData {
  timestamp: number;
  fps: number;
  cpu: number;
  memory: number;
  gpu: number;
  latency: number;
  frameTime: number;
  drops: number;
}

interface NodePerformance {
  nodeId: string;
  nodeName: string;
  processingTime: number;
  memoryUsage: number;
  errorCount: number;
  lastError?: string;
}

interface PerformanceMonitorProps {
  updateInterval?: number;
  historyLength?: number;
}

export const PerformanceMonitor: React.FC<PerformanceMonitorProps> = ({
  updateInterval = 1000,
  historyLength = 60
}) => {
  const { apiClient, engineStatus } = useNodeStore();
  const [performanceData, setPerformanceData] = useState<PerformanceData[]>([]);
  const [nodePerformance, setNodePerformance] = useState<NodePerformance[]>([]);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [alerts, setAlerts] = useState<string[]>([]);
  const intervalRef = useRef<NodeJS.Timeout>();

  // Performance thresholds
  const THRESHOLDS = {
    fps: { warning: 25, critical: 20 },
    cpu: { warning: 80, critical: 90 },
    memory: { warning: 85, critical: 95 },
    gpu: { warning: 85, critical: 95 },
    latency: { warning: 100, critical: 200 },
    frameTime: { warning: 40, critical: 50 }
  };

  // Start monitoring
  const startMonitoring = async () => {
    try {
      setIsMonitoring(true);
      
      // Request monitoring start from backend
      await apiClient.request('POST', '/api/monitoring/start', {
        interval: updateInterval,
        metrics: ['fps', 'cpu', 'memory', 'gpu', 'latency', 'nodes']
      });

      // Start local update loop
      intervalRef.current = setInterval(updatePerformanceData, updateInterval);
    } catch (err) {
      console.error('Failed to start monitoring:', err);
      setIsMonitoring(false);
    }
  };

  // Stop monitoring
  const stopMonitoring = async () => {
    try {
      await apiClient.request('POST', '/api/monitoring/stop');
      
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = undefined;
      }
      
      setIsMonitoring(false);
    } catch (err) {
      console.error('Failed to stop monitoring:', err);
    }
  };

  // Update performance data
  const updatePerformanceData = async () => {
    try {
      const response = await apiClient.request('GET', '/api/monitoring/metrics');
      
      if (response.success && response.data) {
        const newData: PerformanceData = {
          timestamp: Date.now(),
          fps: response.data.fps || 0,
          cpu: response.data.cpu || 0,
          memory: response.data.memory || 0,
          gpu: response.data.gpu || 0,
          latency: response.data.latency || 0,
          frameTime: response.data.frameTime || 0,
          drops: response.data.drops || 0
        };

        // Update performance history
        setPerformanceData(prev => {
          const updated = [...prev, newData];
          return updated.slice(-historyLength);
        });

        // Update node performance
        if (response.data.nodes) {
          setNodePerformance(response.data.nodes);
        }

        // Check for alerts
        checkAlerts(newData);
      }
    } catch (err) {
      console.error('Failed to update performance data:', err);
    }
  };

  // Check for performance alerts
  const checkAlerts = (data: PerformanceData) => {
    const newAlerts: string[] = [];

    // Check each metric against thresholds
    Object.entries(THRESHOLDS).forEach(([metric, threshold]) => {
      const value = data[metric as keyof PerformanceData] as number;
      
      if (value >= threshold.critical) {
        newAlerts.push(`🔴 CRITICAL: ${metric.toUpperCase()} at ${value}${getUnit(metric)}`);
      } else if (value >= threshold.warning) {
        newAlerts.push(`🟡 WARNING: ${metric.toUpperCase()} at ${value}${getUnit(metric)}`);
      }
    });

    // Update alerts (keep only recent ones)
    setAlerts(prev => {
      const combined = [...prev, ...newAlerts];
      return combined.slice(-10); // Keep last 10 alerts
    });
  };

  // Get unit for metric
  const getUnit = (metric: string): string => {
    switch (metric) {
      case 'fps': return ' fps';
      case 'cpu':
      case 'memory':
      case 'gpu': return '%';
      case 'latency':
      case 'frameTime': return 'ms';
      default: return '';
    }
  };

  // Get status color based on value and thresholds
  const getStatusColor = (value: number, metric: string): string => {
    const threshold = THRESHOLDS[metric as keyof typeof THRESHOLDS];
    if (!threshold) return '#00ff00';
    
    if (value >= threshold.critical) return '#ff0000';
    if (value >= threshold.warning) return '#ffaa00';
    return '#00ff00';
  };

  // Format number with appropriate precision
  const formatNumber = (value: number, decimals: number = 1): string => {
    return value.toFixed(decimals);
  };

  // Toggle monitoring
  const toggleMonitoring = () => {
    if (isMonitoring) {
      stopMonitoring();
    } else {
      startMonitoring();
    }
  };

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
      if (isMonitoring) {
        stopMonitoring();
      }
    };
  }, [isMonitoring]);

  // Get current performance data
  const currentData = performanceData[performanceData.length - 1];

  return (
    <div className="performance-monitor">
      {/* Header */}
      <div className="monitor-header">
        <h3 className="monitor-title">Performance Monitor</h3>
        <div className="monitor-controls">
          <button 
            onClick={toggleMonitoring}
            className={`monitor-btn ${isMonitoring ? 'stop' : 'start'}`}
          >
            {isMonitoring ? '⏸️ Stop' : '▶️ Start'}
          </button>
          <button 
            onClick={() => setAlerts([])}
            className="monitor-btn clear"
          >
            🗑️ Clear Alerts
          </button>
        </div>
      </div>

      {/* System Performance */}
      <div className="performance-section">
        <h4 className="section-title">System Performance</h4>
        <div className="metrics-grid">
          <div className="metric-card">
            <div className="metric-header">
              <span className="metric-name">FPS</span>
              <span 
                className="metric-value"
                style={{ color: getStatusColor(currentData?.fps || 0, 'fps') }}
              >
                {formatNumber(currentData?.fps || 0, 0)}
              </span>
            </div>
            <div className="metric-bar">
              <div 
                className="metric-fill"
                style={{ 
                  width: `${Math.min((currentData?.fps || 0) / 60 * 100, 100)}%`,
                  backgroundColor: getStatusColor(currentData?.fps || 0, 'fps')
                }}
              />
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-header">
              <span className="metric-name">CPU</span>
              <span 
                className="metric-value"
                style={{ color: getStatusColor(currentData?.cpu || 0, 'cpu') }}
              >
                {formatNumber(currentData?.cpu || 0)}%
              </span>
            </div>
            <div className="metric-bar">
              <div 
                className="metric-fill"
                style={{ 
                  width: `${currentData?.cpu || 0}%`,
                  backgroundColor: getStatusColor(currentData?.cpu || 0, 'cpu')
                }}
              />
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-header">
              <span className="metric-name">Memory</span>
              <span 
                className="metric-value"
                style={{ color: getStatusColor(currentData?.memory || 0, 'memory') }}
              >
                {formatNumber(currentData?.memory || 0)}%
              </span>
            </div>
            <div className="metric-bar">
              <div 
                className="metric-fill"
                style={{ 
                  width: `${currentData?.memory || 0}%`,
                  backgroundColor: getStatusColor(currentData?.memory || 0, 'memory')
                }}
              />
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-header">
              <span className="metric-name">GPU</span>
              <span 
                className="metric-value"
                style={{ color: getStatusColor(currentData?.gpu || 0, 'gpu') }}
              >
                {formatNumber(currentData?.gpu || 0)}%
              </span>
            </div>
            <div className="metric-bar">
              <div 
                className="metric-fill"
                style={{ 
                  width: `${currentData?.gpu || 0}%`,
                  backgroundColor: getStatusColor(currentData?.gpu || 0, 'gpu')
                }}
              />
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-header">
              <span className="metric-name">Latency</span>
              <span 
                className="metric-value"
                style={{ color: getStatusColor(currentData?.latency || 0, 'latency') }}
              >
                {formatNumber(currentData?.latency || 0)}ms
              </span>
            </div>
            <div className="metric-bar">
              <div 
                className="metric-fill"
                style={{ 
                  width: `${Math.min((currentData?.latency || 0) / 200 * 100, 100)}%`,
                  backgroundColor: getStatusColor(currentData?.latency || 0, 'latency')
                }}
              />
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-header">
              <span className="metric-name">Frame Time</span>
              <span 
                className="metric-value"
                style={{ color: getStatusColor(currentData?.frameTime || 0, 'frameTime') }}
              >
                {formatNumber(currentData?.frameTime || 0)}ms
              </span>
            </div>
            <div className="metric-bar">
              <div 
                className="metric-fill"
                style={{ 
                  width: `${Math.min((currentData?.frameTime || 0) / 50 * 100, 100)}%`,
                  backgroundColor: getStatusColor(currentData?.frameTime || 0, 'frameTime')
                }}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Node Performance */}
      <div className="performance-section">
        <h4 className="section-title">Node Performance</h4>
        <div className="node-list">
          {nodePerformance.map((node) => (
            <div key={node.nodeId} className="node-card">
              <div className="node-header">
                <span className="node-name">{node.nodeName}</span>
                <span className="node-time">{formatNumber(node.processingTime)}ms</span>
              </div>
              <div className="node-details">
                <span className="node-detail">Memory: {formatNumber(node.memoryUsage)}MB</span>
                <span className="node-detail">Errors: {node.errorCount}</span>
              </div>
              {node.lastError && (
                <div className="node-error">
                  Last Error: {node.lastError}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Alerts */}
      {alerts.length > 0 && (
        <div className="performance-section">
          <h4 className="section-title">Alerts</h4>
          <div className="alerts-list">
            {alerts.map((alert, index) => (
              <div key={index} className="alert-item">
                {alert}
              </div>
            ))}
          </div>
        </div>
      )}

      <style jsx>{`
        .performance-monitor {
          background: #1a1a1a;
          border: 1px solid #333;
          border-radius: 8px;
          padding: 16px;
          color: #fff;
        }

        .monitor-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
          padding-bottom: 12px;
          border-bottom: 1px solid #333;
        }

        .monitor-title {
          font-size: 18px;
          font-weight: 600;
          margin: 0;
        }

        .monitor-controls {
          display: flex;
          gap: 8px;
        }

        .monitor-btn {
          padding: 6px 12px;
          border: 1px solid #555;
          border-radius: 4px;
          background: #333;
          color: #fff;
          font-size: 12px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .monitor-btn:hover {
          background: #444;
        }

        .monitor-btn.start {
          background: #0f7b0f;
          border-color: #0f7b0f;
        }

        .monitor-btn.stop {
          background: #c41e3a;
          border-color: #c41e3a;
        }

        .monitor-btn.clear {
          background: #666;
          border-color: #666;
        }

        .performance-section {
          margin-bottom: 24px;
        }

        .section-title {
          font-size: 14px;
          font-weight: 600;
          margin-bottom: 12px;
          color: #aaa;
        }

        .metrics-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
          gap: 12px;
        }

        .metric-card {
          background: #2a2a2a;
          border: 1px solid #333;
          border-radius: 6px;
          padding: 12px;
        }

        .metric-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;
        }

        .metric-name {
          font-size: 12px;
          font-weight: 500;
          color: #aaa;
        }

        .metric-value {
          font-size: 16px;
          font-weight: 600;
          font-family: monospace;
        }

        .metric-bar {
          height: 4px;
          background: #444;
          border-radius: 2px;
          overflow: hidden;
        }

        .metric-fill {
          height: 100%;
          transition: width 0.3s ease;
        }

        .node-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .node-card {
          background: #2a2a2a;
          border: 1px solid #333;
          border-radius: 6px;
          padding: 12px;
        }

        .node-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 6px;
        }

        .node-name {
          font-size: 14px;
          font-weight: 500;
        }

        .node-time {
          font-size: 12px;
          font-family: monospace;
          color: #0ff;
        }

        .node-details {
          display: flex;
          gap: 16px;
          font-size: 12px;
          color: #aaa;
        }

        .node-error {
          margin-top: 6px;
          padding: 4px 8px;
          background: #3a1a1a;
          border: 1px solid #5a2a2a;
          border-radius: 4px;
          font-size: 11px;
          color: #faa;
        }

        .alerts-list {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .alert-item {
          padding: 8px 12px;
          background: #3a2a1a;
          border: 1px solid #5a4a2a;
          border-radius: 4px;
          font-size: 12px;
          font-family: monospace;
        }
      `}</style>
    </div>
  );
};

export default PerformanceMonitor;