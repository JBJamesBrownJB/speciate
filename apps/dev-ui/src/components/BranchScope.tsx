import React, { useRef, useEffect, useState } from 'react';
import { CockpitTooltip } from './CockpitTooltip';
import { COLORS } from '../utils/cockpit';

interface Props {
  branchMissRate: number;
}

const SCOPE_RADIUS = 100;
const PARTICLE_COUNT = 50;
const CANVAS_SIZE = 220;

interface Particle {
  angle: number;
  distance: number;
}

const getScopeState = (missRate: number) => {
  if (missRate < 1) {
    return {
      color: COLORS.success,
      label: 'Archetypes Stable',
      maxRadius: 20,
    };
  }
  if (missRate < 5) {
    return {
      color: COLORS.warning,
      label: 'Moderate Fragmentation',
      maxRadius: 50,
    };
  }
  return {
    color: COLORS.critical,
    label: '⚠️ Archetype Thrashing',
    maxRadius: SCOPE_RADIUS,
  };
};

const generateParticles = (missRate: number, maxRadius: number): Particle[] => {
  const particles: Particle[] = [];
  const spreadFactor = Math.min(1, missRate / 5);

  for (let i = 0; i < PARTICLE_COUNT; i++) {
    const angle = (Math.PI * 2 * i) / PARTICLE_COUNT + Math.random() * 0.2;
    const normalizedDistance = Math.random();
    const distance = normalizedDistance * maxRadius * (0.5 + spreadFactor * 0.5);

    particles.push({ angle, distance });
  }

  return particles;
};

export const BranchScope: React.FC<Props> = ({ branchMissRate }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [showTooltip, setShowTooltip] = useState(false);
  const particlesRef = useRef<Particle[]>([]);
  const targetParticlesRef = useRef<Particle[]>([]);

  const scopeState = getScopeState(branchMissRate);

  useEffect(() => {
    const newTargetParticles = generateParticles(branchMissRate, scopeState.maxRadius);
    targetParticlesRef.current = newTargetParticles;

    if (particlesRef.current.length === 0) {
      particlesRef.current = newTargetParticles;
    }
  }, [branchMissRate, scopeState.maxRadius]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const centerX = CANVAS_SIZE / 2;
    const centerY = CANVAS_SIZE / 2;

    let animationFrameId: number;

    const render = () => {
      ctx.clearRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);

      for (let i = 0; i < particlesRef.current.length; i++) {
        const current = particlesRef.current[i];
        const target = targetParticlesRef.current[i];

        const distDiff = target.distance - current.distance;
        const angleDiff = target.angle - current.angle;

        if (Math.abs(distDiff) > 0.5 || Math.abs(angleDiff) > 0.01) {
          current.distance += distDiff * 0.1;
          current.angle += angleDiff * 0.1;
        }

        const x = centerX + current.distance * Math.cos(current.angle);
        const y = centerY + current.distance * Math.sin(current.angle);

        ctx.beginPath();
        ctx.arc(x, y, 2, 0, Math.PI * 2);
        ctx.fillStyle = scopeState.color;
        ctx.fill();
      }

      ctx.strokeStyle = scopeState.color;
      ctx.lineWidth = 2;

      ctx.beginPath();
      ctx.moveTo(centerX - 15, centerY);
      ctx.lineTo(centerX - 5, centerY);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(centerX + 5, centerY);
      ctx.lineTo(centerX + 15, centerY);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(centerX, centerY - 15);
      ctx.lineTo(centerX, centerY - 5);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(centerX, centerY + 5);
      ctx.lineTo(centerX, centerY + 15);
      ctx.stroke();

      ctx.beginPath();
      ctx.arc(centerX, centerY, SCOPE_RADIUS, 0, Math.PI * 2);
      ctx.strokeStyle = 'rgba(203, 213, 225, 0.3)';
      ctx.lineWidth = 1;
      ctx.stroke();

      ctx.beginPath();
      ctx.arc(centerX, centerY, 3, 0, Math.PI * 2);
      ctx.strokeStyle = scopeState.color;
      ctx.lineWidth = 2;
      ctx.stroke();

      animationFrameId = requestAnimationFrame(render);
    };

    render();

    return () => {
      cancelAnimationFrame(animationFrameId);
    };
  }, [scopeState.color]);

  return (
    <div
      className="cockpit-panel"
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
    >
      <div className="cockpit-panel-title">Prediction</div>
      <canvas
        ref={canvasRef}
        width={CANVAS_SIZE}
        height={CANVAS_SIZE}
        className="scope-canvas"
      />
      <div className="scope-value">
        {branchMissRate.toFixed(1)}%
      </div>
      <div
        className="scope-status"
        style={{ color: scopeState.color }}
      >
        {scopeState.label}
      </div>

      {showTooltip && (
        <CockpitTooltip
          header="Prediction (Branch Accuracy)"
          current={`Current: ${(100 - branchMissRate).toFixed(1)}% (${branchMissRate.toFixed(1)}% miss rate) ✓`}
          sections={[
            {
              title: 'What this means:',
              items: [
                { text: 'Your ECS archetypes are perfectly sorted' },
                { text: 'CPU branch predictor is ~100% accurate' },
                { text: 'Entity component patterns are predictable' },
                ...(branchMissRate < 1
                  ? [
                      { text: 'This is EXCELLENT - indicates:', type: 'success' as const },
                      { text: '✓ With<CanSeek> filters working perfectly', indent: true },
                      { text: '✓ Minimal component add/remove churn', indent: true },
                      { text: '✓ Stable archetype structure', indent: true },
                    ]
                  : [{ text: 'Archetype fragmentation detected', type: 'warning' as const }]),
              ],
            },
            {
              title: 'What to watch for:',
              items: [
                { text: '⚠️ Miss rate >5% → Archetype fragmentation' },
                { text: '→ New system randomly adding/removing components', indent: true },
                { text: '→ Check behavior_transition logic', indent: true },
              ],
            },
          ]}
          target="Best indicator of: ECS query efficiency"
        />
      )}
    </div>
  );
};
