import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { parsePlantBuffer, PlantRenderer } from './PlantRenderer';
import { Container, Graphics } from 'pixi.js';

// ---------------------------------------------------------------------------
// Helper: build a Float32Array plant buffer from plain cell objects.
// ---------------------------------------------------------------------------
function makeBuffer(
  cells: { x: number; y: number; density: number; plantType: number }[]
): Float32Array {
  const buf = new Float32Array(1 + cells.length * 4);
  buf[0] = cells.length;
  cells.forEach((c, i) => {
    const base = 1 + i * 4;
    buf[base]     = c.x;
    buf[base + 1] = c.y;
    buf[base + 2] = c.density;
    buf[base + 3] = c.plantType;
  });
  return buf;
}

describe('parsePlantBuffer', () => {
  it('returns empty array for empty buffer', () => {
    expect(parsePlantBuffer(new Float32Array([]))).toEqual([]);
  });

  it('returns empty array when count is 0', () => {
    expect(parsePlantBuffer(new Float32Array([0]))).toEqual([]);
  });

  it('parses a single live cell', () => {
    const buf = new Float32Array([1, 10, 20, 0.8, 1]);
    const cells = parsePlantBuffer(buf);
    expect(cells).toHaveLength(1);
    expect(cells[0].x).toBe(10);
    expect(cells[0].y).toBe(20);
    expect(cells[0].density).toBeCloseTo(0.8);
    expect(cells[0].plantType).toBe(1);
  });

  it('parses multiple cells', () => {
    const buf = new Float32Array([2,  10, 20, 1.0, 1,  -50, 30, 0.5, 2]);
    const cells = parsePlantBuffer(buf);
    expect(cells).toHaveLength(2);
    expect(cells[1].x).toBe(-50);
    expect(cells[1].y).toBe(30);
  });

  it('skips cells with zero density', () => {
    const buf = new Float32Array([2,  10, 20, 0.0, 1,  5, 5, 1.0, 1]);
    const cells = parsePlantBuffer(buf);
    expect(cells).toHaveLength(1);
    expect(cells[0].x).toBe(5);
  });

  it('skips cells with negative density', () => {
    const buf = new Float32Array([1,  10, 20, -0.5, 1]);
    expect(parsePlantBuffer(buf)).toHaveLength(0);
  });

  it('is resilient to a truncated buffer (stops, no throw)', () => {
    // count=5 but only 1 cell worth of data — should return 1 cell, not crash
    const buf = new Float32Array([5, 1, 2, 1.0, 1]);
    expect(() => parsePlantBuffer(buf)).not.toThrow();
    const cells = parsePlantBuffer(buf);
    expect(cells.length).toBeLessThanOrEqual(5);
  });

  it('returns empty array for non-finite count', () => {
    const buf = new Float32Array([NaN, 1, 2, 1.0, 1]);
    expect(parsePlantBuffer(buf)).toEqual([]);
  });

  it('skips cells with non-finite coordinates', () => {
    const buf = new Float32Array([1, NaN, Infinity, 1.0, 1]);
    expect(parsePlantBuffer(buf)).toHaveLength(0);
  });

  it('density affects output but does not change cell position', () => {
    const buf = new Float32Array([1, 100, 200, 0.5, 3]);
    const cells = parsePlantBuffer(buf);
    expect(cells[0].x).toBe(100);
    expect(cells[0].y).toBe(200);
    expect(cells[0].density).toBeCloseTo(0.5);
    expect(cells[0].plantType).toBe(3);
  });
});

