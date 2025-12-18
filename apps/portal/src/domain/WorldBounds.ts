export interface WorldBounds {
  readonly minX: number;
  readonly maxX: number;
  readonly minY: number;
  readonly maxY: number;
}

export function createWorldBounds(
  minX: number,
  maxX: number,
  minY: number,
  maxY: number
): WorldBounds {
  return { minX, maxX, minY, maxY };
}

export function worldBoundsFromDimensions(
  width: number,
  height: number
): WorldBounds {
  const halfWidth = width / 2;
  const halfHeight = height / 2;
  return {
    minX: -halfWidth,
    maxX: halfWidth,
    minY: -halfHeight,
    maxY: halfHeight,
  };
}

export function worldBoundsWidth(bounds: WorldBounds): number {
  return bounds.maxX - bounds.minX;
}

export function worldBoundsHeight(bounds: WorldBounds): number {
  return bounds.maxY - bounds.minY;
}

export function worldBoundsContains(
  bounds: WorldBounds,
  x: number,
  y: number
): boolean {
  return (
    x >= bounds.minX && x <= bounds.maxX && y >= bounds.minY && y <= bounds.maxY
  );
}
