import React, { useState, useEffect } from 'react';
import { useNodeStore } from '../stores/useNodeStore';
import VideoPreview from './VideoPreview';
import PerformanceMonitor from './PerformanceMonitor';

interface PreviewMonitorPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

interface PreviewNode {
  id: string;
  name: string;
  type: string;
  isActive: boolean;
}

export const PreviewMonitorPanel: React.FC<PreviewMonitorPanelProps> = ({
  isOpen,
  onClose
}) => {
  const { nodes } = useNodeStore();
  const [activeTab, setActiveTab] = useState<'preview' | 'monitor'>('preview');
  const [previewNodes, setPreviewNodes] = useState<PreviewNode[]>([]);
  const [selectedPreviewNode, setSelectedPreviewNode] = useState<string | null>(null);

  // Extract preview-capable nodes
  useEffect(() => {
    const videoNodes = nodes.filter(node => 
      node.type === 'input' || 
      node.type === 'effect' || 
      node.type === 'output'
    );

    const previewCapableNodes: PreviewNode[] = videoNodes.map(node => ({
      id: node.id,
      name: node.data.label || node.type,
      type: node.type,
      isActive: true // TODO: Check if node is actually processing
    }));

    setPreviewNodes(previewCapableNodes);

    const isSelectedNodeValid = selectedPreviewNode && previewCapableNodes.some(n => n.id === selectedPreviewNode);

    // Auto-select a node if the current selection is invalid or doesn't exist
    if (!isSelectedNodeValid) {
      if (previewCapableNodes.length > 0) {
        setSelectedPreviewNode(previewCapableNodes[0].id);
      } else {
        setSelectedPreviewNode(null);
      }
    }
  }, [nodes, selectedPreviewNode]);

  if (!isOpen) return null;

  return (
    <div className="preview-monitor-panel">
      <div className="panel-overlay" onClick={onClose} />
      
      <div className="panel-content">
        {/* Panel Header */}
        <div className="panel-header">
          <h2 className="panel-title">Preview & Monitor</h2>
          <button className="close-btn" onClick={onClose}>
            ✕
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="tab-nav">
          <button 
            className={`tab-btn ${activeTab === 'preview' ? 'active' : ''}`}
            onClick={() => setActiveTab('preview')}
          >
            📹 Video Preview
          </button>
          <button 
            className={`tab-btn ${activeTab === 'monitor' ? 'active' : ''}`}
            onClick={() => setActiveTab('monitor')}
          >
            📊 Performance Monitor
          </button>
        </div>

        {/* Tab Content */}
        <div className="tab-content">
          {activeTab === 'preview' && (
            <div className="preview-tab">
              {/* Node Selection */}
              <div className="node-selection">
                <h3 className="selection-title">Select Node to Preview</h3>
                <div className="node-list">
                  {previewNodes.map((node) => (
                    <div 
                      key={node.id}
                      className={`node-item ${selectedPreviewNode === node.id ? 'selected' : ''}`}
                      onClick={() => setSelectedPreviewNode(node.id)}
                    >
                      <div className="node-info">
                        <span className="node-name">{node.name}</span>
                        <span className="node-type">{node.type}</span>
                      </div>
                      <div className={`node-status ${node.isActive ? 'active' : 'inactive'}`}>
                        {node.isActive ? '🟢' : '🔴'}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Video Preview */}
              <div className="preview-container">
                {selectedPreviewNode ? (
                  <VideoPreview
                    nodeId={selectedPreviewNode}
                    width={640}
                    height={360}
                    showStats={true}
                    title={`Preview: ${previewNodes.find(n => n.id === selectedPreviewNode)?.name || 'Unknown'}`}
                  />
                ) : (
                  <div className="no-preview">
                    <p>No node selected for preview</p>
                  </div>
                )}
              </div>

              {/* Multiple Preview Support */}
              <div className="multi-preview-controls">
                <h4 className="controls-title">Multiple Preview</h4>
                <div className="preview-grid">
                  {previewNodes.slice(0, 4).map((node) => (
                    <div key={`mini-${node.id}`} className="mini-preview">
                      <VideoPreview
                        nodeId={node.id}
                        width={320}
                        height={180}
                        showStats={false}
                        title={node.name}
                      />
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {activeTab === 'monitor' && (
            <div className="monitor-tab">
              <PerformanceMonitor 
                updateInterval={1000}
                historyLength={60}
              />
            </div>
          )}
        </div>
      </div>

      <style jsx>{`
        .preview-monitor-panel {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          z-index: 1000;
          display: flex;
          align-items: center;
          justify-content: center;
        }

        .panel-overlay {
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.7);
          backdrop-filter: blur(5px);
        }

        .panel-content {
          position: relative;
          width: 90vw;
          max-width: 1200px;
          height: 85vh;
          max-height: 800px;
          background: #1a1a1a;
          border: 1px solid #333;
          border-radius: 12px;
          display: flex;
          flex-direction: column;
          overflow: hidden;
          box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
        }

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px 20px;
          background: #2a2a2a;
          border-bottom: 1px solid #333;
        }

        .panel-title {
          font-size: 18px;
          font-weight: 600;
          color: #fff;
          margin: 0;
        }

        .close-btn {
          background: none;
          border: none;
          color: #aaa;
          font-size: 20px;
          cursor: pointer;
          padding: 4px;
          transition: color 0.2s;
        }

        .close-btn:hover {
          color: #fff;
        }

        .tab-nav {
          display: flex;
          background: #222;
          border-bottom: 1px solid #333;
        }

        .tab-btn {
          flex: 1;
          padding: 12px 16px;
          background: none;
          border: none;
          color: #aaa;
          font-size: 14px;
          cursor: pointer;
          transition: all 0.2s;
          border-bottom: 2px solid transparent;
        }

        .tab-btn:hover {
          background: #2a2a2a;
          color: #fff;
        }

        .tab-btn.active {
          background: #2a2a2a;
          color: #fff;
          border-bottom-color: #4a9eff;
        }

        .tab-content {
          flex: 1;
          overflow-y: auto;
          padding: 20px;
        }

        .preview-tab {
          display: flex;
          flex-direction: column;
          gap: 20px;
        }

        .node-selection {
          background: #222;
          border: 1px solid #333;
          border-radius: 8px;
          padding: 16px;
        }

        .selection-title {
          font-size: 16px;
          font-weight: 600;
          color: #fff;
          margin: 0 0 12px 0;
        }

        .node-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .node-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 12px;
          background: #333;
          border: 1px solid #444;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .node-item:hover {
          background: #3a3a3a;
          border-color: #555;
        }

        .node-item.selected {
          background: #2a4a7a;
          border-color: #4a9eff;
        }

        .node-info {
          display: flex;
          flex-direction: column;
          gap: 2px;
        }

        .node-name {
          font-size: 14px;
          font-weight: 500;
          color: #fff;
        }

        .node-type {
          font-size: 12px;
          color: #aaa;
          text-transform: uppercase;
        }

        .node-status {
          font-size: 16px;
        }

        .preview-container {
          flex: 1;
          display: flex;
          justify-content: center;
          align-items: center;
          min-height: 300px;
        }

        .no-preview {
          text-align: center;
          color: #aaa;
          padding: 40px;
        }

        .multi-preview-controls {
          background: #222;
          border: 1px solid #333;
          border-radius: 8px;
          padding: 16px;
        }

        .controls-title {
          font-size: 16px;
          font-weight: 600;
          color: #fff;
          margin: 0 0 12px 0;
        }

        .preview-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
          gap: 16px;
        }

        .mini-preview {
          border: 1px solid #333;
          border-radius: 6px;
          overflow: hidden;
        }

        .monitor-tab {
          height: 100%;
          overflow-y: auto;
        }

        /* Responsive Design */
        @media (max-width: 768px) {
          .panel-content {
            width: 95vw;
            height: 90vh;
          }

          .preview-grid {
            grid-template-columns: 1fr;
          }

          .tab-btn {
            font-size: 12px;
            padding: 10px 12px;
          }
        }
      `}</style>
    </div>
  );
};

export default PreviewMonitorPanel;