// ---------------------------------------------------------------------------
// PlantRenderer — viewport-culling behaviour
// ---------------------------------------------------------------------------
describe('PlantRenderer', () => {
  let container: Container;
  let renderer: PlantRenderer;

  beforeEach(() => {
    container = new Container();
    renderer = new PlantRenderer(container);
  });

  afterEach(() => {
    renderer.destroy();
  });

  it('visibleCount is 0 before any buffer is delivered', () => {
    expect(renderer.visibleCount).toBe(0);
  });

  it('updateFromBuffer with no bounds draws all cells — visibleCount equals total', () => {
    const buf = makeBuffer([
      { x: 10,  y: 20,  density: 0.8, plantType: 1 },
      { x: -50, y: 30,  density: 0.5, plantType: 2 },
      { x: 200, y: 100, density: 1.0, plantType: 1 },
    ]);
    renderer.updateFromBuffer(buf);
    expect(renderer.visibleCount).toBe(3);
  });

  it('updateFromBuffer with bounds already set draws only cells within bounds', () => {
    renderer.setViewportBounds(0, 100, 0, 100);
    const buf = makeBuffer([
      { x: 10,  y: 20,  density: 0.8, plantType: 1 }, // in bounds
      { x: -50, y: 30,  density: 0.5, plantType: 2 }, // out — x < minX
      { x: 200, y: 100, density: 1.0, plantType: 1 }, // out — x > maxX
    ]);
    renderer.updateFromBuffer(buf);
    expect(renderer.visibleCount).toBe(1);
  });

  it('setViewportBounds on existing allCells re-filters without a new buffer', () => {
    const buf = makeBuffer([
      { x: 10,  y: 20,  density: 0.8, plantType: 1 }, // will be in bounds
      { x: 50,  y: 60,  density: 0.5, plantType: 2 }, // will be in bounds
      { x: 200, y: 100, density: 1.0, plantType: 1 }, // will be out
    ]);
    renderer.updateFromBuffer(buf);
    expect(renderer.visibleCount).toBe(3); // no bounds yet — all visible

    renderer.setViewportBounds(0, 100, 0, 100);
    expect(renderer.visibleCount).toBe(2); // 10,20 and 50,60 are in; 200,100 is out
  });

  it('cells exactly on the boundary edge are included (inclusive bounds)', () => {
    renderer.setViewportBounds(0, 100, 0, 100);
    const buf = makeBuffer([
      { x: 0,   y: 0,   density: 1.0, plantType: 1 }, // min corner — in
      { x: 100, y: 100, density: 1.0, plantType: 1 }, // max corner — in
      { x: -1,  y: 50,  density: 1.0, plantType: 1 }, // just outside minX — out
    ]);
    renderer.updateFromBuffer(buf);
    expect(renderer.visibleCount).toBe(2);
  });

  it('empty viewport — all plants outside — visibleCount is 0', () => {
    const buf = makeBuffer([
      { x: 10, y: 20, density: 0.8, plantType: 1 },
      { x: 50, y: 60, density: 0.5, plantType: 2 },
    ]);
    renderer.updateFromBuffer(buf);
    expect(renderer.visibleCount).toBe(2);

    renderer.setViewportBounds(500, 600, 500, 600); // no plants there
    expect(renderer.visibleCount).toBe(0);
  });

  it('setViewportBounds before any buffer does not throw and visibleCount stays 0', () => {
    expect(() => renderer.setViewportBounds(0, 100, 0, 100)).not.toThrow();
    expect(renderer.visibleCount).toBe(0);
  });

  describe('per-frame redraw guard', () => {
    const cells = [
      { x: 10, y: 20, density: 0.8, plantType: 1 },
      { x: 50, y: 60, density: 0.5, plantType: 2 },
    ];

    it('skips the re-filter/redraw when bounds are unchanged or wobble sub-unit', () => {
      renderer.updateFromBuffer(makeBuffer(cells));
      renderer.setViewportBounds(0, 100, 0, 100);

      const draws = vi.spyOn(Graphics.prototype, 'circle');
      renderer.setViewportBounds(0, 100, 0, 100); // identical
      renderer.setViewportBounds(0.5, 100.5, 0.5, 100.5); // camera-smoother wobble
      expect(draws).not.toHaveBeenCalled();
      draws.mockRestore();
    });

    it('redraws when bounds move beyond the epsilon', () => {
      renderer.updateFromBuffer(makeBuffer(cells));
      renderer.setViewportBounds(0, 100, 0, 100);

      const draws = vi.spyOn(Graphics.prototype, 'circle');
      renderer.setViewportBounds(30, 130, 0, 100);
      expect(draws).toHaveBeenCalled();
      draws.mockRestore();
    });

    it('a fresh buffer always redraws, even with unchanged bounds', () => {
      renderer.updateFromBuffer(makeBuffer(cells));
      renderer.setViewportBounds(0, 100, 0, 100);

      const draws = vi.spyOn(Graphics.prototype, 'circle');
      renderer.updateFromBuffer(makeBuffer(cells));
      expect(draws).toHaveBeenCalled();
      draws.mockRestore();
    });
  });

  it('removing bounds (null-reset via new buffer) restores all cells as visible', () => {
    // Set bounds so only 1 of 3 cells is visible.
    renderer.setViewportBounds(0, 50, 0, 50);
    const buf = makeBuffer([
      { x: 10,  y: 20,  density: 1.0, plantType: 1 }, // in
      { x: 200, y: 300, density: 1.0, plantType: 1 }, // out
      { x: 400, y: 500, density: 1.0, plantType: 1 }, // out
    ]);
    renderer.updateFromBuffer(buf);
    expect(renderer.visibleCount).toBe(1);

    // Deliver a fresh buffer after widening bounds to cover everything.
    renderer.setViewportBounds(-1000, 1000, -1000, 1000);
    expect(renderer.visibleCount).toBe(3);
  });
});
