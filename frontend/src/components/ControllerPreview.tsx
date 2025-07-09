import React, { useState, useEffect, useRef } from 'react';
import { useTheme, getThemeColors } from '../contexts/ThemeContext';

interface ControllerPreviewProps {
  controllerType: 'LFO' | 'Timeline' | 'MathController';
  parameters: Record<string, any>;
  width?: number;
  height?: number;
}

export const ControllerPreview: React.FC<ControllerPreviewProps> = ({
  controllerType,
  parameters,
  width = 200,
  height = 100,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>();
  const startTimeRef = useRef<number>(Date.now());
  const { isDark } = useTheme();
  const colors = getThemeColors(isDark);

  const [isPlaying, setIsPlaying] = useState(true);

  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const animate = () => {
      if (!isPlaying) return;

      const now = Date.now();
      const elapsed = (now - startTimeRef.current) / 1000;

      ctx.clearRect(0, 0, width, height);

      // Draw background
      ctx.fillStyle = colors.surface;
      ctx.fillRect(0, 0, width, height);

      // Draw grid
      ctx.strokeStyle = colors.border;
      ctx.lineWidth = 1;
      for (let i = 0; i <= 10; i++) {
        const x = (i / 10) * width;
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, height);
        ctx.stroke();
      }
      for (let i = 0; i <= 5; i++) {
        const y = (i / 5) * height;
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(width, y);
        ctx.stroke();
      }

      // Draw center line
      ctx.strokeStyle = colors.textSecondary;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, height / 2);
      ctx.lineTo(width, height / 2);
      ctx.stroke();

      // Draw waveform
      ctx.strokeStyle = colors.primary;
      ctx.lineWidth = 2;
      ctx.beginPath();

      const timeWindow = 2; // Show 2 seconds of waveform
      const pointCount = width;

      for (let i = 0; i < pointCount; i++) {
        const t = (i / pointCount) * timeWindow;
        const x = i;
        let y = height / 2;

        if (controllerType === 'LFO') {
          y = calculateLFOValue(t + elapsed, parameters);
        } else if (controllerType === 'MathController') {
          y = calculateMathValue(t + elapsed, parameters);
        } else if (controllerType === 'Timeline') {
          y = calculateTimelineValue(t + elapsed, parameters);
        }

        // Convert from [-1, 1] to canvas coordinates
        y = height / 2 - (y * height / 2);

        if (i === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      }

      ctx.stroke();

      // Draw current value indicator
      const currentValue = controllerType === 'LFO' 
        ? calculateLFOValue(elapsed, parameters)
        : controllerType === 'MathController'
        ? calculateMathValue(elapsed, parameters)
        : calculateTimelineValue(elapsed, parameters);

      const currentY = height / 2 - (currentValue * height / 2);
      ctx.fillStyle = colors.primary;
      ctx.beginPath();
      ctx.arc(width - 10, currentY, 4, 0, 2 * Math.PI);
      ctx.fill();

      // Draw value text
      ctx.fillStyle = colors.text;
      ctx.font = '12px monospace';
      ctx.textAlign = 'right';
      ctx.fillText(currentValue.toFixed(3), width - 15, currentY - 10);

      animationRef.current = requestAnimationFrame(animate);
    };

    if (isPlaying) {
      animate();
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [controllerType, parameters, isPlaying, colors, width, height]);

  const calculateLFOValue = (time: number, params: Record<string, any>) => {
    const frequency = params.frequency || 1.0;
    const amplitude = params.amplitude || 1.0;
    const offset = params.offset || 0.0;
    const phase = params.phase || 0.0;
    const waveform = params.waveform || 'Sine';

    const phaseAdjustedTime = time + (phase * (1.0 / frequency));
    let baseValue = 0;

    switch (waveform) {
      case 'Sine':
        baseValue = Math.sin(phaseAdjustedTime * frequency * 2 * Math.PI);
        break;
      case 'Square':
        const sqPhase = (phaseAdjustedTime * frequency) % 1.0;
        baseValue = sqPhase < 0.5 ? 1.0 : -1.0;
        break;
      case 'Triangle':
        const triPhase = (phaseAdjustedTime * frequency) % 1.0;
        baseValue = triPhase < 0.5 ? 4.0 * triPhase - 1.0 : 3.0 - 4.0 * triPhase;
        break;
      case 'Sawtooth':
        const sawPhase = (phaseAdjustedTime * frequency) % 1.0;
        baseValue = 2.0 * sawPhase - 1.0;
        break;
      case 'Noise':
        baseValue = Math.random() * 2.0 - 1.0;
        break;
      default:
        baseValue = Math.sin(phaseAdjustedTime * frequency * 2 * Math.PI);
    }

    return Math.max(-1.0, Math.min(1.0, baseValue * amplitude + offset));
  };

  const calculateMathValue = (time: number, params: Record<string, any>) => {
    const expression = params.expression || 'sin(t)';
    const timeScale = params.time_scale || 1.0;
    const a = params.var_a || 1.0;
    const b = params.var_b || 0.0;
    const c = params.var_c || 0.0;
    const scaledTime = time * timeScale;

    // Simple expression evaluation
    try {
      switch (expression) {
        case 'sin(t)':
          return Math.sin(scaledTime * Math.PI);
        case 'cos(t)':
          return Math.cos(scaledTime * Math.PI);
        case 'sin(t * a)':
          return Math.sin(scaledTime * a * Math.PI);
        case 'cos(t * a)':
          return Math.cos(scaledTime * a * Math.PI);
        case 'a * sin(t) + b':
          return a * Math.sin(scaledTime * Math.PI) + b;
        case 'a * cos(t) + b':
          return a * Math.cos(scaledTime * Math.PI) + b;
        case 'abs(sin(t))':
          return Math.abs(Math.sin(scaledTime * Math.PI));
        case 't':
          return scaledTime % 2.0 - 1.0;
        case 'a':
          return a;
        case 'b':
          return b;
        case 'c':
          return c;
        default:
          return parseFloat(expression) || 0.0;
      }
    } catch {
      return 0.0;
    }
  };

  const calculateTimelineValue = (time: number, params: Record<string, any>) => {
    const duration = params.duration || 10.0;
    const loop = params.loop || true;
    const speed = params.speed || 1.0;
    const play = params.play || false;

    if (!play) return 0.0;

    let currentTime = time * speed;
    if (loop) {
      currentTime = currentTime % duration;
    } else if (currentTime > duration) {
      currentTime = duration;
    }

    // Simple linear interpolation between 0 and 1
    return (currentTime / duration) * 2.0 - 1.0;
  };

  return (
    <div style={{ marginTop: '1rem' }}>
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '0.5rem',
      }}>
        <span style={{
          fontSize: '0.875rem',
          color: colors.text,
          fontWeight: 'bold',
        }}>
          Preview
        </span>
        <button
          onClick={() => setIsPlaying(!isPlaying)}
          style={{
            padding: '0.25rem 0.5rem',
            fontSize: '0.75rem',
            background: colors.primary,
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {isPlaying ? 'Pause' : 'Play'}
        </button>
      </div>
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        style={{
          border: `1px solid ${colors.border}`,
          borderRadius: '4px',
          display: 'block',
          width: '100%',
        }}
      />
    </div>
  );
};