import React, { useState, useEffect } from 'react';
import { useNodeStore } from '../stores/useNodeStore';
import { Sliders, Settings, X } from 'lucide-react';
import type { ParameterDefinition } from '../types';
import { useTheme, getThemeColors, getThemeStyles } from '../contexts/ThemeContext';
import { ControllerPreview } from './ControllerPreview';

interface ParameterPanelProps {
  selectedNodeId: string | null;
  onClose: () => void;
}

export const ParameterPanel: React.FC<ParameterPanelProps> = ({ selectedNodeId, onClose }) => {
  const { nodeProperties, setNodeParameters } = useNodeStore();
  const { isDark } = useTheme();
  const colors = getThemeColors(isDark);
  const styles = getThemeStyles(isDark);
  const [parameters, setParameters] = useState<Record<string, any>>({});
  const [isDirty, setIsDirty] = useState(false);

  const nodeProperty = selectedNodeId ? nodeProperties[selectedNodeId] : null;

  useEffect(() => {
    if (nodeProperty) {
      // Initialize parameters with default values
      const initialParams: Record<string, any> = {};
      Object.entries(nodeProperty.parameters || {}).forEach(([key, param]) => {
        initialParams[key] = param.defaultValue;
      });
      setParameters(initialParams);
      setIsDirty(false);
    }
  }, [nodeProperty]);

  const handleParameterChange = (paramName: string, value: any) => {
    setParameters(prev => ({
      ...prev,
      [paramName]: value
    }));
    setIsDirty(true);
  };

  const handleApply = async () => {
    if (selectedNodeId && isDirty) {
      try {
        await setNodeParameters(selectedNodeId, parameters);
        setIsDirty(false);
        console.log('✅ Parameters applied successfully');
      } catch (error) {
        console.error('❌ Failed to apply parameters:', error);
      }
    }
  };

  const handleReset = () => {
    if (nodeProperty) {
      const resetParams: Record<string, any> = {};
      Object.entries(nodeProperty.parameters || {}).forEach(([key, param]) => {
        resetParams[key] = param.defaultValue;
      });
      setParameters(resetParams);
      setIsDirty(false);
    }
  };

  const renderParameterInput = (paramName: string, param: ParameterDefinition) => {
    const value = parameters[paramName];
    
    const inputStyles = {
      width: '100%',
      padding: '0.5rem',
      border: `1px solid ${colors.border}`,
      borderRadius: '4px',
      fontSize: '0.875rem',
      background: colors.surface,
      color: colors.text,
    };

    // Special handling for controller parameters
    if (paramName === 'frequency' || paramName === 'amplitude' || paramName === 'time_scale' || paramName === 'speed') {
      return (
        <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
          <input
            type="range"
            min={param.minValue ? Number(param.minValue) : 0}
            max={param.maxValue ? Number(param.maxValue) : 1}
            step="0.01"
            value={value || 0}
            onChange={(e) => handleParameterChange(paramName, parseFloat(e.target.value) || 0)}
            style={{
              flex: 1,
              accentColor: colors.primary,
            }}
          />
          <input
            type="number"
            step="0.01"
            min={param.minValue ? Number(param.minValue) : undefined}
            max={param.maxValue ? Number(param.maxValue) : undefined}
            value={value || 0}
            onChange={(e) => handleParameterChange(paramName, parseFloat(e.target.value) || 0)}
            style={{
              width: '80px',
              padding: '0.25rem',
              border: `1px solid ${colors.border}`,
              borderRadius: '4px',
              fontSize: '0.75rem',
              background: colors.surface,
              color: colors.text,
            }}
          />
        </div>
      );
    }

    if (paramName === 'phase' || paramName === 'offset') {
      return (
        <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
          <input
            type="range"
            min={param.minValue ? Number(param.minValue) : -1}
            max={param.maxValue ? Number(param.maxValue) : 1}
            step="0.01"
            value={value || 0}
            onChange={(e) => handleParameterChange(paramName, parseFloat(e.target.value) || 0)}
            style={{
              flex: 1,
              accentColor: colors.primary,
            }}
          />
          <span style={{ 
            width: '50px', 
            fontSize: '0.75rem', 
            textAlign: 'right',
            color: colors.textSecondary,
          }}>
            {(value || 0).toFixed(2)}
          </span>
        </div>
      );
    }

    switch (param.parameterType) {
      case 'Float':
        return (
          <input
            type="number"
            step="0.01"
            min={param.minValue ? Number(param.minValue) : undefined}
            max={param.maxValue ? Number(param.maxValue) : undefined}
            value={value || 0}
            onChange={(e) => handleParameterChange(paramName, parseFloat(e.target.value) || 0)}
            style={inputStyles}
          />
        );

      case 'Integer':
        return (
          <input
            type="number"
            step="1"
            min={param.minValue ? Number(param.minValue) : undefined}
            max={param.maxValue ? Number(param.maxValue) : undefined}
            value={value || 0}
            onChange={(e) => handleParameterChange(paramName, parseInt(e.target.value) || 0)}
            style={inputStyles}
          />
        );

      case 'Boolean':
        return (
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={value || false}
              onChange={(e) => handleParameterChange(paramName, e.target.checked)}
              style={{ marginRight: '0.5rem' }}
            />
            <span style={{ fontSize: '0.875rem' }}>
              {value ? 'Enabled' : 'Disabled'}
            </span>
          </label>
        );

      case 'String':
        return (
          <input
            type="text"
            value={value || ''}
            onChange={(e) => handleParameterChange(paramName, e.target.value)}
            style={inputStyles}
          />
        );

      case 'Color':
        return (
          <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
            <input
              type="color"
              value={value || '#ffffff'}
              onChange={(e) => handleParameterChange(paramName, e.target.value)}
              style={{
                width: '3rem',
                height: '2rem',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            />
            <input
              type="text"
              value={value || '#ffffff'}
              onChange={(e) => handleParameterChange(paramName, e.target.value)}
              style={{
                flex: 1,
                padding: '0.5rem',
                border: '1px solid #ddd',
                borderRadius: '4px',
                fontSize: '0.875rem',
              }}
            />
          </div>
        );

      default:
        if (typeof param.parameterType === 'object' && 'Enum' in param.parameterType) {
          return (
            <select
              value={value || param.parameterType.Enum[0]}
              onChange={(e) => handleParameterChange(paramName, e.target.value)}
              style={{
                width: '100%',
                padding: '0.5rem',
                border: `1px solid ${colors.border}`,
                borderRadius: '4px',
                fontSize: '0.875rem',
                background: colors.surface,
                color: colors.text,
              }}
            >
              {param.parameterType.Enum.map((option) => (
                <option key={option} value={option}>
                  {option}
                </option>
              ))}
            </select>
          );
        }
        return (
          <input
            type="text"
            value={value || ''}
            onChange={(e) => handleParameterChange(paramName, e.target.value)}
            style={inputStyles}
          />
        );
    }
  };

  if (!selectedNodeId || !nodeProperty) {
    return (
      <div style={{
        width: '300px',
        background: '#f8f9fa',
        borderLeft: '1px solid #dee2e6',
        padding: '1rem',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        color: '#6c757d',
      }}>
        <Settings size={48} style={{ marginBottom: '1rem', opacity: 0.5 }} />
        <p style={{ textAlign: 'center', margin: 0 }}>
          Select a node to edit its parameters
        </p>
      </div>
    );
  }

  return (
    <div style={{
      width: '320px',
      background: colors.background,
      borderLeft: `2px solid ${colors.borderLight}`,
      display: 'flex',
      flexDirection: 'column',
      height: '100%',
      boxShadow: styles.shadowLight,
    }}>
      {/* Header */}
      <div style={{
        padding: '1rem',
        borderBottom: '1px solid #dee2e6',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        background: styles.headerGradient,
        color: '#ffffff',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <Sliders size={20} />
          <h3 style={{ margin: 0, fontSize: '1rem', fontWeight: '600', color: 'inherit' }}>
            Parameters
          </h3>
        </div>
        <button
          onClick={onClose}
          style={{
            background: 'none',
            border: 'none',
            cursor: 'pointer',
            padding: '0.25rem',
            borderRadius: '4px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <X size={16} />
        </button>
      </div>

      {/* Node Info */}
      <div style={{
        padding: '1rem',
        borderBottom: '1px solid #dee2e6',
        background: '#ffffff',
      }}>
        <h4 style={{ 
          margin: '0 0 0.5rem 0', 
          fontSize: '0.875rem', 
          fontWeight: '600',
          color: '#2c3e50',
          letterSpacing: '0.3px',
        }}>
          {nodeProperty.name}
        </h4>
        <p style={{ margin: 0, fontSize: '0.75rem', color: '#8e9aaf' }}>
          ID: {nodeProperty.id}
        </p>
        <p style={{ margin: '0.25rem 0 0 0', fontSize: '0.75rem', color: '#8e9aaf' }}>
          Type: {JSON.stringify(nodeProperty.nodeType)}
        </p>
      </div>

      {/* Parameters */}
      <div style={{
        flex: 1,
        padding: '1rem',
        overflowY: 'auto',
        background: '#ffffff',
      }}>
        {Object.entries(nodeProperty.parameters || {}).length === 0 ? (
          <p style={{ color: '#6c757d', fontStyle: 'italic', margin: 0 }}>
            No parameters available for this node.
          </p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            {Object.entries(nodeProperty.parameters || {}).map(([paramName, param]) => (
              <div key={paramName}>
                <label style={{
                  display: 'block',
                  marginBottom: '0.5rem',
                  fontSize: '0.875rem',
                  fontWeight: '500',
                  color: '#2c3e50',
                }}>
                  {param.name}
                </label>
                {renderParameterInput(paramName, param)}
                {param.description && (
                  <p style={{
                    margin: '0.25rem 0 0 0',
                    fontSize: '0.75rem',
                    color: '#6c757d',
                    fontStyle: 'italic',
                  }}>
                    {param.description}
                  </p>
                )}
              </div>
            ))}
          </div>
        )}

        {/* Controller Preview */}
        {nodeProperty.nodeType && 'Control' in nodeProperty.nodeType && (
          ['LFO', 'Timeline', 'MathController'].includes(nodeProperty.nodeType.Control)
        ) && (
          <ControllerPreview
            controllerType={nodeProperty.nodeType.Control as 'LFO' | 'Timeline' | 'MathController'}
            parameters={parameters}
          />
        )}
      </div>

      {/* Actions */}
      <div style={{
        padding: '1rem',
        borderTop: `1px solid ${colors.borderLight}`,
        display: 'flex',
        gap: '0.5rem',
        background: colors.backgroundSecondary,
      }}>
        <button
          onClick={handleApply}
          disabled={!isDirty}
          style={{
            flex: 1,
            padding: '0.6rem 1.2rem',
            background: isDirty ? styles.buttonSuccess : '#6c757d',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            fontSize: '0.875rem',
            cursor: isDirty ? 'pointer' : 'not-allowed',
            fontWeight: 'bold',
          }}
        >
          Apply Changes
        </button>
        <button
          onClick={handleReset}
          style={{
            padding: '0.6rem 1rem',
            background: isDark ? 'linear-gradient(135deg, #4a5568 0%, #2d3748 100%)' : 'linear-gradient(135deg, #6c757d 0%, #495057 100%)',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            fontSize: '0.875rem',
            cursor: 'pointer',
          }}
        >
          Reset
        </button>
      </div>

      {/* Status */}
      {isDirty && (
        <div style={{
          padding: '0.5rem 1rem',
          background: isDark 
            ? 'linear-gradient(135deg, #744210 0%, #975a16 100%)'
            : 'linear-gradient(135deg, #fff3cd 0%, #ffeaa7 100%)',
          color: isDark ? '#fbb6ce' : '#856404',
          borderTop: `2px solid ${isDark ? '#975a16' : '#ffeaa7'}`,
          fontSize: '0.75rem',
        }}>
          ⚠️ You have unsaved changes
        </div>
      )}
    </div>
  );
};