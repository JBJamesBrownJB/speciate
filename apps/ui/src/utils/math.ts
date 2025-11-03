import type { Vec3 } from '@/types/entities';

export function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function lerpVec3(a: Vec3, b: Vec3, t: number): Vec3 {
  return {
    x: lerp(a.x, b.x, t),
    y: lerp(a.y, b.y, t),
    z: lerp(a.z, b.z, t),
  };
}

export function calculateInterpolationFactor(
  currentTime: number,
  currentUpdateTime: number,
  updateInterval: number = 100
): number {
  const timeSinceUpdate = currentTime - currentUpdateTime;
  const t = timeSinceUpdate / updateInterval;
  return clamp(t, 0, 1);
}
