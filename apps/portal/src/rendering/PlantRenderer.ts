import { Graphics, Container } from 'pixi.js';

/** Floats per live cell in the sparse buffer: x, y, density, plant_type. */
export const PLANT_FLOATS_PER_CELL = 4;

export interface PlantCellData {
  x: number;
  y: number;
  density: number;
  plantType: number;
}

/**
 * Parse a sparse plant buffer into an array of live cell data.
 *
 * Pure function — no PixiJS dependency. Testable in isolation.
 *
 * Buffer format (matches Rust `PlantGrid::write_sparse`):
 *   [count, x₀, y₀, density₀, type₀, x₁, y₁, density₁, type₁, ...]
 */
export function parsePlantBuffer(buf: Float32Array): PlantCellData[] {
  if (buf.length < 1) return [];

  const count = buf[0];
  if (!Number.isFinite(count) || count <= 0) return [];

  const result: PlantCellData[] = [];
  for (let i = 0; i < count; i++) {
    const base = 1 + i * PLANT_FLOATS_PER_CELL;
    if (base + PLANT_FLOATS_PER_CELL > buf.length) break; // truncated — stop safely

    const x = buf[base];
    const y = buf[base + 1];
    const density = buf[base + 2];
    const plantType = buf[base + 3];

    if (!Number.isFinite(x) || !Number.isFinite(y) || density <= 0) continue;

    result.push({ x, y, density, plantType });
  }
  return result;
}

interface ViewportBounds {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
}

/**
 * Renders the P0 plant grid as simple coloured circles in world space.
 *
 * This is the Phase 1 lean-slice visual — each live plant cell is a filled
 * circle at its world-space centre. Phase 4 will replace this with a WebGL
 * ground shader reading from a density texture.
 */
export class PlantRenderer {
  private graphics: Graphics;
  private allCells: PlantCellData[] = [];
  private viewportBounds: ViewportBounds | null = null;
  private _visibleCount = 0;

  constructor(container: Container) {
    this.graphics = new Graphics();
    container.addChildAt(this.graphics, 0); // below creatures
  }

  /**
   * Update the plant render from a fresh sparse buffer snapshot.
   * Called whenever the Electron main process delivers a new plant update.
   */
  updateFromBuffer(buf: Float32Array): void {
    this.allCells = parsePlantBuffer(buf);
    this.renderVisible();
  }

  /**
   * Set the world-space viewport bounds for culling.
   * Only cells whose centre falls within [minX, maxX] × [minY, maxY]
   * (inclusive) will be drawn. Triggers an immediate re-filter and redraw.
   */
  setViewportBounds(minX: number, maxX: number, minY: number, maxY: number): void {
    this.viewportBounds = { minX, maxX, minY, maxY };
    this.renderVisible();
  }

  /** Number of plant cells currently rendered (within viewport bounds). */
  get visibleCount(): number {
    return this._visibleCount;
  }

  /**
   * Filter allCells to the current viewport bounds (inclusive) and redraw.
   * When no bounds are set every cell is rendered — backward-compatible.
   */
  private renderVisible(): void {
    const cells =
      this.viewportBounds === null
        ? this.allCells
        : this.allCells.filter(({ x, y }) => {
            const { minX, maxX, minY, maxY } = this.viewportBounds!;
            return x >= minX && x <= maxX && y >= minY && y <= maxY;
          });

    this.graphics.clear();
    for (const { x, y, density } of cells) {
      const alpha = Math.min(1, density * 0.7 + 0.3);
      this.graphics.fill({ color: 0x2d8a4e, alpha });
      this.graphics.circle(x, y, 2);
      this.graphics.fill();
    }
    this._visibleCount = cells.length;
  }

  destroy(): void {
    this.graphics.destroy();
  }
}
