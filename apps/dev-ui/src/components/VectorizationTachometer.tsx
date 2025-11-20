import React, { useRef, useEffect, useState } from 'react';
import { CockpitTooltip } from './CockpitTooltip';
import { COLORS, ColorZone, getColorFromZones, getLabelFromZones, normalizeValue, mapToRange } from '../utils/cockpit';

interface Props {
  ipc: number;
}

const IPC_MIN = 0;
const IPC_MAX = 4.0;
const SCALAR_LIMIT = 1.5;

const GAUGE_RADIUS = 80;
const GAUGE_THICKNESS = 15;
const GAUGE_START_ANGLE = Math.PI * 0.75;
const GAUGE_END_ANGLE = Math.PI * 2.25;

const COLOR_ZONES: ColorZone[] = [
  { start: 0.0, end: 1.0, color: COLORS.critical, label: 'Scalar/Stalled' },
  { start: 1.0, end: 1.5, color: COLORS.warning, label: 'Scalar Peak' },
  { start: 1.5, end: 2.5, color: COLORS.success, label: 'SIMD Active' },
  { start: 2.5, end: 4.0, color: COLORS.streaming, label: 'AVX2/512' },
];

const ipcToAngle = (ipc: number): number => {
  const normalized = normalizeValue(ipc, IPC_MIN, IPC_MAX);
  return mapToRange(normalized, GAUGE_START_ANGLE, GAUGE_END_ANGLE);
};

export const VectorizationTachometer: React.FC<Props> = ({ ipc }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [targetAngle, setTargetAngle] = useState(ipcToAngle(ipc));
  const [currentAngle, setCurrentAngle] = useState(ipcToAngle(ipc));
  const [showTooltip, setShowTooltip] = useState(false);

  useEffect(() => {
    setTargetAngle(ipcToAngle(ipc));
  }, [ipc]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;
    const centerX = width / 2;
    const centerY = height / 2 + 20;

    const smoothing = 0.15;
    const angleDiff = targetAngle - currentAngle;

    if (Math.abs(angleDiff) > 0.001) {
      setCurrentAngle((prev) => prev + angleDiff * smoothing);
    }

    let animationFrameId: number;
    const render = () => {
      ctx.clearRect(0, 0, width, height);

      for (const zone of COLOR_ZONES) {
        const startAngle = ipcToAngle(zone.start);
        const endAngle = ipcToAngle(zone.end);

        ctx.beginPath();
        ctx.arc(centerX, centerY, GAUGE_RADIUS, startAngle, endAngle);
        ctx.lineWidth = GAUGE_THICKNESS;
        ctx.strokeStyle = zone.color;
        ctx.stroke();
      }

      const scalarLimitAngle = ipcToAngle(SCALAR_LIMIT);
      const limitStartX = centerX + (GAUGE_RADIUS - GAUGE_THICKNESS / 2 - 5) * Math.cos(scalarLimitAngle);
      const limitStartY = centerY + (GAUGE_RADIUS - GAUGE_THICKNESS / 2 - 5) * Math.sin(scalarLimitAngle);
      const limitEndX = centerX + (GAUGE_RADIUS + GAUGE_THICKNESS / 2 + 5) * Math.cos(scalarLimitAngle);
      const limitEndY = centerY + (GAUGE_RADIUS + GAUGE_THICKNESS / 2 + 5) * Math.sin(scalarLimitAngle);

      ctx.beginPath();
      ctx.moveTo(limitStartX, limitStartY);
      ctx.lineTo(limitEndX, limitEndY);
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = 2;
      ctx.stroke();

      const needleLength = GAUGE_RADIUS - GAUGE_THICKNESS / 2;
      const needleX = centerX + needleLength * Math.cos(currentAngle);
      const needleY = centerY + needleLength * Math.sin(currentAngle);

      ctx.beginPath();
      ctx.moveTo(centerX, centerY);
      ctx.lineTo(needleX, needleY);
      ctx.strokeStyle = getColorFromZones(ipc, COLOR_ZONES);
      ctx.lineWidth = 3;
      ctx.stroke();

      ctx.beginPath();
      ctx.arc(centerX, centerY, 5, 0, Math.PI * 2);
      ctx.fillStyle = getColorFromZones(ipc, COLOR_ZONES);
      ctx.fill();

      animationFrameId = requestAnimationFrame(render);
    };

    render();

    return () => {
      cancelAnimationFrame(animationFrameId);
    };
  }, [currentAngle, targetAngle, ipc]);

  return (
    <div
      className="cockpit-panel"
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
    >
      <div className="cockpit-panel-title">IPC</div>
      <canvas
        ref={canvasRef}
        width={200}
        height={160}
        className="tachometer-canvas"
      />
      <div className="tachometer-value">
        {ipc.toFixed(2)}
      </div>
      <div
        className="tachometer-status"
        style={{ color: getColorFromZones(ipc, COLOR_ZONES) }}
      >
        {getLabelFromZones(ipc, COLOR_ZONES)}
      </div>

      {showTooltip && (
        <CockpitTooltip
          header="IPC (Instructions Per Cycle)"
          current={`Current: ${ipc.toFixed(2)} (${getLabelFromZones(ipc, COLOR_ZONES)} ✓)`}
          sections={[
            {
              title: 'What this means:',
              items: [
                { text: `Your code is executing ${ipc.toFixed(2)} instructions per CPU cycle` },
                { text: 'Values >1.5 indicate successful auto-vectorization' },
                ...(ipc >= 1.5
                  ? [{ text: 'This is EXCELLENT - Rust LLVM is using AVX SIMD', type: 'success' as const }]
                  : [{ text: 'Below vectorization threshold', type: 'warning' as const }]),
              ],
            },
            {
              title: 'What to watch for:',
              items: [
                { text: '⚠️ Drops to 1.0 → Recent commit broke vectorization' },
                { text: '→ Check movement/rotation systems', indent: true },
              ],
            },
          ]}
          target="Target: >1.5 (vectorized code)"
        />
      )}
    </div>
  );
};
