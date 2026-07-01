import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { Container } from 'pixi.js';
import { SpatialGridOverlay, GridMode } from './SpatialGridOverlay';
import { SPATIAL_GRID_CONFIG } from '@/core/constants';

describe('SpatialGridOverlay', () => {
  let container: Container;
  let overlay: SpatialGridOverlay;

  beforeEach(() => {
    container = new Container();
    overlay = new SpatialGridOverlay(container);
  });

  afterEach(() => {
    overlay.destroy();
  });

  describe('default cell sizes (before first telemetry frame)', () => {
    it('defaults to the real engine grid: L0 = 20 m, L1 = 60 m (not the stale 10/30)', () => {
      expect(overlay.getL0CellSize()).toBe(20);
      expect(overlay.getL1CellSize()).toBe(60);
    });

    it('sources the defaults from the shared constant', () => {
      expect(overlay.getL0CellSize()).toBe(SPATIAL_GRID_CONFIG.L0_CELL_SIZE);
      expect(overlay.getL1CellSize()).toBe(SPATIAL_GRID_CONFIG.L1_CELL_SIZE);
    });

    it('telemetry still overrides the defaults', () => {
      overlay.setCellSize(25);
      overlay.setL1CellSize(75);
      expect(overlay.getL0CellSize()).toBe(25);
      expect(overlay.getL1CellSize()).toBe(75);
    });
  });

  describe('G-key cycle (MODE_ORDER)', () => {
    it('cycles Off → L0 → L1 → Off (P0 removed from cycle)', () => {
      expect(overlay.getMode()).toBe(GridMode.Off);
      overlay.toggle();
      expect(overlay.getMode()).toBe(GridMode.L0);
      overlay.toggle();
      expect(overlay.getMode()).toBe(GridMode.L1);
      overlay.toggle();
      expect(overlay.getMode()).toBe(GridMode.Off);
    });

    it('does not include P0 in the G-key cycle', () => {
      // Cycle through all states — P0 should never appear
      const modes: GridMode[] = [];
      for (let i = 0; i < 6; i++) {
        overlay.toggle();
        modes.push(overlay.getMode());
      }
      expect(modes).not.toContain(GridMode.P0);
    });
  });

  describe('setMode', () => {
    it('can jump directly to P0 (for toolbar use)', () => {
      overlay.setMode(GridMode.P0);
      expect(overlay.getMode()).toBe(GridMode.P0);
    });

    it('can return from P0 to Off', () => {
      overlay.setMode(GridMode.P0);
      overlay.setMode(GridMode.Off);
      expect(overlay.getMode()).toBe(GridMode.Off);
    });
  });

  describe('hasPlantAt', () => {
    it('returns false for an empty overlay', () => {
      expect(overlay.hasPlantAt(10, 10)).toBe(false);
    });

    it('returns true for a coordinate that falls inside a planted cell', () => {
      // Cell size = 4. Plant at center (2, 2) of cell (0,0)→(4,4).
      const buf = new Float32Array([1, 2, 2, 1.0, 1]);
      overlay.updateP0Cells(buf);
      // Query anywhere in the same 4m cell
      expect(overlay.hasPlantAt(1, 1)).toBe(true);
      expect(overlay.hasPlantAt(3.9, 3.9)).toBe(true);
    });

    it('returns false for a coordinate in an adjacent empty cell', () => {
      const buf = new Float32Array([1, 2, 2, 1.0, 1]);
      overlay.updateP0Cells(buf);
      // Next cell starts at x=4
      expect(overlay.hasPlantAt(5, 2)).toBe(false);
    });

    it('returns false after updateP0Cells clears previous data with empty buffer', () => {
      const buf = new Float32Array([1, 2, 2, 1.0, 1]);
      overlay.updateP0Cells(buf);
      expect(overlay.hasPlantAt(2, 2)).toBe(true);

      overlay.updateP0Cells(new Float32Array([0]));
      expect(overlay.hasPlantAt(2, 2)).toBe(false);
    });
  });
});
