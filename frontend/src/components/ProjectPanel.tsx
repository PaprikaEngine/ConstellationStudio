import React, { useState } from 'react';
import { Save, FolderOpen, Download, Upload, FileText, AlertCircle, CheckCircle, Copy, Trash2, X } from 'lucide-react';
import { useTheme, getThemeColors, getThemeStyles } from '../contexts/ThemeContext';
import { useNodeStore } from '../stores/useNodeStore';
import { ProjectManager } from '../utils/projectManager';
import { ProjectConfiguration, ProjectSaveOptions } from '../types/project';
import { useNotifications } from './NotificationSystem';

interface ProjectPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ProjectPanel: React.FC<ProjectPanelProps> = ({ isOpen, onClose }) => {
  const { isDark } = useTheme();
  const colors = getThemeColors(isDark);
  const styles = getThemeStyles(isDark);
  const { nodes, edges, viewport, setNodesAndEdges } = useNodeStore();
  const notifications = useNotifications();

  const [projectName, setProjectName] = useState('');
  const [projectDescription, setProjectDescription] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  if (!isOpen) return null;

  const handleSaveToFile = () => {
    try {
      const options: ProjectSaveOptions = {
        name: projectName || undefined,
        description: projectDescription || undefined,
      };

      const config = ProjectManager.exportConfiguration(nodes, edges, viewport, options);
      ProjectManager.saveToFile(config);
      
      notifications.success(
        'Project Saved',
        `Project "${config.name}" has been saved to file`
      );
    } catch (error) {
      notifications.error(
        'Save Failed',
        `Failed to save project: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  };

  const handleLoadFromFile = async () => {
    setIsLoading(true);
    try {
      const result = await ProjectManager.loadFromFile();
      
      if (result.success && result.configuration) {
        // Apply the loaded configuration
        setNodesAndEdges(result.configuration.nodes, result.configuration.edges);
        
        // Update form fields
        setProjectName(result.configuration.name);
        setProjectDescription(result.configuration.description || '');
        
        notifications.success(
          'Project Loaded',
          `Project "${result.configuration.name}" has been loaded successfully`
        );
        
        if (result.warnings && result.warnings.length > 0) {
          notifications.warning(
            'Load Warnings',
            `${result.warnings.length} warnings occurred during load`
          );
        }
      } else {
        notifications.error(
          'Load Failed',
          result.error || 'Unknown error occurred'
        );
      }
    } catch (error) {
      notifications.error(
        'Load Failed',
        `Failed to load project: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleSaveToLocalStorage = () => {
    try {
      const options: ProjectSaveOptions = {
        name: projectName || undefined,
        description: projectDescription || undefined,
      };

      const config = ProjectManager.exportConfiguration(nodes, edges, viewport, options);
      const success = ProjectManager.saveToLocalStorage(config);
      
      if (success) {
        notifications.success(
          'Auto-Save',
          'Project has been saved to browser storage'
        );
      } else {
        notifications.error(
          'Auto-Save Failed',
          'Failed to save to browser storage'
        );
      }
    } catch (error) {
      notifications.error(
        'Auto-Save Failed',
        `Failed to auto-save: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  };

  const handleLoadFromLocalStorage = () => {
    try {
      const result = ProjectManager.loadFromLocalStorage();
      
      if (result.success && result.configuration) {
        setNodesAndEdges(result.configuration.nodes, result.configuration.edges);
        setProjectName(result.configuration.name);
        setProjectDescription(result.configuration.description || '');
        
        notifications.success(
          'Auto-Load',
          'Project has been loaded from browser storage'
        );
      } else {
        notifications.error(
          'Auto-Load Failed',
          result.error || 'No saved project found'
        );
      }
    } catch (error) {
      notifications.error(
        'Auto-Load Failed',
        `Failed to auto-load: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  };

  const handleExportJSON = () => {
    try {
      const options: ProjectSaveOptions = {
        name: projectName || undefined,
        description: projectDescription || undefined,
      };

      const config = ProjectManager.exportConfiguration(nodes, edges, viewport, options);
      const jsonString = JSON.stringify(config, null, 2);
      
      navigator.clipboard.writeText(jsonString).then(() => {
        notifications.success(
          'JSON Exported',
          'Project configuration copied to clipboard'
        );
      }).catch(() => {
        // Fallback for older browsers
        const textarea = document.createElement('textarea');
        textarea.value = jsonString;
        document.body.appendChild(textarea);
        textarea.select();
        document.execCommand('copy');
        document.body.removeChild(textarea);
        
        notifications.success(
          'JSON Exported',
          'Project configuration copied to clipboard'
        );
      });
    } catch (error) {
      notifications.error(
        'Export Failed',
        `Failed to export JSON: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  };

  const handleLoadSample = () => {
    try {
      const sampleConfig = ProjectManager.generateSampleConfiguration();
      setNodesAndEdges(sampleConfig.nodes, sampleConfig.edges);
      setProjectName(sampleConfig.name);
      setProjectDescription(sampleConfig.description || '');
      
      notifications.success(
        'Sample Loaded',
        'Sample project configuration has been loaded'
      );
    } catch (error) {
      notifications.error(
        'Sample Load Failed',
        `Failed to load sample: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  };

  const handleClearProject = () => {
    if (window.confirm('Are you sure you want to clear the current project? All unsaved changes will be lost.')) {
      setNodesAndEdges([], []);
      setProjectName('');
      setProjectDescription('');
      
      notifications.info(
        'Project Cleared',
        'Current project has been cleared'
      );
    }
  };

  return (
    <>
      {/* Backdrop */}
      <div
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0, 0, 0, 0.5)',
          zIndex: 9998,
          backdropFilter: 'blur(4px)',
        }}
        onClick={onClose}
      />

      {/* Project Panel */}
      <div
        style={{
          position: 'fixed',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          width: '580px',
          maxWidth: '90vw',
          maxHeight: '80vh',
          background: colors.background,
          borderRadius: '16px',
          boxShadow: styles.shadowHeavy,
          border: `2px solid ${colors.borderLight}`,
          zIndex: 9999,
          overflow: 'hidden',
        }}
      >
        {/* Header */}
        <div
          style={{
            background: styles.headerGradient,
            color: '#ffffff',
            padding: '1.5rem',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
            <FolderOpen size={24} />
            <h2 style={{
              margin: 0,
              fontSize: '1.25rem',
              fontWeight: '600',
              letterSpacing: '0.3px',
            }}>
              Project Manager
            </h2>
          </div>
          
          <button
            onClick={onClose}
            style={{
              background: 'rgba(255, 255, 255, 0.2)',
              border: '1px solid rgba(255, 255, 255, 0.3)',
              borderRadius: '8px',
              color: '#ffffff',
              cursor: 'pointer',
              padding: '0.5rem',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.2s ease',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'rgba(255, 255, 255, 0.3)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'rgba(255, 255, 255, 0.2)';
            }}
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div style={{ padding: '2rem', overflowY: 'auto', maxHeight: 'calc(80vh - 120px)' }}>
          {/* Project Info */}
          <div style={{ marginBottom: '2rem' }}>
            <h3 style={{
              margin: '0 0 1rem 0',
              fontSize: '1rem',
              fontWeight: '600',
              color: colors.text,
              borderBottom: `2px solid ${colors.borderLight}`,
              paddingBottom: '0.5rem',
            }}>
              Project Information
            </h3>
            
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
              <div>
                <label style={{
                  display: 'block',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  color: colors.text,
                  marginBottom: '0.5rem',
                }}>
                  Project Name
                </label>
                <input
                  type="text"
                  value={projectName}
                  onChange={(e) => setProjectName(e.target.value)}
                  placeholder="Enter project name..."
                  style={{
                    width: '100%',
                    padding: '0.75rem',
                    border: `1px solid ${colors.border}`,
                    borderRadius: '6px',
                    fontSize: '0.875rem',
                    background: colors.surface,
                    color: colors.text,
                  }}
                />
              </div>
              
              <div>
                <label style={{
                  display: 'block',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  color: colors.text,
                  marginBottom: '0.5rem',
                }}>
                  Description
                </label>
                <textarea
                  value={projectDescription}
                  onChange={(e) => setProjectDescription(e.target.value)}
                  placeholder="Enter project description..."
                  rows={3}
                  style={{
                    width: '100%',
                    padding: '0.75rem',
                    border: `1px solid ${colors.border}`,
                    borderRadius: '6px',
                    fontSize: '0.875rem',
                    background: colors.surface,
                    color: colors.text,
                    resize: 'vertical',
                  }}
                />
              </div>
            </div>
          </div>

          {/* File Operations */}
          <div style={{ marginBottom: '2rem' }}>
            <h3 style={{
              margin: '0 0 1rem 0',
              fontSize: '1rem',
              fontWeight: '600',
              color: colors.text,
              borderBottom: `2px solid ${colors.borderLight}`,
              paddingBottom: '0.5rem',
            }}>
              File Operations
            </h3>
            
            <div style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '0.75rem',
            }}>
              <button
                onClick={handleSaveToFile}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                  padding: '0.75rem',
                  background: styles.buttonSuccess,
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-1px)';
                  e.currentTarget.style.boxShadow = styles.shadowMedium;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.boxShadow = 'none';
                }}
              >
                <Download size={16} />
                Save to File
              </button>
              
              <button
                onClick={handleLoadFromFile}
                disabled={isLoading}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                  padding: '0.75rem',
                  background: styles.buttonPrimary,
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  cursor: isLoading ? 'not-allowed' : 'pointer',
                  opacity: isLoading ? 0.6 : 1,
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  if (!isLoading) {
                    e.currentTarget.style.transform = 'translateY(-1px)';
                    e.currentTarget.style.boxShadow = styles.shadowMedium;
                  }
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.boxShadow = 'none';
                }}
              >
                <Upload size={16} />
                {isLoading ? 'Loading...' : 'Load from File'}
              </button>
            </div>
          </div>

