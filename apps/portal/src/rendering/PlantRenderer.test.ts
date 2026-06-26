import { describe, it, expect } from 'vitest';
import { parsePlantBuffer } from './PlantRenderer';

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
