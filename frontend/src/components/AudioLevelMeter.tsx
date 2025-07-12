import React, { useEffect, useState, useRef, useCallback } from 'react';

interface AudioLevelData {
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

interface AudioLevelMeterProps {
  nodeId: string;
  width?: number;
  height?: number;
  showLabels?: boolean;
  showValues?: boolean;
  updateInterval?: number;
  mode?: 'mono' | 'stereo';
  className?: string;
}

export const AudioLevelMeter: React.FC<AudioLevelMeterProps> = ({
  nodeId,
  width = 20,
  height = 80,
  showLabels = true,
  showValues = false,
  updateInterval = 50, // 20fps
  mode = 'mono',
  className = ''
}) => {
  const [audioLevel, setAudioLevel] = useState<AudioLevelData | null>(null);
  const [peakHold, setPeakHold] = useState({ left: 0, right: 0 });
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationFrameRef = useRef<number>();
  const lastUpdateRef = useRef<number>(0);

  // Get audio level data from API
  const fetchAudioLevel = useCallback(async () => {
    try {
      const response = await fetch(`/api/nodes/${nodeId}/audio/level`);
      if (response.ok) {
        const data = await response.json();
        setAudioLevel(data);
        
        // Update peak hold
        setPeakHold(prev => ({
          left: Math.max(prev.left, data.peak_left),
          right: Math.max(prev.right, data.peak_right)
        }));
      }
    } catch (error) {
      console.error('Failed to fetch audio level:', error);
    }
  }, [nodeId]);

  // Convert linear value to meter position (0-1)
  const linearToMeterPosition = useCallback((linearValue: number): number => {
    // Clamp to reasonable range and convert to 0-1 scale
    const clamped = Math.max(0, Math.min(1.2, linearValue));
    return clamped / 1.2;
  }, []);

  // Convert dB to meter position for display
  const dbToMeterPosition = useCallback((dbValue: number): number => {
    // Map -60dB to 0dB range to 0-1 scale
    const minDb = -60;
    const maxDb = 0;
    const clamped = Math.max(minDb, Math.min(maxDb + 6, dbValue)); // Allow +6dB for clipping
    return (clamped - minDb) / (maxDb + 6 - minDb);
  }, []);

  // Get color for audio level
  const getLevelColor = useCallback((dbValue: number, isClipping: boolean): string => {
    if (isClipping || dbValue > 0) {
      return '#ff0000'; // Red for clipping
    } else if (dbValue > -6) {
      return '#ff6600'; // Orange for hot levels
    } else if (dbValue > -18) {
      return '#ffff00'; // Yellow for moderate levels
    } else {
      return '#00ff00'; // Green for safe levels
    }
  }, []);

  // Draw the meter
  const drawMeter = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas || !audioLevel) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const meterWidth = mode === 'stereo' ? width / 2 - 1 : width;
    const meterHeight = height - (showLabels ? 20 : 0);
    const startY = showLabels ? 15 : 0;

    // Calculate levels
    const leftRmsPos = dbToMeterPosition(audioLevel.db_rms_left);
    const leftPeakPos = dbToMeterPosition(audioLevel.db_peak_left);
    const rightRmsPos = mode === 'stereo' ? dbToMeterPosition(audioLevel.db_rms_right) : leftRmsPos;
    const rightPeakPos = mode === 'stereo' ? dbToMeterPosition(audioLevel.db_peak_right) : leftPeakPos;

    // Draw meters
    const drawSingleMeter = (x: number, rmsPos: number, peakPos: number, dbRms: number, dbPeak: number) => {
      // Background
      ctx.fillStyle = '#333333';
      ctx.fillRect(x, startY, meterWidth, meterHeight);

      // RMS level (filled from bottom)
      const rmsHeight = meterHeight * rmsPos;
      const rmsY = startY + meterHeight - rmsHeight;
      
      ctx.fillStyle = getLevelColor(dbRms, audioLevel.is_clipping);
      ctx.fillRect(x, rmsY, meterWidth, rmsHeight);

      // Peak level (thin line)
      const peakY = startY + meterHeight - (meterHeight * peakPos);
      ctx.fillStyle = getLevelColor(dbPeak, audioLevel.is_clipping);
      ctx.fillRect(x, peakY - 1, meterWidth, 2);

      // Peak hold (thin line)
      const holdPos = mode === 'mono' 
        ? Math.max(peakHold.left, peakHold.right) 
        : (x === 0 ? peakHold.left : peakHold.right);
      const holdY = startY + meterHeight - (meterHeight * dbToMeterPosition(20 * Math.log10(holdPos)));
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(x, holdY, meterWidth, 1);

      // Scale marks (optional)
      if (showLabels && meterWidth >= 15) {
        ctx.fillStyle = '#666666';
        ctx.font = '8px monospace';
        ctx.textAlign = 'right';
        
        // 0dB mark
        const zeroDbY = startY + meterHeight - (meterHeight * dbToMeterPosition(0));
        ctx.fillRect(x + meterWidth - 2, zeroDbY, 2, 1);
        
        // -6dB mark
        const sixDbY = startY + meterHeight - (meterHeight * dbToMeterPosition(-6));
        ctx.fillRect(x + meterWidth - 2, sixDbY, 2, 1);
        
        // -18dB mark
        const eighteenDbY = startY + meterHeight - (meterHeight * dbToMeterPosition(-18));
        ctx.fillRect(x + meterWidth - 2, eighteenDbY, 2, 1);
      }

      // Border
      ctx.strokeStyle = '#555555';
      ctx.lineWidth = 1;
      ctx.strokeRect(x, startY, meterWidth, meterHeight);
    };