          {/* Browser Storage */}
          <div style={{ marginBottom: '2rem' }}>
            <h3 style={{
              margin: '0 0 1rem 0',
              fontSize: '1rem',
              fontWeight: '600',
              color: colors.text,
              borderBottom: `2px solid ${colors.borderLight}`,
              paddingBottom: '0.5rem',
            }}>
              Browser Storage
            </h3>
            
            <div style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '0.75rem',
            }}>
              <button
                onClick={handleSaveToLocalStorage}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                  padding: '0.75rem',
                  background: isDark ? '#4a5568' : '#6c757d',
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-1px)';
                  e.currentTarget.style.opacity = '0.9';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.opacity = '1';
                }}
              >
                <Save size={16} />
                Auto-Save
              </button>
              
              <button
                onClick={handleLoadFromLocalStorage}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                  padding: '0.75rem',
                  background: isDark ? '#4a5568' : '#6c757d',
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-1px)';
                  e.currentTarget.style.opacity = '0.9';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.opacity = '1';
                }}
              >
                <FolderOpen size={16} />
                Auto-Load
              </button>
            </div>
          </div>

          {/* Utilities */}
          <div>
            <h3 style={{
              margin: '0 0 1rem 0',
              fontSize: '1rem',
              fontWeight: '600',
              color: colors.text,
              borderBottom: `2px solid ${colors.borderLight}`,
              paddingBottom: '0.5rem',
            }}>
              Utilities
            </h3>
            
            <div style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '0.75rem',
            }}>
              <button
                onClick={handleExportJSON}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                  padding: '0.75rem',
                  background: colors.info,
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-1px)';
                  e.currentTarget.style.opacity = '0.9';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.opacity = '1';
                }}
              >
                <Copy size={16} />
                Copy JSON
              </button>
              
              <button
                onClick={handleLoadSample}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                  padding: '0.75rem',
                  background: colors.warning,
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-1px)';
                  e.currentTarget.style.opacity = '0.9';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.opacity = '1';
                }}
              >
                <FileText size={16} />
                Load Sample
              </button>
              
              <button
                onClick={handleClearProject}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                  padding: '0.75rem',
                  background: colors.error,
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                  gridColumn: 'span 2',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-1px)';
                  e.currentTarget.style.opacity = '0.9';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.opacity = '1';
                }}
              >
                <Trash2 size={16} />
                Clear Project
              </button>
            </div>
          </div>

          {/* Info */}
          <div style={{
            marginTop: '2rem',
            background: colors.backgroundSecondary,
            borderRadius: '8px',
            padding: '1rem',
            border: `1px solid ${colors.borderLight}`,
          }}>
            <div style={{
              fontSize: '0.75rem',
              color: colors.textMuted,
              lineHeight: '1.4',
            }}>
              <div style={{ fontWeight: '600', marginBottom: '0.5rem' }}>Current Project Status:</div>
              <div>Nodes: {nodes.length}</div>
              <div>Connections: {edges.length}</div>
              <div style={{ marginTop: '0.5rem', fontSize: '0.7rem' }}>
                Files are saved as JSON with .json extension. Browser storage is temporary and may be cleared.
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
};