    if (mode === 'stereo') {
      // Draw left channel
      drawSingleMeter(0, leftRmsPos, leftPeakPos, audioLevel.db_rms_left, audioLevel.db_peak_left);
      // Draw right channel
      drawSingleMeter(meterWidth + 2, rightRmsPos, rightPeakPos, audioLevel.db_rms_right, audioLevel.db_peak_right);
    } else {
      // Draw mono (averaged) channel
      const monoRmsPos = (leftRmsPos + rightRmsPos) / 2;
      const monoPeakPos = (leftPeakPos + rightPeakPos) / 2;
      const monoDbRms = (audioLevel.db_rms_left + audioLevel.db_rms_right) / 2;
      const monoDbPeak = (audioLevel.db_peak_left + audioLevel.db_peak_right) / 2;
      drawSingleMeter(0, monoRmsPos, monoPeakPos, monoDbRms, monoDbPeak);
    }

    // Labels
    if (showLabels) {
      ctx.fillStyle = '#cccccc';
      ctx.font = '10px Arial';
      ctx.textAlign = 'center';
      
      if (mode === 'stereo' && width >= 40) {
        ctx.fillText('L', meterWidth / 2, 10);
        ctx.fillText('R', meterWidth + 2 + meterWidth / 2, 10);
      } else {
        ctx.fillText('🔊', width / 2, 10);
      }
    }

    // Values display
    if (showValues && audioLevel) {
      ctx.fillStyle = audioLevel.is_clipping ? '#ff0000' : '#cccccc';
      ctx.font = '8px monospace';
      ctx.textAlign = 'center';
      
      const monoDbRms = (audioLevel.db_rms_left + audioLevel.db_rms_right) / 2;
      const valueText = audioLevel.is_clipping ? 'CLIP' : 
                      Math.abs(monoDbRms) === Infinity ? '-∞' :
                      monoDbRms.toFixed(1);
      
      ctx.fillText(valueText, width / 2, height - 2);
    }
  }, [audioLevel, peakHold, width, height, mode, showLabels, showValues, dbToMeterPosition, getLevelColor]);

  // Animation loop
  const animate = useCallback(() => {
    const now = Date.now();
    if (now - lastUpdateRef.current >= updateInterval) {
      fetchAudioLevel();
      lastUpdateRef.current = now;
    }
    
    drawMeter();
    animationFrameRef.current = requestAnimationFrame(animate);
  }, [fetchAudioLevel, drawMeter, updateInterval]);

  // Peak hold decay
  useEffect(() => {
    const interval = setInterval(() => {
      setPeakHold(prev => ({
        left: Math.max(0, prev.left * 0.95), // Decay peak hold
        right: Math.max(0, prev.right * 0.95)
      }));
    }, 100);

    return () => clearInterval(interval);
  }, []);

  // Start/stop animation
  useEffect(() => {
    animationFrameRef.current = requestAnimationFrame(animate);
    
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [animate]);

  // Canvas setup
  useEffect(() => {
    const canvas = canvasRef.current;
    if (canvas) {
      canvas.width = width;
      canvas.height = height;
    }
  }, [width, height]);

  return (
    <div className={`audio-level-meter ${className}`} style={{ 
      width: `${width}px`, 
      height: `${height}px`,
      display: 'inline-block',
      position: 'relative'
    }}>
      <canvas
        ref={canvasRef}
        style={{
          imageRendering: 'pixelated',
          border: audioLevel?.is_clipping ? '1px solid #ff0000' : '1px solid #555555'
        }}
      />
      {audioLevel?.is_clipping && (
        <div style={{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: '100%',
          backgroundColor: 'rgba(255, 0, 0, 0.2)',
          pointerEvents: 'none',
          animation: 'blink 0.5s infinite'
        }} />
      )}
      <style jsx>{`
        @keyframes blink {
          0%, 50% { opacity: 1; }
          51%, 100% { opacity: 0; }
        }
      `}</style>
    </div>
  );
};

export default AudioLevelMeter;